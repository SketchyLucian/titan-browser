use crate::ipc::{
    Bookmark, BrowserModule, BrowserSettings, IpcBrowserState, IpcIncoming, IpcTabInfo,
};
use crate::storage::StorageManager;
use crate::updater::{UpdateCheckResult, UpdateInfo, UpdateState, UpdateStatus};
use crate::url_utils::normalize_or_search_url_with_engine;
use std::sync::Arc;
use std::thread;
use tao::dpi::{LogicalPosition, LogicalSize};
use tao::event_loop::EventLoopProxy;
use tao::window::Window;
use wry::{PageLoadEvent, Rect, WebView, WebViewBuilder};

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

#[derive(Debug)]
pub enum UserEvent {
    Ipc(String),
    UpdateCheckFinished(UpdateCheckResult),
    PageLoadStarted { tab_id: u32, url: String },
    PageLoadFinished { tab_id: u32, url: String },
    Exit,
}

pub struct Tab {
    pub id: u32,
    pub url: String,
    pub title: String,
    pub is_loading: bool,
    pub can_go_back: bool,
    pub can_go_forward: bool,
    pub webview: WebView,
}

pub struct BrowserManager {
    pub window: Arc<Window>,
    pub proxy: EventLoopProxy<UserEvent>,
    pub storage: StorageManager,
    pub header_webview: Option<WebView>,
    pub tabs: Vec<Tab>,
    pub active_tab_id: Option<u32>,
    pub next_tab_id: u32,
    pub bookmarks: Vec<Bookmark>,
    pub modules: Vec<BrowserModule>,
    pub settings: BrowserSettings,
    pub zoom: f64,
    pub window_size: (f64, f64),
    pub blocked_logs: Vec<crate::ipc::BlockedRequestLog>,
    pub adblock_logs: Vec<crate::ipc::BlockedRequestLog>,
    pub adblock_manager: crate::adblock_engine::AdblockEngineManager,
    pub update_state: UpdateState,
}

impl BrowserManager {
    pub fn new(window: Arc<Window>, proxy: EventLoopProxy<UserEvent>) -> Self {
        let storage = StorageManager::new();
        let bookmarks = storage.load_bookmarks();
        let modules = storage.load_modules();
        let settings = storage.load_settings();
        let win_size = window.inner_size().to_logical::<f64>(window.scale_factor());

        let adblock_manager = crate::adblock_engine::AdblockEngineManager::new();
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

        Self {
            window,
            proxy,
            storage,
            header_webview: None,
            tabs: Vec::new(),
            active_tab_id: None,
            next_tab_id: 1,
            bookmarks,
            modules,
            settings,
            zoom: 1.0,
            window_size: (win_size.width, win_size.height),
            blocked_logs: Vec::new(),
            adblock_logs: Vec::new(),
            adblock_manager,
            update_state: UpdateState::default(),
        }
    }

    pub fn get_header_height(&self) -> f64 {
        if self.settings.show_bookmarks_bar && !self.bookmarks.is_empty() {
            HEADER_HEIGHT_EXPANDED
        } else {
            HEADER_HEIGHT_COLLAPSED
        }
    }

    pub fn init(&mut self) {
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

        let header = WebViewBuilder::new()
            .with_bounds(header_bounds)
            .with_background_color((r, g, b, a))
            .with_transparent(false)
            .with_html(&html_content)
            .with_ipc_handler(move |req| {
                let _ = proxy_clone.send_event(UserEvent::Ipc(req.body().clone()));
            })
            .build(&*self.window)
            .expect("Failed to create header webview");

        self.header_webview = Some(header);

        // Open default initial tab (Titan New Tab)
        self.create_tab("titan://newtab");

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
            || url == "titan://darkmode"
            || url == "about:settings"
            || url == "about:themes"
            || url == "about:privacy"
            || url == "about:adblock"
            || url == "about:shields"
    }

    pub fn is_internal_url(url: &str) -> bool {
        Self::is_settings_url(url)
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
        let (dynamic_selectors, dynamic_scriptlet) =
            self.adblock_manager.get_cosmetic_resources(target_url);

        render_typescript(
            include_str!("../web-scripts/dist/desktop-adblock.js"),
            "__TITAN_DESKTOP_ADBLOCK_CONFIG__",
            serde_json::json!({
                "enabled": self.settings.adblock_enabled,
                "blockVideoAds": self.settings.adblock_block_video_ads,
                "cosmeticFiltering": self.settings.adblock_cosmetic_filtering,
                "blockPopups": self.settings.adblock_block_popups,
                "aggressiveMode": self.settings.adblock_aggressive_mode,
                "whitelistedDomains": self.settings.adblock_whitelisted_domains,
                "blockedDomains": self.settings.adblock_blocked_domains,
                "dynamicSelectors": dynamic_selectors,
                "scriptletCode": dynamic_scriptlet,
            }),
        )
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

    pub fn create_tab(&mut self, target_url: &str) -> u32 {
        let tab_id = self.next_tab_id;
        self.next_tab_id += 1;

        let is_newtab = Self::is_newtab_url(target_url);
        let is_settings = Self::is_settings_url(target_url);
        let is_themes = target_url == "titan://themes" || target_url == "about:themes";
        let is_privacy = target_url == "titan://privacy" || target_url == "about:privacy";
        let is_adblock = target_url == "titan://adblock"
            || target_url == "about:adblock"
            || target_url == "titan://shields"
            || target_url == "about:shields";

        let normalized_url = if is_newtab {
            "titan://newtab".to_string()
        } else if is_settings {
            "titan://settings".to_string()
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
        let tab_id_copy = tab_id;
        let is_internal = is_newtab || is_settings;
        let theme_script = self.get_theme_injection_script();
        let privacy_script = self.get_privacy_injection_script();
        let adblock_script = self.get_adblock_injection_script(&normalized_url);

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
        ]
        .join("\n");

        let content_bounds = self.get_content_bounds();
        let bg_color = self.get_theme_background_color();

        let builder = WebViewBuilder::new()
            .with_bounds(content_bounds)
            .with_background_color(bg_color)
            .with_transparent(false)
            .with_visible(false)
            .with_initialization_script(&init_script)
            .with_ipc_handler(move |req| {
                let _ = proxy_ipc.send_event(UserEvent::Ipc(req.body().clone()));
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
            } else {
                "general"
            };
            Some(self.get_settings_html(section))
        } else {
            None
        };

        let webview = if let Some(ref html) = internal_html {
            builder.with_html(html)
        } else {
            builder.with_url(&normalized_url)
        }
        .build(&*self.window)
        .expect("Failed to create content webview for tab");

        let default_title = if is_newtab {
            "New Tab".to_string()
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

        let new_tab = Tab {
            id: tab_id,
            url: normalized_url,
            title: default_title,
            is_loading: !is_internal,
            can_go_back: false,
            can_go_forward: false,
            webview,
        };

        self.tabs.push(new_tab);
        self.switch_tab(tab_id);
        tab_id
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
    }

    pub fn close_tab(&mut self, target_id: u32) {
        if let Some(pos) = self.tabs.iter().position(|t| t.id == target_id) {
            let was_active = self.active_tab_id == Some(target_id);
            self.tabs.remove(pos);

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
            }
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
            if let Some(tab) = self.tabs.iter_mut().find(|t| t.id == id) {
                if !url.is_empty() && url != "about:blank" {
                    tab.url = url;
                }
                if !title.is_empty() {
                    tab.title = title;
                }
                if let Some(back) = can_go_back {
                    tab.can_go_back = back;
                }
                if let Some(forward) = can_go_forward {
                    tab.can_go_forward = forward;
                }
            }
            self.sync_tab_update(id);
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
                        self.storage.save_settings(&self.settings);
                        self.sync_settings_tabs();
                    }
                }
                IpcIncoming::AddAdblockWhitelist { domain } => {
                    if let Some(d) = crate::privacy::normalize_domain_rule(&domain) {
                        if !self.settings.adblock_whitelisted_domains.contains(&d) {
                            self.settings.adblock_whitelisted_domains.push(d);
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
                        self.storage.save_settings(&self.settings);
                        self.sync_settings_tabs();
                    }
                }
                IpcIncoming::ResetAdblockRules => {
                    self.settings.adblock_blocked_domains = crate::ipc::default_adblock_domains();
                    self.settings.adblock_whitelisted_domains.clear();
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
                IpcIncoming::OpenThemes => {
                    if let Some(pos) = self.tabs.iter().position(|t| Self::is_settings_url(&t.url))
                    {
                        let tab_id = self.tabs[pos].id;
                        self.switch_tab(tab_id);
                        let script = desktop_command(serde_json::json!({
                            "type": "switchSettingsView",
                            "view": "themes",
                        }));
                        let _ = self.tabs[pos].webview.evaluate_script(&script);
                    } else {
                        self.create_tab("titan://themes");
                    }
                }
                IpcIncoming::OpenPrivacy => {
                    if let Some(pos) = self.tabs.iter().position(|t| Self::is_settings_url(&t.url))
                    {
                        let tab_id = self.tabs[pos].id;
                        self.switch_tab(tab_id);
                        let script = desktop_command(serde_json::json!({
                            "type": "switchSettingsView",
                            "view": "privacy",
                        }));
                        let _ = self.tabs[pos].webview.evaluate_script(&script);
                    } else {
                        self.create_tab("titan://privacy");
                    }
                }
                IpcIncoming::OpenAdblock => {
                    if let Some(pos) = self.tabs.iter().position(|t| Self::is_settings_url(&t.url))
                    {
                        let tab_id = self.tabs[pos].id;
                        self.switch_tab(tab_id);
                        let script = desktop_command(serde_json::json!({
                            "type": "switchSettingsView",
                            "view": "adblock",
                        }));
                        let _ = self.tabs[pos].webview.evaluate_script(&script);
                    } else {
                        self.create_tab("titan://adblock");
                    }
                }
                IpcIncoming::OpenSettings => {
                    if let Some(pos) = self.tabs.iter().position(|t| Self::is_settings_url(&t.url))
                    {
                        let tab_id = self.tabs[pos].id;
                        self.switch_tab(tab_id);
                        let script = desktop_command(serde_json::json!({
                            "type": "switchSettingsView",
                            "view": "general",
                        }));
                        let _ = self.tabs[pos].webview.evaluate_script(&script);
                    } else {
                        self.create_tab("titan://settings");
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
            let state = IpcBrowserState {
                tabs: self
                    .tabs
                    .iter()
                    .map(|t| IpcTabInfo {
                        id: t.id,
                        url: t.url.clone(),
                        title: t.title.clone(),
                        is_loading: t.is_loading,
                        can_go_back: t.can_go_back,
                        can_go_forward: t.can_go_forward,
                    })
                    .collect(),
                active_tab_id: self.active_tab_id,
                bookmarks: self.bookmarks.clone(),
                modules: self.modules.clone(),
                settings: self.settings.clone(),
                zoom: self.zoom,
                search_engine: self.settings.search_engine.clone(),
                is_maximized: self.window.is_maximized(),
                blocked_logs: self.blocked_logs.clone(),
                adblock_logs: self.adblock_logs.clone(),
                adblock_filter_lists: self.adblock_manager.get_filter_lists_info(),
                adblock_stats: self.adblock_manager.get_stats(),
                update_state: self.update_state.clone(),
            };

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
