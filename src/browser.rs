use crate::ipc::{
    Bookmark, BrowserModule, BrowserSession, BrowserSettings, DownloadRecord, HistoryEntry,
    IpcIncoming, IpcTabInfo, SessionTab,
};
use crate::storage::StorageManager;
use crate::updater::{UpdateCheckResult, UpdateInfo, UpdateState, UpdateStatus};
use crate::url_utils::normalize_or_search_url_with_engine;
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::Arc;
use std::thread;
use tao::dpi::{LogicalPosition, LogicalSize};
use tao::event_loop::EventLoopProxy;
use tao::window::Window;
use wry::{
    NewWindowResponse, PageLoadEvent, Rect, WebView, WebViewBuilder, WebViewBuilderExtWindows,
    WebViewExtWindows,
};

pub const HEADER_HEIGHT_COLLAPSED: f64 = 76.0;
pub const HEADER_HEIGHT_EXPANDED: f64 = 102.0;

fn render_typescript(template: &str, placeholder: &str, config: serde_json::Value) -> String {
    let config_json = serde_json::to_string(&config).unwrap_or_else(|_| "{}".into());
    template.replace(placeholder, &config_json)
}

fn desktop_command(command: serde_json::Value) -> String {
    render_typescript(
        include_str!("../web-scripts/dist/desktop-command.js"),
        "__TITAN_DESKTOP_COMMAND__",
        command,
    )
}

fn prepare_download_destination(
    url: &str,
    destination: &mut std::path::PathBuf,
) -> Option<std::path::PathBuf> {
    if let Some(configured_dir) = std::env::var_os("TITAN_DOWNLOAD_DIR") {
        let directory = std::path::PathBuf::from(configured_dir);
        if let Err(error) = std::fs::create_dir_all(&directory) {
            eprintln!(
                "Could not create download directory {}: {error}",
                directory.display()
            );
        } else {
            let file_name = destination
                .file_name()
                .filter(|name| !name.is_empty())
                .map(std::ffi::OsStr::to_os_string)
                .or_else(|| {
                    url::Url::parse(url)
                        .ok()?
                        .path_segments()?
                        .next_back()
                        .filter(|segment| !segment.is_empty())
                        .map(std::ffi::OsString::from)
                })
                .unwrap_or_else(|| std::ffi::OsString::from("download"));
            *destination = directory.join(file_name);
        }
    }

    (!destination.as_os_str().is_empty()).then(|| destination.clone())
}

fn desktop_adblock_script(
    manager: &crate::adblock_engine::AdblockEngineManager,
    settings: &BrowserSettings,
    target_url: &str,
) -> String {
    if !settings.adblock_enabled {
        return String::new();
    }
    let (dynamic_selectors, dynamic_scriptlet) = manager.get_cosmetic_resources(target_url);
    render_typescript(
        include_str!("../web-scripts/dist/desktop-adblock.js"),
        "__TITAN_DESKTOP_ADBLOCK_CONFIG__",
        serde_json::json!({
            "enabled": settings.adblock_enabled,
            "blockVideoAds": settings.adblock_block_video_ads,
            "cosmeticFiltering": settings.adblock_cosmetic_filtering,
            "blockPopups": settings.adblock_block_popups,
            "aggressiveMode": settings.adblock_aggressive_mode,
            "whitelistedDomains": settings.adblock_whitelisted_domains,
            "blockedDomains": settings.adblock_blocked_domains,
            "dynamicSelectors": dynamic_selectors,
            "scriptletCode": dynamic_scriptlet,
        }),
    )
}

fn desktop_extensions_script(extensions: &[crate::extensions::ExtensionInfo]) -> String {
    let installed_ids: Vec<String> = extensions.iter().map(|e| e.id.clone()).collect();
    render_typescript(
        include_str!("../web-scripts/dist/desktop-extensions.js"),
        "__TITAN_DESKTOP_EXTENSIONS_CONFIG__",
        serde_json::json!({
            "installedIds": installed_ids,
        }),
    )
}

#[derive(Debug)]
pub enum UserEvent {
    Ipc(String),
    UpdateCheckFinished(UpdateCheckResult),
    PageLoadStarted {
        tab_id: u32,
        url: String,
    },
    PageLoadFinished {
        tab_id: u32,
        url: String,
    },
    OpenPopup {
        url: String,
    },
    DownloadStarted {
        url: String,
        path: Option<std::path::PathBuf>,
    },
    DownloadCompleted {
        url: String,
        path: Option<std::path::PathBuf>,
        success: bool,
    },
    AdoptPopup {
        tab_id: u32,
    },
    Exit,
}

fn popup_target_url(requested_url: &str) -> Option<String> {
    let requested_url = requested_url.trim();
    if requested_url.is_empty() || requested_url.eq_ignore_ascii_case("about:blank") {
        return Some("titan://newtab".into());
    }

    let parsed = url::Url::parse(requested_url).ok()?;
    matches!(parsed.scheme(), "http" | "https" | "chrome-extension").then(|| parsed.into())
}

fn is_allowed_content_ipc(message: &str, own_tab_id: u32) -> bool {
    let Ok(message) = serde_json::from_str::<IpcIncoming>(message) else {
        return false;
    };
    match message {
        IpcIncoming::NewTab { url } => url.as_deref().is_none_or(BrowserManager::is_newtab_url),
        IpcIncoming::CloseTab { tab_id } => tab_id == own_tab_id,
        IpcIncoming::GoBack
        | IpcIncoming::GoForward
        | IpcIncoming::Reload
        | IpcIncoming::FocusAddressBar
        | IpcIncoming::OpenSettings
        | IpcIncoming::OpenHistory
        | IpcIncoming::OpenDownloads => true,
        IpcIncoming::NewPrivateTab => true,
        IpcIncoming::InstallExtension { .. } => true,
        IpcIncoming::OpenExtensionPopup { .. } => true,
        IpcIncoming::OpenExtensionOptions { .. } => true,
        IpcIncoming::SetHeaderExpanded { .. } => true,
        IpcIncoming::TabStateUpdate { tab_id, .. } => tab_id == Some(own_tab_id),
        IpcIncoming::ReportBlockedRequest { .. } | IpcIncoming::ReportBlockedAd { .. } => true,
        _ => false,
    }
}

pub struct Tab {
    pub id: u32,
    pub url: String,
    pub title: String,
    pub is_loading: bool,
    pub can_go_back: bool,
    pub can_go_forward: bool,
    pub is_private: bool,
    pub webview: WebView,
}

pub struct BrowserManager {
    pub window: Arc<Window>,
    pub proxy: EventLoopProxy<UserEvent>,
    pub storage: StorageManager,
    pub header_webview: Option<WebView>,
    pub prewarmed_settings_tab: Option<Tab>,
    pub tabs: Vec<Tab>,
    pub active_tab_id: Option<u32>,
    pub next_tab_id: Rc<Cell<u32>>,
    pub bookmarks: Vec<Bookmark>,
    pub modules: Vec<BrowserModule>,
    pub settings: BrowserSettings,
    pub zoom: f64,
    pub window_size: (f64, f64),
    pub blocked_logs: Vec<crate::ipc::BlockedRequestLog>,
    pub adblock_logs: Vec<crate::ipc::BlockedRequestLog>,
    pub adblock_manager: Rc<crate::adblock_engine::AdblockEngineManager>,
    #[cfg(target_os = "windows")]
    pub desktop_adblock_settings: crate::desktop_adblock::SharedDesktopAdblockSettings,
    pub update_state: UpdateState,
    pub history: Vec<HistoryEntry>,
    pub downloads: Vec<DownloadRecord>,
    pub extensions: Vec<crate::extensions::ExtensionInfo>,
    pub header_expanded: bool,
    next_download_id: u64,
    pending_popup_tabs: Rc<RefCell<Vec<Tab>>>,
    is_restoring_session: bool,
}

impl BrowserManager {
    pub fn new(window: Arc<Window>, proxy: EventLoopProxy<UserEvent>) -> Self {
        let storage = StorageManager::new();
        let bookmarks = storage.load_bookmarks();
        let modules = storage.load_modules();
        let settings = storage.load_settings();
        let history = storage.load_history();
        let extensions = storage.load_extensions();
        let mut downloads = storage.load_downloads();
        for download in &mut downloads {
            if download.status == "downloading" {
                download.status = "interrupted".into();
            }
        }
        let next_download_id = downloads
            .iter()
            .map(|download| download.id)
            .max()
            .unwrap_or(0)
            .saturating_add(1);
        storage.save_downloads(&downloads);
        let win_size = window.inner_size().to_logical::<f64>(window.scale_factor());

        let adblock_manager = Rc::new(crate::adblock_engine::AdblockEngineManager::new());
        for rule in &settings.adblock_custom_rules {
            adblock_manager.add_custom_rule(rule.clone());
        }
        for list in &[
            "easylist",
            "easyprivacy",
            "ublock_filters",
            "ublock_badware",
            "ublock_privacy",
            "ublock_quick_fixes",
            "turtlecute_test",
        ] {
            let enabled = settings.adblock_filter_lists.iter().any(|l| l == list);
            adblock_manager.toggle_filter_list(list, enabled);
        }

        #[cfg(target_os = "windows")]
        let desktop_adblock_settings = crate::desktop_adblock::shared_settings(&settings);

        Self {
            window,
            proxy,
            storage,
            header_webview: None,
            prewarmed_settings_tab: None,
            tabs: Vec::new(),
            active_tab_id: None,
            next_tab_id: Rc::new(Cell::new(1)),
            bookmarks,
            modules,
            settings,
            zoom: 1.0,
            window_size: (win_size.width, win_size.height),
            blocked_logs: Vec::new(),
            adblock_logs: Vec::new(),
            adblock_manager,
            #[cfg(target_os = "windows")]
            desktop_adblock_settings,
            update_state: UpdateState::default(),
            history,
            downloads,
            extensions,
            header_expanded: false,
            next_download_id,
            pending_popup_tabs: Rc::new(RefCell::new(Vec::new())),
            is_restoring_session: false,
        }
    }

    pub fn get_header_height(&self) -> f64 {
        if self.header_expanded {
            460.0
        } else if self.settings.show_bookmarks_bar && !self.bookmarks.is_empty() {
            HEADER_HEIGHT_EXPANDED
        } else {
            HEADER_HEIGHT_COLLAPSED
        }
    }

    pub fn init(&mut self, initial_url: Option<&str>) {
        let (r, g, b, a) = self.get_theme_background_color();
        #[cfg(target_os = "windows")]
        crate::drag_util::apply_dark_window_attributes(&self.window, (r, g, b));

        let (width, _) = self.window_size;
        let header_height = self.get_header_height();
        let header_bounds = Rect {
            position: LogicalPosition::new(0.0, 0.0).into(),
            size: LogicalSize::new(width, header_height).into(),
        };

        let proxy_clone = self.proxy.clone();
        let html_content = Self::get_chrome_html();

        #[allow(unused_mut)]
        let mut header_builder = WebViewBuilder::new();
        #[cfg(target_os = "windows")]
        {
            header_builder = header_builder.with_browser_extensions_enabled(true);
        }
        let header = header_builder
            .with_bounds(header_bounds)
            .with_background_color((r, g, b, a))
            .with_transparent(false)
            .with_html(&html_content)
            .with_ipc_handler(move |req| {
                let _ = proxy_clone.send_event(UserEvent::Ipc(req.body().clone()));
            })
            .build_as_child(&*self.window)
            .expect("Failed to create header webview");

        self.header_webview = Some(header);

        #[cfg(target_os = "windows")]
        if let Some(ref view) = self.header_webview {
            for ext in &self.extensions {
                if ext.enabled && std::path::Path::new(&ext.path).exists() {
                    let _ = crate::desktop_data::install_browser_extension(view, &ext.path);
                }
            }
        }

        self.restore_session();
        if let Some(url) = initial_url.and_then(popup_target_url) {
            self.open_external_url(&url);
        }
        self.prewarm_settings_tab();

        if self.settings.auto_update_enabled {
            self.check_for_updates();
        }
    }

    pub fn get_chrome_html() -> String {
        let html = include_str!("../ui/index.html");
        let css = include_str!("../ui/style.css");
        let js = include_str!("../ui/dist/app.js");

        html.replace(
            "<link rel=\"stylesheet\" href=\"style.css\" />",
            &format!("<style>{}</style>", css),
        )
        .replace(
            "<script src=\"app.js\"></script>",
            &format!("<script>{}</script>", js),
        )
    }

    pub fn get_settings_html(&self, active_section: &str) -> String {
        let html = include_str!("../ui/settings.html");
        let js = include_str!("../ui/dist/settings.js");
        let theme_class = format!("theme-{}", self.settings.theme);
        let html_themed = html.replace(
            "class=\"theme-titan-dark\"",
            &format!("class=\"{}\"", theme_class),
        );
        let state_json = serde_json::to_string(&serde_json::json!({
            "settings": self.settings,
            "modules": self.modules,
            "blocked_logs": self.blocked_logs,
            "adblock_logs": self.adblock_logs,
            "adblock_filter_lists": self.adblock_manager.get_filter_lists_info(),
            "adblock_stats": self.adblock_manager.get_stats(),
            "adblock_custom_rules": self.adblock_manager.get_custom_rules(),
            "mandatory_blocked_domains": crate::privacy::BLOCKED_TELEMETRY_DOMAINS,
            "update_state": self.update_state,
            "extensions": &self.extensions,
            "active_section": active_section,
        }))
        .unwrap_or_else(|_| "{}".into())
        .replace('<', "\\u003c");

        html_themed.replace(
            "<script src=\"settings.js\"></script>",
            &format!(
                "<script id=\"titan-settings-state\" type=\"application/json\">{}</script><script>{}</script>",
                state_json, js
            ),
        )
    }

    pub fn get_newtab_html(&self) -> String {
        let html = include_str!("../ui/newtab.html");
        let js = include_str!("../ui/dist/newtab.js");
        let theme_class = format!("theme-{}", self.settings.theme);
        let html_themed = html.replace(
            "class=\"theme-titan-dark\"",
            &format!("class=\"{}\"", theme_class),
        );
        let state_json = serde_json::to_string(&serde_json::json!({
            "theme": self.settings.theme,
            "accent_color": self.settings.accent_color,
            "search_engine": self.settings.search_engine,
        }))
        .unwrap_or_else(|_| "{}".into())
        .replace('<', "\\u003c");

        html_themed.replace(
            "<script src=\"newtab.js\"></script>",
            &format!(
                "<script id=\"titan-newtab-state\" type=\"application/json\">{}</script><script>{}</script>",
                state_json, js
            ),
        )
    }

    pub fn get_history_html(&self) -> String {
        let html = include_str!("../ui/history.html");
        let js = include_str!("../ui/dist/history.js");
        let theme_class = format!("theme-{}", self.settings.theme);
        let html_themed = html.replace(
            "class=\"theme-titan-dark\"",
            &format!("class=\"{}\"", theme_class),
        );
        let state_json = serde_json::to_string(&self.history)
            .unwrap_or_else(|_| "[]".into())
            .replace('<', "\\u003c");

        html_themed.replace(
            "<script src=\"history.js\"></script>",
            &format!(
                "<script id=\"titan-history-state\" type=\"application/json\">{}</script><script>{}</script>",
                state_json, js
            ),
        )
    }

    pub fn get_downloads_html(&self) -> String {
        let html = include_str!("../ui/downloads.html");
        let js = include_str!("../ui/dist/downloads.js");
        let theme_class = format!("theme-{}", self.settings.theme);
        let html_themed = html.replace(
            "class=\"theme-titan-dark\"",
            &format!("class=\"{}\"", theme_class),
        );
        let state_json = serde_json::to_string(&self.downloads)
            .unwrap_or_else(|_| "[]".into())
            .replace('<', "\\u003c");

        html_themed.replace(
            "<script src=\"downloads.js\"></script>",
            &format!(
                "<script id=\"titan-downloads-state\" type=\"application/json\">{}</script><script>{}</script>",
                state_json, js
            ),
        )
    }

    pub fn is_newtab_url(url: &str) -> bool {
        url == "titan://newtab"
            || url == "about:newtab"
            || url == "titan://home"
            || url == "about:home"
            || url == "about:blank"
    }

    pub fn is_settings_url(url: &str) -> bool {
        url == "titan://settings"
            || url == "titan://themes"
            || url == "titan://privacy"
            || url == "titan://adblock"
            || url == "titan://shields"
            || url == "titan://modules"
            || url == "titan://extensions"
            || url == "titan://darkmode"
            || url == "about:settings"
            || url == "about:themes"
            || url == "about:privacy"
            || url == "about:adblock"
            || url == "about:shields"
            || url == "about:extensions"
    }

    pub fn is_history_url(url: &str) -> bool {
        url == "titan://history" || url == "about:history"
    }

    pub fn is_downloads_url(url: &str) -> bool {
        url == "titan://downloads" || url == "about:downloads"
    }

    pub fn is_internal_url(url: &str) -> bool {
        Self::is_settings_url(url)
            || Self::is_history_url(url)
            || Self::is_downloads_url(url)
            || Self::is_newtab_url(url)
            || url.starts_with("titan://")
            || url.starts_with("about:")
    }

    pub fn get_current_window_size(&self) -> (f64, f64) {
        let scale = self.window.scale_factor();
        let logical = self.window.inner_size().to_logical::<f64>(scale);
        if logical.width > 10.0 && logical.height > 10.0 {
            (logical.width, logical.height)
        } else {
            self.window_size
        }
    }

    fn get_content_bounds(&self) -> Rect {
        let (width, height) = self.get_current_window_size();
        let header_height = self.get_header_height();
        let content_height = (height - header_height).max(10.0);
        Rect {
            position: LogicalPosition::new(0.0, header_height).into(),
            size: LogicalSize::new(width, content_height).into(),
        }
    }

    pub fn is_module_enabled(&self, id: &str) -> bool {
        self.modules
            .iter()
            .find(|m| m.id == id)
            .map(|m| m.enabled)
            .unwrap_or(false)
    }

    pub fn get_theme_background_color(&self) -> (u8, u8, u8, u8) {
        match self.settings.theme.as_str() {
            "titan-light" => (241, 245, 249, 255),
            "midnight" => (0, 0, 0, 255),
            "cyber-neon" => (13, 11, 24, 255),
            "nordic" => (15, 23, 28, 255),
            "amber" => (20, 18, 16, 255),
            _ => (15, 16, 21, 255),
        }
    }

    pub fn get_theme_injection_script(&self) -> String {
        render_typescript(
            include_str!("../web-scripts/dist/desktop-theme.js"),
            "__TITAN_DESKTOP_THEME_CONFIG__",
            serde_json::json!({
                "isLight": self.settings.theme == "titan-light",
                "forceAdaptation": self.is_module_enabled("dark_reader"),
            }),
        )
    }

    pub fn get_privacy_injection_script(&self) -> String {
        render_typescript(
            include_str!("../web-scripts/dist/desktop-privacy.js"),
            "__TITAN_DESKTOP_PRIVACY_CONFIG__",
            serde_json::json!({
                "doNotTrack": self.settings.do_not_track,
                "globalPrivacyControl": self.settings.global_privacy_control,
                "blockWebRtc": self.settings.block_webrtc_leak,
                "blockFingerprinting": self.settings.block_fingerprinting,
                "blockHyperlinkAuditing": self.settings.block_hyperlink_auditing,
                "telemetryDisabled": true,
                "mandatoryDomains": crate::privacy::BLOCKED_TELEMETRY_DOMAINS,
                "blockedDomains": self.settings.blocked_domains,
                "whitelistedDomains": self.settings.whitelisted_domains,
            }),
        )
    }

    pub fn get_adblock_injection_script(&self, target_url: &str) -> String {
        desktop_adblock_script(&self.adblock_manager, &self.settings, target_url)
    }

    pub fn get_adblock_dynamic_evaluation_script(&self, url: &str) -> String {
        let (hide_selectors, scriptlet) = self.adblock_manager.get_cosmetic_resources(url);
        if hide_selectors.is_empty() && scriptlet.is_empty() {
            return String::new();
        }

        render_typescript(
            include_str!("../web-scripts/dist/desktop-dynamic-adblock.js"),
            "__TITAN_DESKTOP_DYNAMIC_ADBLOCK_CONFIG__",
            serde_json::json!({
                "css": hide_selectors.join(",\n"),
                "scriptlet": scriptlet,
            }),
        )
    }

    pub fn update_all_tabs_theme(&self) {
        let script = self.get_theme_injection_script();
        for tab in &self.tabs {
            if !Self::is_internal_url(&tab.url) {
                let _ = tab.webview.evaluate_script(&script);
            }
        }
    }

    pub fn update_all_tabs_privacy(&self) {
        let script = self.get_privacy_injection_script();
        for tab in &self.tabs {
            if !Self::is_internal_url(&tab.url) {
                let _ = tab.webview.evaluate_script(&script);
            }
        }
    }

    #[cfg(target_os = "windows")]
    fn sync_desktop_adblock_settings(&self) {
        crate::desktop_adblock::update_shared_settings(
            &self.desktop_adblock_settings,
            &self.settings,
        );
    }

    fn build_tab(&mut self, target_url: &str) -> Tab {
        self.build_tab_with_mode(target_url, false)
    }

    fn build_tab_with_mode(&mut self, target_url: &str, is_private: bool) -> Tab {
        let tab_id = self.next_tab_id.get();
        self.next_tab_id.set(tab_id.saturating_add(1));

        let is_newtab = Self::is_newtab_url(target_url);
        let is_settings = Self::is_settings_url(target_url);
        let is_history = Self::is_history_url(target_url);
        let is_downloads = Self::is_downloads_url(target_url);
        let is_themes = target_url == "titan://themes" || target_url == "about:themes";
        let is_privacy = target_url == "titan://privacy" || target_url == "about:privacy";
        let is_adblock = target_url == "titan://adblock"
            || target_url == "about:adblock"
            || target_url == "titan://shields"
            || target_url == "about:shields";
        let is_extensions = target_url == "titan://extensions" || target_url == "about:extensions";

        let normalized_url = if is_newtab {
            "titan://newtab".to_string()
        } else if is_settings {
            "titan://settings".to_string()
        } else if is_history {
            "titan://history".to_string()
        } else if is_downloads {
            "titan://downloads".to_string()
        } else {
            let clean_url = if self.settings.strip_tracking_parameters {
                crate::url_utils::strip_tracking_parameters(target_url)
            } else {
                target_url.to_string()
            };
            normalize_or_search_url_with_engine(&clean_url, &self.settings.search_engine)
        };

        let proxy_ipc = self.proxy.clone();
        let proxy_load = self.proxy.clone();
        let proxy_download_started = self.proxy.clone();
        let proxy_download_completed = self.proxy.clone();
        let tab_id_copy = tab_id;
        let is_internal = is_newtab || is_settings || is_history || is_downloads;
        let theme_script = self.get_theme_injection_script();
        let privacy_script = self.get_privacy_injection_script();
        let adblock_script = self.get_adblock_injection_script(&normalized_url);
        let extensions_script = desktop_extensions_script(&self.extensions);
        let popup_theme_script = theme_script.clone();
        let popup_privacy_script = privacy_script.clone();
        let popup_settings = self.settings.clone();
        let popup_adblock_manager = self.adblock_manager.clone();
        let popup_extensions = self.extensions.clone();
        let popup_native_adblock_settings = self.desktop_adblock_settings.clone();
        let popup_tabs = self.pending_popup_tabs.clone();
        let popup_next_tab_id = self.next_tab_id.clone();
        let popup_window = self.window.clone();
        let popup_proxy = self.proxy.clone();

        let tab_state_script = render_typescript(
            include_str!("../web-scripts/dist/desktop-tab-state.js"),
            "__TITAN_DESKTOP_TAB_STATE_CONFIG__",
            serde_json::json!({ "tabId": tab_id }),
        );
        let init_script = [
            tab_state_script,
            theme_script,
            privacy_script,
            adblock_script,
            extensions_script,
        ]
        .join("\n");

        let content_bounds = self.get_content_bounds();
        let bg_color = self.get_theme_background_color();

        // Tab activation owns focus; a hidden tab must not request it at creation.
        #[allow(unused_mut)]
        let mut builder = WebViewBuilder::new();
        #[cfg(target_os = "windows")]
        {
            builder = builder.with_browser_extensions_enabled(true);
        }
        let builder = builder
            .with_incognito(is_private)
            .with_bounds(content_bounds)
            .with_background_color(bg_color)
            .with_transparent(false)
            .with_visible(false)
            .with_focused(false)
            .with_initialization_script(&init_script)
            .with_ipc_handler(move |req| {
                let message = req.body();
                if is_internal || is_allowed_content_ipc(message, tab_id_copy) {
                    let _ = proxy_ipc.send_event(UserEvent::Ipc(message.clone()));
                }
            })
            .with_new_window_req_handler(move |url, features| {
                let requested_url = url.trim();
                if !requested_url.is_empty()
                    && !requested_url.eq_ignore_ascii_case("about:blank")
                    && popup_target_url(requested_url).is_none()
                {
                    return NewWindowResponse::Deny;
                }

                let popup_tab_id = popup_next_tab_id.get();
                popup_next_tab_id.set(popup_tab_id.saturating_add(1));
                let popup_url = if requested_url.is_empty() {
                    "about:blank".to_string()
                } else {
                    requested_url.to_string()
                };
                let popup_tab_state_script = render_typescript(
                    include_str!("../web-scripts/dist/desktop-tab-state.js"),
                    "__TITAN_DESKTOP_TAB_STATE_CONFIG__",
                    serde_json::json!({ "tabId": popup_tab_id }),
                );
                let popup_adblock_script =
                    desktop_adblock_script(&popup_adblock_manager, &popup_settings, &popup_url);
                let popup_extensions_script = desktop_extensions_script(&popup_extensions);
                let popup_init_script = [
                    popup_tab_state_script,
                    popup_theme_script.clone(),
                    popup_privacy_script.clone(),
                    popup_adblock_script,
                    popup_extensions_script,
                ]
                .join("\n");

                let ipc_proxy = popup_proxy.clone();
                let load_proxy = popup_proxy.clone();
                let nested_popup_proxy = popup_proxy.clone();
                let download_started_proxy = popup_proxy.clone();
                let download_completed_proxy = popup_proxy.clone();
                #[allow(unused_mut)]
                let mut popup_builder = WebViewBuilder::new();
                #[cfg(target_os = "windows")]
                {
                    popup_builder = popup_builder.with_browser_extensions_enabled(true);
                }
                let popup_webview = popup_builder
                    .with_incognito(is_private)
                    .with_environment(features.opener.environment)
                    .with_bounds(content_bounds)
                    .with_background_color(bg_color)
                    .with_transparent(false)
                    .with_visible(false)
                    .with_focused(false)
                    .with_initialization_script(&popup_init_script)
                        .with_ipc_handler(move |request| {
                            let message = request.body();
                            if is_allowed_content_ipc(message, popup_tab_id) {
                                let _ = ipc_proxy.send_event(UserEvent::Ipc(message.clone()));
                            }
                        })
                        .with_new_window_req_handler(move |nested_url, _| {
                            let _ = nested_popup_proxy
                                .send_event(UserEvent::OpenPopup { url: nested_url });
                            NewWindowResponse::Deny
                        })
                        .with_download_started_handler(move |url, path| {
                            let path = prepare_download_destination(&url, path);
                            let _ = download_started_proxy
                                .send_event(UserEvent::DownloadStarted { url, path });
                            true
                        })
                        .with_download_completed_handler(move |url, path, success| {
                            let _ = download_completed_proxy
                                .send_event(UserEvent::DownloadCompleted { url, path, success });
                        })
                        .with_on_page_load_handler(move |event, url| match event {
                            PageLoadEvent::Started => {
                                let _ = load_proxy.send_event(UserEvent::PageLoadStarted {
                                    tab_id: popup_tab_id,
                                    url: url.to_string(),
                                });
                            }
                            PageLoadEvent::Finished => {
                                let _ = load_proxy.send_event(UserEvent::PageLoadFinished {
                                    tab_id: popup_tab_id,
                                    url: url.to_string(),
                                });
                            }
                        })
                        .build_as_child(&*popup_window);

                let popup_webview = match popup_webview {
                    Ok(webview) => webview,
                    Err(error) => {
                        eprintln!("Could not create popup WebView: {error}");
                        return NewWindowResponse::Deny;
                    }
                };
                if let Err(error) = crate::desktop_adblock::attach_request_blocker(
                    &popup_webview,
                    popup_adblock_manager.clone(),
                    popup_native_adblock_settings.clone(),
                    popup_proxy.clone(),
                ) {
                    eprintln!("Could not attach popup request blocker: {error}");
                }
                let core_webview = popup_webview.webview();
                popup_tabs.borrow_mut().push(Tab {
                    id: popup_tab_id,
                    url: popup_url,
                    title: "Popup".into(),
                    is_loading: true,
                    can_go_back: false,
                    can_go_forward: false,
                    is_private,
                    webview: popup_webview,
                });
                let _ = popup_proxy.send_event(UserEvent::AdoptPopup {
                    tab_id: popup_tab_id,
                });
                NewWindowResponse::Create {
                    webview: core_webview,
                }
            })
            .with_download_started_handler(move |url, path| {
                let path = prepare_download_destination(&url, path);
                let _ = proxy_download_started.send_event(UserEvent::DownloadStarted { url, path });
                true
            })
            .with_download_completed_handler(move |url, path, success| {
                let _ = proxy_download_completed.send_event(UserEvent::DownloadCompleted {
                    url,
                    path,
                    success,
                });
            })
            .with_on_page_load_handler(move |event, url| match event {
                PageLoadEvent::Started => {
                    let _ = proxy_load.send_event(UserEvent::PageLoadStarted {
                        tab_id: tab_id_copy,
                        url: url.to_string(),
                    });
                }
                PageLoadEvent::Finished => {
                    let _ = proxy_load.send_event(UserEvent::PageLoadFinished {
                        tab_id: tab_id_copy,
                        url: url.to_string(),
                    });
                }
            });

        let internal_html = if is_newtab {
            Some(self.get_newtab_html())
        } else if is_settings {
            let section = if is_themes {
                "themes"
            } else if is_privacy {
                "privacy"
            } else if is_adblock {
                "adblock"
            } else if is_extensions {
                "extensions"
            } else {
                "general"
            };
            Some(self.get_settings_html(section))
        } else if is_history {
            Some(self.get_history_html())
        } else if is_downloads {
            Some(self.get_downloads_html())
        } else {
            None
        };

        let webview = if let Some(ref html) = internal_html {
            builder
                .with_html(html)
                .build_as_child(&*self.window)
                .expect("Failed to create content webview for tab")
        } else {
            let webview = builder
                .build_as_child(&*self.window)
                .expect("Failed to create content webview for tab");

            #[cfg(target_os = "windows")]
            if let Err(error) = crate::desktop_adblock::attach_request_blocker(
                &webview,
                self.adblock_manager.clone(),
                self.desktop_adblock_settings.clone(),
                self.proxy.clone(),
            ) {
                eprintln!("Failed to attach native adblock request handler: {error}");
            }

            webview
                .load_url(&normalized_url)
                .expect("Failed to load content URL");
            webview
        };

        let default_title = if is_newtab {
            "New Tab".to_string()
        } else if is_history {
            "History".to_string()
        } else if is_downloads {
            "Downloads".to_string()
        } else if is_themes {
            "Themes".to_string()
        } else if is_privacy {
            "Privacy & Security".to_string()
        } else if is_adblock {
            "AdBlock & Shields".to_string()
        } else if is_settings {
            "Settings".to_string()
        } else if normalized_url.contains("youtube.com") {
            "YouTube".to_string()
        } else {
            "New Tab".to_string()
        };

        Tab {
            id: tab_id,
            url: normalized_url,
            title: default_title,
            is_loading: !is_internal,
            can_go_back: false,
            can_go_forward: false,
            is_private,
            webview,
        }
    }

    fn get_parked_content_bounds(&self) -> Rect {
        let (width, height) = self.get_current_window_size();
        let content_height = (height - self.get_header_height()).max(10.0);
        Rect {
            position: LogicalPosition::new(0.0, height + 1.0).into(),
            size: LogicalSize::new(width, content_height).into(),
        }
    }

    pub fn create_tab(&mut self, target_url: &str) -> u32 {
        let new_tab = self.build_tab(target_url);
        let tab_id = new_tab.id;
        self.tabs.push(new_tab);
        self.switch_tab(tab_id);
        self.save_session();
        tab_id
    }

    pub fn create_private_tab(&mut self) -> u32 {
        let new_tab = self.build_tab_with_mode("titan://newtab", true);
        let tab_id = new_tab.id;
        self.tabs.push(new_tab);
        self.switch_tab(tab_id);
        tab_id
    }

    fn restore_session(&mut self) {
        let session = self.storage.load_session();
        self.is_restoring_session = true;
        let targets: Vec<String> = session
            .tabs
            .iter()
            .take(25)
            .filter_map(|tab| Self::restorable_url(&tab.url))
            .collect();

        for target in &targets {
            self.create_tab(target);
        }
        if self.tabs.is_empty() {
            self.create_tab("titan://newtab");
        } else {
            let active_index = session.active_index.min(self.tabs.len() - 1);
            let active_id = self.tabs[active_index].id;
            self.switch_tab(active_id);
        }
        self.is_restoring_session = false;
        self.save_session();
    }

    fn restorable_url(url: &str) -> Option<String> {
        if Self::is_internal_url(url) {
            return Some(if Self::is_newtab_url(url) {
                "titan://newtab".into()
            } else if Self::is_history_url(url) {
                "titan://history".into()
            } else if Self::is_downloads_url(url) {
                "titan://downloads".into()
            } else {
                "titan://settings".into()
            });
        }
        popup_target_url(url)
    }

    fn save_session(&self) {
        if self.is_restoring_session {
            return;
        }
        let regular_tabs: Vec<&Tab> = self.tabs.iter().filter(|tab| !tab.is_private).collect();
        let active_index = self
            .active_tab_id
            .and_then(|id| regular_tabs.iter().position(|tab| tab.id == id))
            .unwrap_or(0);
        let session = BrowserSession {
            tabs: regular_tabs
                .iter()
                .map(|tab| SessionTab {
                    url: tab.url.clone(),
                    title: tab.title.clone(),
                })
                .collect(),
            active_index,
        };
        self.storage.save_session(&session);
    }

    fn record_history(&mut self, title: &str, url: &str) {
        if Self::is_internal_url(url) || popup_target_url(url).is_none() {
            return;
        }
        let previous = self
            .history
            .iter()
            .position(|entry| entry.url == url)
            .map(|index| self.history.remove(index));
        let visit_count = previous
            .map(|entry| entry.visit_count.saturating_add(1))
            .unwrap_or(1);
        let last_visited_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|duration| duration.as_millis() as u64)
            .unwrap_or(0);
        self.history.insert(
            0,
            HistoryEntry {
                title: if title.trim().is_empty() { url } else { title }.into(),
                url: url.into(),
                last_visited_ms,
                visit_count,
            },
        );
        self.history.truncate(2_000);
        self.storage.save_history(&self.history);
    }

    fn open_history(&mut self) {
        if let Some(tab_id) = self
            .tabs
            .iter()
            .find(|tab| Self::is_history_url(&tab.url))
            .map(|tab| tab.id)
        {
            let html = self.get_history_html();
            if let Some(tab) = self.tabs.iter().find(|tab| tab.id == tab_id) {
                let _ = tab.webview.load_html(&html);
            }
            self.switch_tab(tab_id);
        } else {
            self.create_tab("titan://history");
        }
    }

    fn clear_history(&mut self) {
        self.history.clear();
        self.storage.save_history(&self.history);
        let html = self.get_history_html();
        for tab in &self.tabs {
            if Self::is_history_url(&tab.url) {
                let _ = tab.webview.load_html(&html);
            }
        }
    }

    fn open_downloads(&mut self) {
        if let Some(tab_id) = self
            .tabs
            .iter()
            .find(|tab| Self::is_downloads_url(&tab.url))
            .map(|tab| tab.id)
        {
            self.refresh_download_pages();
            self.switch_tab(tab_id);
        } else {
            self.create_tab("titan://downloads");
        }
    }

    fn refresh_download_pages(&self) {
        let html = self.get_downloads_html();
        for tab in &self.tabs {
            if Self::is_downloads_url(&tab.url) {
                let _ = tab.webview.load_html(&html);
            }
        }
    }

    pub fn on_download_started(&mut self, url: String, path: Option<std::path::PathBuf>) {
        let started_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|duration| duration.as_millis() as u64)
            .unwrap_or(0);
        let record = DownloadRecord {
            id: self.next_download_id,
            url,
            file_path: path.map(|path| path.to_string_lossy().into_owned()),
            status: "downloading".into(),
            started_ms,
        };
        self.next_download_id = self.next_download_id.saturating_add(1);
        self.downloads.insert(0, record);
        self.downloads.truncate(500);
        self.storage.save_downloads(&self.downloads);
        self.refresh_download_pages();
    }

    pub fn on_download_completed(
        &mut self,
        url: String,
        path: Option<std::path::PathBuf>,
        success: bool,
    ) {
        if let Some(download) = self
            .downloads
            .iter_mut()
            .find(|download| download.url == url && download.status == "downloading")
        {
            if let Some(path) = path {
                download.file_path = Some(path.to_string_lossy().into_owned());
            }
            download.status = if success { "complete" } else { "failed" }.into();
        }
        self.storage.save_downloads(&self.downloads);
        self.refresh_download_pages();
    }

    fn clear_downloads(&mut self) {
        self.downloads.clear();
        self.storage.save_downloads(&self.downloads);
        self.refresh_download_pages();
    }

    fn open_download(&self, download_id: u64) {
        let Some(path) = self
            .downloads
            .iter()
            .find(|download| download.id == download_id && download.status == "complete")
            .and_then(|download| download.file_path.as_deref())
        else {
            return;
        };
        let path = std::path::Path::new(path);
        if !path.is_file() {
            return;
        }

        #[cfg(target_os = "windows")]
        if !crate::menu_util::open_file(path) {
            eprintln!("Could not open downloaded file {}", path.display());
        }
    }

    pub fn open_popup(&mut self, requested_url: &str) {
        if let Some(target_url) = popup_target_url(requested_url) {
            self.create_tab(&target_url);
        }
    }

    pub fn adopt_popup(&mut self, tab_id: u32) {
        let pending = {
            let mut popup_tabs = self.pending_popup_tabs.borrow_mut();
            popup_tabs
                .iter()
                .position(|tab| tab.id == tab_id)
                .map(|index| popup_tabs.remove(index))
        };
        if let Some(tab) = pending {
            self.tabs.push(tab);
            self.switch_tab(tab_id);
            self.save_session();
        }
    }

    pub fn open_external_url(&mut self, requested_url: &str) {
        let Some(target_url) = popup_target_url(requested_url) else {
            return;
        };
        let active_is_newtab = self
            .active_tab_id
            .and_then(|id| self.tabs.iter().find(|tab| tab.id == id))
            .is_some_and(|tab| Self::is_newtab_url(&tab.url));
        if active_is_newtab {
            self.navigate_active_tab(&target_url);
        } else {
            self.create_tab(&target_url);
        }
    }

    fn focus_address_bar(&self) {
        let script = desktop_command(serde_json::json!({ "type": "focusAddressBar" }));
        if let Some(header) = &self.header_webview {
            let _ = header.focus();
            let _ = header.evaluate_script(&script);
        }
    }

    fn prewarm_settings_tab(&mut self) {
        if self.prewarmed_settings_tab.is_none()
            && !self.tabs.iter().any(|tab| Self::is_settings_url(&tab.url))
        {
            let tab = self.build_tab("titan://settings");
            let _ = tab.webview.set_bounds(self.get_parked_content_bounds());
            let _ = tab.webview.set_visible(true);
            self.prewarmed_settings_tab = Some(tab);
        }
    }

    fn open_settings_view(&mut self, view: &str) {
        let target_id =
            if let Some(tab) = self.tabs.iter().find(|tab| Self::is_settings_url(&tab.url)) {
                tab.id
            } else if let Some(tab) = self.prewarmed_settings_tab.take() {
                let tab_id = tab.id;
                self.tabs.push(tab);
                tab_id
            } else {
                let target_url = match view {
                    "themes" => "titan://themes",
                    "privacy" => "titan://privacy",
                    "adblock" => "titan://adblock",
                    "extensions" => "titan://extensions",
                    _ => "titan://settings",
                };
                self.create_tab(target_url);
                return;
            };

        if let Some(tab) = self.tabs.iter_mut().find(|tab| tab.id == target_id) {
            tab.title = match view {
                "themes" => "Themes",
                "privacy" => "Privacy & Security",
                "adblock" => "AdBlock & Shields",
                "extensions" => "Extensions & Add-ons",
                _ => "Settings",
            }
            .into();
        }

        self.switch_tab(target_id);
        self.sync_settings_tabs();
        let script = desktop_command(serde_json::json!({
            "type": "switchSettingsView",
            "view": view,
        }));
        if let Some(tab) = self.tabs.iter().find(|tab| tab.id == target_id) {
            let _ = tab.webview.evaluate_script(&script);
        }
    }

    pub fn switch_tab(&mut self, target_id: u32) {
        let content_bounds = self.get_content_bounds();

        // 1. Hide all other tabs first so their deactivation does not steal focus or block hit testing
        for tab in &mut self.tabs {
            if tab.id != target_id {
                let _ = tab.webview.set_visible(false);
            }
        }

        // 2. Show and reposition the target active tab
        if let Some(active_tab) = self.tabs.iter_mut().find(|t| t.id == target_id) {
            let _ = active_tab.webview.set_bounds(content_bounds);
            let _ = active_tab.webview.set_visible(true);
            if let Ok(current_url) = active_tab.webview.url() {
                if !current_url.is_empty() && current_url != "about:blank" {
                    active_tab.url = current_url;
                }
            }
        }

        self.active_tab_id = Some(target_id);
        self.sync_full_state();

        // 3. Focus the active tab after state synchronization to ensure it receives user interaction
        if let Some(active_tab) = self.tabs.iter_mut().find(|t| t.id == target_id) {
            let _ = active_tab.webview.focus();
        }
        self.save_session();
    }

    pub fn close_tab(&mut self, target_id: u32) {
        if let Some(pos) = self.tabs.iter().position(|t| t.id == target_id) {
            let was_active = self.active_tab_id == Some(target_id);
            let closed_tab = self.tabs.remove(pos);
            if Self::is_settings_url(&closed_tab.url) {
                let _ = closed_tab
                    .webview
                    .set_bounds(self.get_parked_content_bounds());
                let _ = closed_tab.webview.set_visible(true);
                self.prewarmed_settings_tab = Some(closed_tab);
            }

            if self.tabs.is_empty() {
                // If all tabs closed, create a fresh New Tab
                self.create_tab("titan://newtab");
            } else if was_active {
                let new_idx = if pos >= self.tabs.len() {
                    self.tabs.len() - 1
                } else {
                    pos
                };
                let next_id = self.tabs[new_idx].id;
                self.switch_tab(next_id);
            } else {
                self.sync_full_state();
            }
            self.save_session();
        }
    }

    pub fn navigate_active_tab(&mut self, input: &str) {
        if Self::is_newtab_url(input) {
            let html = self.get_newtab_html();
            if let Some(active_id) = self.active_tab_id {
                if let Some(tab) = self.tabs.iter_mut().find(|t| t.id == active_id) {
                    tab.url = "titan://newtab".into();
                    tab.title = "New Tab".into();
                    tab.is_loading = false;
                    let _ = tab.webview.load_html(&html);
                }
                self.sync_tab_update(active_id);
                if let Some(tab) = self.tabs.iter().find(|t| t.id == active_id) {
                    let _ = tab.webview.focus();
                }
                self.save_session();
            }
            return;
        }

        if Self::is_settings_url(input) {
            let is_themes = input == "titan://themes" || input == "about:themes";
            let is_privacy = input == "titan://privacy" || input == "about:privacy";
            let is_adblock = input == "titan://adblock"
                || input == "about:adblock"
                || input == "titan://shields"
                || input == "about:shields";
            let section = if is_themes {
                "themes"
            } else if is_privacy {
                "privacy"
            } else if is_adblock {
                "adblock"
            } else {
                "general"
            };
            let html = self.get_settings_html(section);
            if let Some(active_id) = self.active_tab_id {
                if let Some(tab) = self.tabs.iter_mut().find(|t| t.id == active_id) {
                    tab.url = "titan://settings".into();
                    tab.title = if is_themes {
                        "Themes".into()
                    } else if is_privacy {
                        "Privacy & Security".into()
                    } else if is_adblock {
                        "AdBlock & Shields".into()
                    } else {
                        "Settings".into()
                    };
                    tab.is_loading = false;
                    let _ = tab.webview.load_html(&html);
                }
                self.sync_tab_update(active_id);
                if let Some(tab) = self.tabs.iter().find(|t| t.id == active_id) {
                    let _ = tab.webview.focus();
                }
                self.save_session();
            }
            return;
        }

        if Self::is_history_url(input) {
            self.open_history();
            return;
        }

        if Self::is_downloads_url(input) {
            self.open_downloads();
            return;
        }

        let clean_input = if self.settings.strip_tracking_parameters {
            crate::url_utils::strip_tracking_parameters(input)
        } else {
            input.to_string()
        };

        let normalized =
            normalize_or_search_url_with_engine(&clean_input, &self.settings.search_engine);
        if let Some(active_id) = self.active_tab_id {
            if let Some(tab) = self.tabs.iter_mut().find(|t| t.id == active_id) {
                tab.url = normalized.clone();
                tab.is_loading = true;
                let _ = tab.webview.load_url(&normalized);
            }
            self.sync_tab_update(active_id);
            if let Some(tab) = self.tabs.iter().find(|t| t.id == active_id) {
                let _ = tab.webview.focus();
            }
            self.save_session();
        }
    }

    pub fn go_back(&mut self) {
        let script = desktop_command(serde_json::json!({ "type": "goBack" }));
        if let Some(active_id) = self.active_tab_id {
            if let Some(tab) = self.tabs.iter_mut().find(|t| t.id == active_id) {
                let _ = tab.webview.evaluate_script(&script);
            }
            if let Some(tab) = self.tabs.iter().find(|t| t.id == active_id) {
                let _ = tab.webview.focus();
            }
        }
    }

    pub fn go_forward(&mut self) {
        let script = desktop_command(serde_json::json!({ "type": "goForward" }));
        if let Some(active_id) = self.active_tab_id {
            if let Some(tab) = self.tabs.iter_mut().find(|t| t.id == active_id) {
                let _ = tab.webview.evaluate_script(&script);
            }
            if let Some(tab) = self.tabs.iter().find(|t| t.id == active_id) {
                let _ = tab.webview.focus();
            }
        }
    }

    pub fn reload(&mut self) {
        if let Some(active_id) = self.active_tab_id {
            let tab_url = self
                .tabs
                .iter()
                .find(|t| t.id == active_id)
                .map(|t| t.url.clone())
                .unwrap_or_default();

            if Self::is_newtab_url(&tab_url) {
                let html = self.get_newtab_html();
                if let Some(tab) = self.tabs.iter_mut().find(|t| t.id == active_id) {
                    let _ = tab.webview.load_html(&html);
                }
            } else if Self::is_settings_url(&tab_url) {
                let html = self.get_settings_html("general");
                if let Some(tab) = self.tabs.iter_mut().find(|t| t.id == active_id) {
                    let _ = tab.webview.load_html(&html);
                }
            } else {
                let script = desktop_command(serde_json::json!({ "type": "reload" }));
                if let Some(tab) = self.tabs.iter_mut().find(|t| t.id == active_id) {
                    let _ = tab.webview.evaluate_script(&script);
                }
            }
            if let Some(tab) = self.tabs.iter().find(|t| t.id == active_id) {
                let _ = tab.webview.focus();
            }
        }
    }

    pub fn go_home(&mut self) {
        self.navigate_active_tab("titan://newtab");
    }

    pub fn set_zoom(&mut self, zoom: f64) {
        self.zoom = zoom.clamp(0.4, 3.0);
        let zoom_val = self.zoom;
        let script = desktop_command(serde_json::json!({ "type": "setZoom", "zoom": zoom_val }));
        for tab in &self.tabs {
            let _ = tab.webview.evaluate_script(&script);
        }
        self.sync_full_state();
    }

    pub fn toggle_bookmark(&mut self, title: String, url: String) {
        if let Some(pos) = self.bookmarks.iter().position(|b| b.url == url) {
            self.bookmarks.remove(pos);
        } else {
            self.bookmarks.push(Bookmark { title, url });
        }
        self.storage.save_bookmarks(&self.bookmarks);
        let (w, h) = self.window_size;
        self.resize(w, h);
        self.sync_full_state();
    }

    pub fn remove_bookmark(&mut self, url: &str) {
        self.bookmarks.retain(|b| b.url != url);
        self.storage.save_bookmarks(&self.bookmarks);
        let (w, h) = self.window_size;
        self.resize(w, h);
        self.sync_full_state();
    }

    pub fn sync_settings_tabs(&self) {
        let script = desktop_command(serde_json::json!({
            "type": "initializeSettings",
            "state": {
                "settings": self.settings,
                "modules": self.modules,
                "blocked_logs": self.blocked_logs,
                "adblock_logs": self.adblock_logs,
                "adblock_filter_lists": self.adblock_manager.get_filter_lists_info(),
                "adblock_stats": self.adblock_manager.get_stats(),
                "adblock_custom_rules": self.adblock_manager.get_custom_rules(),
                "mandatory_blocked_domains": crate::privacy::BLOCKED_TELEMETRY_DOMAINS,
                "update_state": self.update_state,
                "extensions": &self.extensions,
            },
        }));
        for tab in &self.tabs {
            if Self::is_settings_url(&tab.url) {
                let _ = tab.webview.evaluate_script(&script);
            }
        }
    }

    pub fn toggle_module(&mut self, module_id: &str, enabled: bool) {
        if let Some(module) = self.modules.iter_mut().find(|m| m.id == module_id) {
            module.enabled = enabled;
        }
        self.storage.save_modules(&self.modules);

        // Re-sync any open settings tabs
        self.sync_settings_tabs();

        // Apply dark mode or light mode dynamically across all open web content tabs
        self.update_all_tabs_theme();

        self.sync_full_state();
    }

    pub fn check_for_updates(&mut self) {
        self.update_state.status = UpdateStatus::Checking;
        self.update_state.message = "Checking for updates...".into();
        self.sync_settings_tabs();
        self.sync_full_state();

        let proxy = self.proxy.clone();
        let current_version = self.update_state.current_version.clone();
        thread::spawn(move || {
            let result = crate::updater::check_for_updates(&current_version);
            let _ = proxy.send_event(UserEvent::UpdateCheckFinished(result));
        });
    }

    pub fn on_update_check_finished(&mut self, result: UpdateCheckResult) {
        match result {
            UpdateCheckResult::Available(UpdateInfo {
                version,
                release_url,
            }) => {
                self.update_state.latest_version = Some(version.clone());
                self.update_state.release_url = Some(release_url);
                self.update_state.status = UpdateStatus::UpdateAvailable;
                self.update_state.message = format!("Version {version} is available.");
            }
            UpdateCheckResult::UpToDate(UpdateInfo {
                version,
                release_url,
            }) => {
                self.update_state.latest_version = Some(version.clone());
                self.update_state.release_url = Some(release_url);
                self.update_state.status = UpdateStatus::UpToDate;
                self.update_state.message = format!("Titan Browser is up to date ({version}).");
            }
            UpdateCheckResult::Failed(message) => {
                self.update_state.status = UpdateStatus::Error;
                self.update_state.message = message;
            }
        }

        self.sync_settings_tabs();
        self.sync_full_state();
    }

    pub fn open_update_download(&mut self) {
        let url = self.update_state.release_url.clone().unwrap_or_else(|| {
            "https://github.com/SketchyLucian/titan-browser/releases/latest".into()
        });
        self.create_tab(&url);
    }

    pub fn on_page_load_started(&mut self, tab_id: u32, url: String) {
        let mut target_url = url.clone();
        if let Some(tab) = self.tabs.iter_mut().find(|t| t.id == tab_id) {
            tab.is_loading = true;
            if !url.is_empty() && url != "about:blank" {
                tab.url = url.clone();
            }
            if target_url.is_empty() {
                target_url = tab.url.clone();
            }
        }

        // Early inject uBO cosmetic rules & scriptlets as soon as page load starts
        if !target_url.is_empty()
            && !Self::is_internal_url(&target_url)
            && self.settings.adblock_enabled
            && self.settings.adblock_cosmetic_filtering
        {
            let dynamic_adblock = self.get_adblock_dynamic_evaluation_script(&target_url);
            if !dynamic_adblock.is_empty() {
                if let Some(tab) = self.tabs.iter().find(|t| t.id == tab_id) {
                    let _ = tab.webview.evaluate_script(&dynamic_adblock);
                }
            }
        }

        self.sync_tab_update(tab_id);
    }

    pub fn on_page_load_finished(&mut self, tab_id: u32, url: String) {
        let is_dark_reader = self.is_module_enabled("dark_reader");
        let theme_script = if is_dark_reader {
            Some(self.get_theme_injection_script())
        } else {
            None
        };

        let mut final_url = String::new();
        if let Some(tab) = self.tabs.iter_mut().find(|t| t.id == tab_id) {
            tab.is_loading = false;
            if !url.is_empty() && url != "about:blank" {
                tab.url = url;
            }
            if let Ok(current_url) = tab.webview.url() {
                if !current_url.is_empty() && current_url != "about:blank" {
                    tab.url = current_url;
                }
            }
            final_url = tab.url.clone();

            // Inject matching theme script dynamically if dark reader is forced
            if !Self::is_internal_url(&tab.url) {
                if let Some(ref script) = theme_script {
                    let _ = tab.webview.evaluate_script(script);
                }
            }
        }

        // Inject dynamic uBlock Origin cosmetic rules & scriptlets for the loaded page URL
        if !final_url.is_empty()
            && !Self::is_internal_url(&final_url)
            && self.settings.adblock_enabled
            && self.settings.adblock_cosmetic_filtering
        {
            let dynamic_adblock = self.get_adblock_dynamic_evaluation_script(&final_url);
            if !dynamic_adblock.is_empty() {
                if let Some(tab) = self.tabs.iter().find(|t| t.id == tab_id) {
                    let _ = tab.webview.evaluate_script(&dynamic_adblock);
                }
            }
        }

        self.sync_tab_update(tab_id);
        self.save_session();

        if self.active_tab_id == Some(tab_id) {
            if let Some(tab) = self.tabs.iter().find(|t| t.id == tab_id) {
                let _ = tab.webview.focus();
            }
        }
    }

    pub fn update_tab_state(
        &mut self,
        tab_id: Option<u32>,
        url: String,
        title: String,
        can_go_back: Option<bool>,
        can_go_forward: Option<bool>,
    ) {
        let target_id = tab_id.or(self.active_tab_id);
        if let Some(id) = target_id {
            let mut history_record = None;
            if let Some(tab) = self.tabs.iter_mut().find(|t| t.id == id) {
                let actual_url = tab
                    .webview
                    .url()
                    .ok()
                    .filter(|actual_url| !actual_url.is_empty() && actual_url != "about:blank");
                if let Some(actual_url) = actual_url {
                    tab.url = actual_url;
                } else if Self::is_internal_url(&tab.url) && !url.is_empty() && url != "about:blank"
                {
                    tab.url = url;
                }
                if !title.is_empty() {
                    tab.title = title;
                }
                tab.can_go_back = tab
                    .webview
                    .can_go_back()
                    .ok()
                    .or(can_go_back)
                    .unwrap_or(false);
                tab.can_go_forward = tab
                    .webview
                    .can_go_forward()
                    .ok()
                    .or(can_go_forward)
                    .unwrap_or(false);
                if !tab.is_private {
                    history_record = Some((tab.title.clone(), tab.url.clone()));
                }
            }
            if let Some((history_title, history_url)) = history_record {
                self.record_history(&history_title, &history_url);
            }
            self.sync_tab_update(id);
            self.save_session();
        }
    }

    pub fn resize(&mut self, width: f64, height: f64) {
        self.window_size = (width, height);
        let header_height = self.get_header_height();

        let header_bounds = Rect {
            position: LogicalPosition::new(0.0, 0.0).into(),
            size: LogicalSize::new(width, header_height).into(),
        };

        if let Some(header) = &self.header_webview {
            let _ = header.set_bounds(header_bounds);
        }

        let content_bounds = self.get_content_bounds();
        let active_id = self.active_tab_id;
        for tab in &mut self.tabs {
            if Some(tab.id) == active_id {
                let _ = tab.webview.set_bounds(content_bounds);
                let _ = tab.webview.set_visible(true);
            } else {
                let _ = tab.webview.set_visible(false);
            }
        }

        let parked_bounds = self.get_parked_content_bounds();
        if let Some(tab) = &self.prewarmed_settings_tab {
            let _ = tab.webview.set_bounds(parked_bounds);
            let _ = tab.webview.set_visible(true);
        }
    }

    pub fn handle_incoming_ipc(&mut self, msg_str: &str) {
        if let Ok(msg) = serde_json::from_str::<IpcIncoming>(msg_str) {
            match msg {
                IpcIncoming::UiReady => {
                    self.sync_full_state();
                }
                IpcIncoming::NewTab { url } => {
                    let default_url = url.unwrap_or_else(|| "titan://newtab".into());
                    self.create_tab(&default_url);
                }
                IpcIncoming::NewPrivateTab => {
                    self.create_private_tab();
                }
                IpcIncoming::CloseTab { tab_id } => {
                    self.close_tab(tab_id);
                }
                IpcIncoming::SwitchTab { tab_id } => {
                    self.switch_tab(tab_id);
                }
                IpcIncoming::Navigate { url } => {
                    self.navigate_active_tab(&url);
                }
                IpcIncoming::GoBack => {
                    self.go_back();
                }
                IpcIncoming::GoForward => {
                    self.go_forward();
                }
                IpcIncoming::Reload => {
                    self.reload();
                }
                IpcIncoming::GoHome => {
                    self.go_home();
                }
                IpcIncoming::FocusAddressBar => {
                    self.focus_address_bar();
                }
                IpcIncoming::SetZoom { zoom } => {
                    self.set_zoom(zoom);
                }
                IpcIncoming::ToggleBookmark { title, url } => {
                    self.toggle_bookmark(title, url);
                }
                IpcIncoming::RemoveBookmark { url } => {
                    self.remove_bookmark(&url);
                }
                IpcIncoming::ToggleModule { module_id, enabled } => {
                    self.toggle_module(&module_id, enabled);
                }
                IpcIncoming::SetTheme { theme } => {
                    self.settings.theme = theme;
                    self.storage.save_settings(&self.settings);
                    #[cfg(target_os = "windows")]
                    {
                        let (r, g, b, _) = self.get_theme_background_color();
                        crate::drag_util::apply_dark_window_attributes(&self.window, (r, g, b));
                    }
                    self.sync_settings_tabs();
                    self.sync_full_state();
                    self.update_all_tabs_theme();
                }
                IpcIncoming::SetAccentColor { color } => {
                    self.settings.accent_color = color;
                    self.storage.save_settings(&self.settings);
                    self.sync_settings_tabs();
                    self.sync_full_state();
                }
                IpcIncoming::SetSearchEngine { engine } => {
                    self.settings.search_engine = engine;
                    self.storage.save_settings(&self.settings);
                    self.sync_full_state();
                }
                IpcIncoming::SetShowBookmarksBar { show } => {
                    self.settings.show_bookmarks_bar = show;
                    self.storage.save_settings(&self.settings);
                    let (w, h) = self.window_size;
                    self.resize(w, h);
                    self.sync_full_state();
                }
                IpcIncoming::SetPrivacySetting { key, enabled } => {
                    match key.as_str() {
                        "do_not_track" => self.settings.do_not_track = enabled,
                        "global_privacy_control" => self.settings.global_privacy_control = enabled,
                        "strip_tracking_parameters" => {
                            self.settings.strip_tracking_parameters = enabled
                        }
                        "block_webrtc_leak" => self.settings.block_webrtc_leak = enabled,
                        "block_fingerprinting" => self.settings.block_fingerprinting = enabled,
                        "block_hyperlink_auditing" => {
                            self.settings.block_hyperlink_auditing = enabled
                        }
                        "telemetry_disabled" => self.settings.telemetry_disabled = true,
                        _ => {}
                    }
                    self.storage.save_settings(&self.settings);
                    self.update_all_tabs_privacy();
                    self.sync_settings_tabs();
                    self.sync_full_state();
                }
                IpcIncoming::SetAdblockSetting { key, enabled } => {
                    match key.as_str() {
                        "adblock_enabled" => self.settings.adblock_enabled = enabled,
                        "adblock_block_video_ads" => {
                            self.settings.adblock_block_video_ads = enabled
                        }
                        "adblock_cosmetic_filtering" => {
                            self.settings.adblock_cosmetic_filtering = enabled
                        }
                        "adblock_block_popups" => self.settings.adblock_block_popups = enabled,
                        "adblock_aggressive_mode" => {
                            self.settings.adblock_aggressive_mode = enabled
                        }
                        _ => {}
                    }
                    #[cfg(target_os = "windows")]
                    self.sync_desktop_adblock_settings();
                    self.storage.save_settings(&self.settings);
                    self.sync_settings_tabs();
                    self.sync_full_state();
                }
                IpcIncoming::AddBlockedDomain { domain } => {
                    if let Some(d) = crate::privacy::normalize_domain_rule(&domain) {
                        if !self.settings.blocked_domains.contains(&d) {
                            self.settings.blocked_domains.push(d);
                            self.storage.save_settings(&self.settings);
                            self.sync_settings_tabs();
                        }
                    }
                }
                IpcIncoming::RemoveBlockedDomain { domain } => {
                    if let Some(d) = crate::privacy::normalize_domain_rule(&domain) {
                        if crate::privacy::BLOCKED_TELEMETRY_DOMAINS.contains(&d.as_str()) {
                            self.sync_settings_tabs();
                            return;
                        }
                        self.settings.blocked_domains.retain(|item| item != &d);
                        self.storage.save_settings(&self.settings);
                        self.sync_settings_tabs();
                    }
                }
                IpcIncoming::AddWhitelistedDomain { domain } => {
                    if let Some(d) = crate::privacy::normalize_domain_rule(&domain) {
                        if !self.settings.whitelisted_domains.contains(&d) {
                            self.settings.whitelisted_domains.push(d);
                            self.storage.save_settings(&self.settings);
                            self.sync_settings_tabs();
                        }
                    }
                }
                IpcIncoming::RemoveWhitelistedDomain { domain } => {
                    if let Some(d) = crate::privacy::normalize_domain_rule(&domain) {
                        self.settings.whitelisted_domains.retain(|item| item != &d);
                        self.storage.save_settings(&self.settings);
                        self.sync_settings_tabs();
                    }
                }
                IpcIncoming::ResetPrivacyRules => {
                    self.settings.blocked_domains = crate::ipc::default_blocked_domains();
                    self.settings.whitelisted_domains.clear();
                    self.storage.save_settings(&self.settings);
                    self.sync_settings_tabs();
                }
                IpcIncoming::AddAdblockDomain { domain } => {
                    if let Some(d) = crate::privacy::normalize_domain_rule(&domain) {
                        if !self.settings.adblock_blocked_domains.contains(&d) {
                            self.settings.adblock_blocked_domains.push(d);
                            #[cfg(target_os = "windows")]
                            self.sync_desktop_adblock_settings();
                            self.storage.save_settings(&self.settings);
                            self.sync_settings_tabs();
                        }
                    }
                }
                IpcIncoming::RemoveAdblockDomain { domain } => {
                    if let Some(d) = crate::privacy::normalize_domain_rule(&domain) {
                        self.settings
                            .adblock_blocked_domains
                            .retain(|item| item != &d);
                        #[cfg(target_os = "windows")]
                        self.sync_desktop_adblock_settings();
                        self.storage.save_settings(&self.settings);
                        self.sync_settings_tabs();
                    }
                }
                IpcIncoming::AddAdblockWhitelist { domain } => {
                    if let Some(d) = crate::privacy::normalize_domain_rule(&domain) {
                        if !self.settings.adblock_whitelisted_domains.contains(&d) {
                            self.settings.adblock_whitelisted_domains.push(d);
                            #[cfg(target_os = "windows")]
                            self.sync_desktop_adblock_settings();
                            self.storage.save_settings(&self.settings);
                            self.sync_settings_tabs();
                        }
                    }
                }
                IpcIncoming::RemoveAdblockWhitelist { domain } => {
                    if let Some(d) = crate::privacy::normalize_domain_rule(&domain) {
                        self.settings
                            .adblock_whitelisted_domains
                            .retain(|item| item != &d);
                        #[cfg(target_os = "windows")]
                        self.sync_desktop_adblock_settings();
                        self.storage.save_settings(&self.settings);
                        self.sync_settings_tabs();
                    }
                }
                IpcIncoming::ResetAdblockRules => {
                    self.settings.adblock_blocked_domains = crate::ipc::default_adblock_domains();
                    self.settings.adblock_whitelisted_domains.clear();
                    #[cfg(target_os = "windows")]
                    self.sync_desktop_adblock_settings();
                    self.storage.save_settings(&self.settings);
                    self.sync_settings_tabs();
                }
                IpcIncoming::ClearAdblockLogs => {
                    self.adblock_logs.clear();
                    self.sync_settings_tabs();
                }
                IpcIncoming::SetAutoUpdate { enabled } => {
                    self.settings.auto_update_enabled = enabled;
                    self.storage.save_settings(&self.settings);
                    self.sync_settings_tabs();
                    self.sync_full_state();
                    if enabled {
                        self.check_for_updates();
                    }
                }
                IpcIncoming::CheckForUpdates => {
                    self.check_for_updates();
                }
                IpcIncoming::OpenUpdateDownload => {
                    self.open_update_download();
                }
                IpcIncoming::ToggleFilterList { list_id, enabled } => {
                    self.adblock_manager.toggle_filter_list(&list_id, enabled);
                    if enabled {
                        if !self.settings.adblock_filter_lists.contains(&list_id) {
                            self.settings.adblock_filter_lists.push(list_id);
                        }
                    } else {
                        self.settings.adblock_filter_lists.retain(|l| l != &list_id);
                    }
                    self.storage.save_settings(&self.settings);
                    self.sync_settings_tabs();
                    self.sync_full_state();
                }
                IpcIncoming::AddCustomFilterRule { rule } => {
                    let r = rule.trim().to_string();
                    if !r.is_empty() {
                        self.adblock_manager.add_custom_rule(r.clone());
                        if !self.settings.adblock_custom_rules.contains(&r) {
                            self.settings.adblock_custom_rules.push(r);
                        }
                        self.storage.save_settings(&self.settings);
                        self.sync_settings_tabs();
                        self.sync_full_state();
                    }
                }
                IpcIncoming::RemoveCustomFilterRule { rule } => {
                    let r = rule.trim().to_string();
                    self.adblock_manager.remove_custom_rule(&r);
                    self.settings.adblock_custom_rules.retain(|item| item != &r);
                    self.storage.save_settings(&self.settings);
                    self.sync_settings_tabs();
                    self.sync_full_state();
                }
                IpcIncoming::ReportBlockedRequest {
                    domain,
                    url,
                    req_type,
                } => {
                    let now = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|d| {
                            let secs = d.as_secs();
                            let h = (secs / 3600 % 24) as u32;
                            let m = (secs / 60 % 60) as u32;
                            let s = (secs % 60) as u32;
                            format!("{:02}:{:02}:{:02}", h, m, s)
                        })
                        .unwrap_or_else(|_| "Just now".into());

                    self.blocked_logs.insert(
                        0,
                        crate::ipc::BlockedRequestLog {
                            domain,
                            url: crate::privacy::sanitize_local_log_url(&url),
                            req_type,
                            timestamp: now,
                        },
                    );
                    if self.blocked_logs.len() > 50 {
                        self.blocked_logs.truncate(50);
                    }
                    self.sync_settings_tabs();
                }
                IpcIncoming::ReportBlockedAd {
                    domain,
                    url,
                    req_type,
                } => {
                    let now = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|d| {
                            let secs = d.as_secs();
                            let h = (secs / 3600 % 24) as u32;
                            let m = (secs / 60 % 60) as u32;
                            let s = (secs % 60) as u32;
                            format!("{:02}:{:02}:{:02}", h, m, s)
                        })
                        .unwrap_or_else(|_| "Just now".into());

                    self.adblock_logs.insert(
                        0,
                        crate::ipc::BlockedRequestLog {
                            domain,
                            url: crate::privacy::sanitize_local_log_url(&url),
                            req_type,
                            timestamp: now,
                        },
                    );
                    if self.adblock_logs.len() > 50 {
                        self.adblock_logs.truncate(50);
                    }
                    self.sync_settings_tabs();
                }
                IpcIncoming::ClearBrowsingData {
                    cookies,
                    cache,
                    local_storage,
                } => {
                    #[cfg(target_os = "windows")]
                    if let Some(profile_view) = self.header_webview.as_ref().or_else(|| {
                        self.tabs
                            .iter()
                            .find(|tab| !tab.is_private)
                            .map(|tab| &tab.webview)
                    }) {
                        if let Err(error) = crate::desktop_data::clear_browsing_data(
                            profile_view,
                            cookies,
                            cache,
                            local_storage,
                        ) {
                            eprintln!("Could not clear WebView2 browsing data: {error}");
                        }
                    }

                    #[cfg(not(target_os = "windows"))]
                    {
                        let script = desktop_command(serde_json::json!({
                            "type": "clearBrowsingData",
                            "cookies": cookies,
                            "cache": cache,
                            "localStorage": local_storage,
                        }));
                        for tab in &self.tabs {
                            if !Self::is_internal_url(&tab.url) {
                                let _ = tab.webview.evaluate_script(&script);
                            }
                        }
                    }
                }
                IpcIncoming::OpenThemes => self.open_settings_view("themes"),
                IpcIncoming::OpenPrivacy => self.open_settings_view("privacy"),
                IpcIncoming::OpenAdblock => self.open_settings_view("adblock"),
                IpcIncoming::OpenExtensions => self.open_settings_view("extensions"),
                IpcIncoming::OpenSettings => self.open_settings_view("general"),
                IpcIncoming::InstallExtension { id_or_url, source } => {
                    match crate::extensions::download_and_install_extension(
                        &id_or_url,
                        source.as_deref(),
                    ) {
                        Ok(ext) => {
                            #[cfg(target_os = "windows")]
                            if let Some(profile_view) = self.header_webview.as_ref().or_else(|| {
                                self.tabs
                                    .iter()
                                    .find(|tab| !tab.is_private)
                                    .map(|tab| &tab.webview)
                            }) {
                                if let Err(error) = crate::desktop_data::install_browser_extension(
                                    profile_view,
                                    &ext.path,
                                ) {
                                    eprintln!("Could not dynamically register extension into WebView2: {error}");
                                }
                            }

                            self.extensions.retain(|e| e.id != ext.id);
                            self.extensions.push(ext);
                            self.storage.save_extensions(&self.extensions);
                            self.sync_settings_tabs();
                            self.sync_full_state();
                        }
                        Err(err) => {
                            eprintln!("Failed to install extension: {err}");
                        }
                    }
                }
                IpcIncoming::UninstallExtension { id } => {
                    self.extensions.retain(|e| e.id != id);
                    let ext_dir = crate::extensions::get_extensions_dir().join(&id);
                    let _ = std::fs::remove_dir_all(ext_dir);
                    self.storage.save_extensions(&self.extensions);
                    self.sync_settings_tabs();
                    self.sync_full_state();
                }
                IpcIncoming::ToggleExtension { id, enabled } => {
                    if let Some(ext) = self.extensions.iter_mut().find(|e| e.id == id) {
                        ext.enabled = enabled;
                    }
                    self.storage.save_extensions(&self.extensions);
                    self.sync_settings_tabs();
                    self.sync_full_state();
                }
                IpcIncoming::LoadUnpackedExtension { path } => {
                    match crate::extensions::load_unpacked_extension(&path) {
                        Ok(ext) => {
                            #[cfg(target_os = "windows")]
                            if let Some(profile_view) = self.header_webview.as_ref().or_else(|| {
                                self.tabs
                                    .iter()
                                    .find(|tab| !tab.is_private)
                                    .map(|tab| &tab.webview)
                            }) {
                                if let Err(error) = crate::desktop_data::install_browser_extension(
                                    profile_view,
                                    &ext.path,
                                ) {
                                    eprintln!("Could not dynamically register extension into WebView2: {error}");
                                }
                            }

                            self.extensions.retain(|e| e.id != ext.id);
                            self.extensions.push(ext);
                            self.storage.save_extensions(&self.extensions);
                            self.sync_settings_tabs();
                            self.sync_full_state();
                        }
                        Err(err) => {
                            eprintln!("Failed to load unpacked extension: {err}");
                        }
                    }
                }
                IpcIncoming::OpenExtensionOptions { id } => {
                    if let Some(ext) = self.extensions.iter().find(|e| e.id == id) {
                        let page = ext.options_page.as_deref().or(ext.popup_page.as_deref());
                        if let Some(options_page) = page {
                            let url = format!("chrome-extension://{}/{}", ext.id, options_page);
                            self.create_tab(&url);
                        }
                    }
                }
                IpcIncoming::OpenExtensionPopup { id } => {
                    if let Some(ext) = self.extensions.iter().find(|e| e.id == id) {
                        let page = ext.popup_page.as_deref().or(ext.options_page.as_deref());
                        if let Some(popup_page) = page {
                            let url = format!("chrome-extension://{}/{}", ext.id, popup_page);
                            self.create_tab(&url);
                        }
                    }
                }
                IpcIncoming::SetHeaderExpanded { expanded } => {
                    self.header_expanded = expanded;
                    let (width, _) = self.get_current_window_size();
                    let header_height = self.get_header_height();
                    if let Some(ref header) = self.header_webview {
                        let _ = header.set_bounds(Rect {
                            position: LogicalPosition::new(0.0, 0.0).into(),
                            size: LogicalSize::new(width, header_height).into(),
                        });
                    }
                }
                IpcIncoming::OpenHistory => self.open_history(),
                IpcIncoming::ClearHistory => self.clear_history(),
                IpcIncoming::OpenDownloads => self.open_downloads(),
                IpcIncoming::ClearDownloads => self.clear_downloads(),
                IpcIncoming::OpenDownload { download_id } => self.open_download(download_id),
                IpcIncoming::OpenDefaultBrowserSettings =>
                {
                    #[cfg(target_os = "windows")]
                    if !crate::menu_util::open_default_browser_settings() {
                        eprintln!("Could not open Windows Default Apps settings");
                    }
                }
                IpcIncoming::ShowBookmarkContextMenu { url } => {
                    #[cfg(target_os = "windows")]
                    if let Some(cmd) =
                        crate::menu_util::show_native_bookmark_context_menu(&self.window)
                    {
                        match cmd {
                            1 => {
                                self.create_tab(&url);
                            }
                            2 => {
                                crate::menu_util::copy_to_clipboard(&url);
                            }
                            3 => {
                                self.remove_bookmark(&url);
                            }
                            _ => {}
                        }
                    }
                }
                IpcIncoming::TabStateUpdate {
                    tab_id,
                    url,
                    title,
                    can_go_back,
                    can_go_forward,
                } => {
                    self.update_tab_state(tab_id, url, title, can_go_back, can_go_forward);
                }
                IpcIncoming::DragWindow => {
                    #[cfg(target_os = "windows")]
                    crate::drag_util::drag_window_native(&self.window);
                    #[cfg(not(target_os = "windows"))]
                    let _ = self.window.drag_window();
                }
                IpcIncoming::MinimizeWindow => {
                    self.window.set_minimized(true);
                }
                IpcIncoming::ToggleMaximizeWindow => {
                    let is_max = self.window.is_maximized();
                    self.window.set_maximized(!is_max);
                    self.sync_full_state();
                }
                IpcIncoming::CloseWindow => {
                    let _ = self.proxy.send_event(UserEvent::Exit);
                }
            }
        }
    }

    pub fn sync_full_state(&self) {
        if let Some(header) = &self.header_webview {
            let state = serde_json::json!({
                "tabs": self
                    .tabs
                    .iter()
                    .map(|t| IpcTabInfo {
                        id: t.id,
                        url: t.url.clone(),
                        title: t.title.clone(),
                        is_loading: t.is_loading,
                        can_go_back: t.can_go_back,
                        can_go_forward: t.can_go_forward,
                        is_private: t.is_private,
                    })
                    .collect::<Vec<_>>(),
                "active_tab_id": self.active_tab_id,
                "bookmarks": &self.bookmarks,
                "settings": &self.settings,
                "extensions": &self.extensions,
                "zoom": self.zoom,
            });

            let script = desktop_command(serde_json::json!({
                "type": "browserState",
                "state": state,
            }));
            let _ = header.evaluate_script(&script);
        }
    }

    pub fn sync_tab_update(&self, tab_id: u32) {
        if let Some(header) = &self.header_webview {
            if let Some(tab) = self.tabs.iter().find(|t| t.id == tab_id) {
                let info = IpcTabInfo {
                    id: tab.id,
                    url: tab.url.clone(),
                    title: tab.title.clone(),
                    is_loading: tab.is_loading,
                    can_go_back: tab.can_go_back,
                    can_go_forward: tab.can_go_forward,
                    is_private: tab.is_private,
                };
                let script = desktop_command(serde_json::json!({
                    "type": "tabUpdate",
                    "tab": info,
                }));
                let _ = header.evaluate_script(&script);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{is_allowed_content_ipc, popup_target_url};

    #[test]
    fn popup_target_accepts_web_urls() {
        assert_eq!(
            popup_target_url("https://accounts.example.test/login"),
            Some("https://accounts.example.test/login".into())
        );
        assert_eq!(
            popup_target_url("http://example.test/payment"),
            Some("http://example.test/payment".into())
        );
    }

    #[test]
    fn popup_target_maps_blank_windows_to_a_safe_tab() {
        assert_eq!(popup_target_url(""), Some("titan://newtab".into()));
        assert_eq!(
            popup_target_url("about:blank"),
            Some("titan://newtab".into())
        );
    }

    #[test]
    fn popup_target_rejects_privileged_and_script_urls() {
        for url in [
            "javascript:alert(document.cookie)",
            "data:text/html,unsafe",
            "file:///C:/Windows/System32/drivers/etc/hosts",
            "not a url",
        ] {
            assert_eq!(popup_target_url(url), None, "unexpectedly accepted {url}");
        }
    }

    #[test]
    fn untrusted_pages_can_only_send_scoped_browser_commands() {
        assert!(is_allowed_content_ipc(
            r#"{"type":"CloseTab","tab_id":7}"#,
            7
        ));
        assert!(is_allowed_content_ipc(
            r#"{"type":"NewTab","url":"titan://newtab"}"#,
            7
        ));
        assert!(!is_allowed_content_ipc(
            r#"{"type":"CloseTab","tab_id":8}"#,
            7
        ));
        assert!(!is_allowed_content_ipc(
            r#"{"type":"NewTab","url":"https://attacker.test"}"#,
            7
        ));
        assert!(!is_allowed_content_ipc(
            r#"{"type":"ClearBrowsingData","cookies":true,"cache":true,"local_storage":true}"#,
            7
        ));
        assert!(!is_allowed_content_ipc(r#"{"type":"CloseWindow"}"#, 7));
    }
}
