use crate::ipc::{Bookmark, BrowserModule, BrowserSettings, IpcBrowserState, IpcIncoming, IpcTabInfo};
use crate::storage::StorageManager;
use crate::url_utils::normalize_or_search_url_with_engine;
use std::sync::Arc;
use tao::dpi::{LogicalPosition, LogicalSize};
use tao::event_loop::EventLoopProxy;
use tao::window::Window;
use wry::{PageLoadEvent, Rect, WebView, WebViewBuilder};

pub const HEADER_HEIGHT_COLLAPSED: f64 = 76.0;
pub const HEADER_HEIGHT_EXPANDED: f64 = 102.0;

#[derive(Debug)]
pub enum UserEvent {
    Ipc(String),
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
}

impl BrowserManager {
    pub fn new(window: Arc<Window>, proxy: EventLoopProxy<UserEvent>) -> Self {
        let storage = StorageManager::new();
        let bookmarks = storage.load_bookmarks();
        let modules = storage.load_modules();
        let settings = storage.load_settings();
        let win_size = window.inner_size().to_logical::<f64>(window.scale_factor());

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

        // Open default initial tab (YouTube)
        self.create_tab("https://www.youtube.com");
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
        let state_json = serde_json::to_string(&serde_json::json!({
            "settings": self.settings,
            "modules": self.modules,
            "active_section": active_section,
        }))
        .unwrap_or_else(|_| "{}".into());

        let injection = format!(
            "<script>window.addEventListener('DOMContentLoaded', () => {{ window.initSettings && window.initSettings({}); }});</script>",
            state_json
        );
        html.replace("</body>", &format!("{}</body>", injection))
    }

    pub fn is_settings_url(url: &str) -> bool {
        url == "titan://settings"
            || url == "titan://themes"
            || url == "titan://modules"
            || url == "titan://darkmode"
            || url == "about:settings"
            || url == "about:themes"
    }

    pub fn is_internal_url(url: &str) -> bool {
        Self::is_settings_url(url) || url.starts_with("titan://") || url.starts_with("about:")
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
        let is_light = self.settings.theme == "titan-light";
        let force_adaptation = self.is_module_enabled("dark_reader");

        format!(
            r#"
            (function() {{
                const isLight = {is_light};
                const forceAdaptation = {force_adaptation};
                const host = (window.location.hostname || '').toLowerCase();
                const href = (window.location.href || '').toLowerCase();

                // 1. Clean up any previous forced styles if adaptation is off
                try {{
                    const preCanvas = document.getElementById('titan-pre-dark-canvas');
                    if (preCanvas) preCanvas.remove();
                    const adaptStyle = document.getElementById('titan-theme-adaptation-style');
                    if (!forceAdaptation && adaptStyle) adaptStyle.remove();
                }} catch(e) {{}}

                // If on internal / blank URL, stop here
                if (!host || href.startsWith('titan://') || href.startsWith('about:')) return;

                // 2. Standard prefers-color-scheme media query patching for JS-driven sites
                try {{
                    window._titanIsLight = isLight;
                    if (!window._titanMatchMediaPatched) {{
                        window._titanMatchMediaPatched = true;
                        const origMM = window.matchMedia;
                        window.matchMedia = function(q) {{
                            if (typeof q === 'string') {{
                                if (q.includes('prefers-color-scheme: dark')) {{
                                    return {{
                                        matches: !window._titanIsLight,
                                        media: q,
                                        onchange: null,
                                        addListener: function() {{}},
                                        removeListener: function() {{}},
                                        addEventListener: function() {{}},
                                        removeEventListener: function() {{}},
                                        dispatchEvent: function() {{ return false; }}
                                    }};
                                }}
                                if (q.includes('prefers-color-scheme: light')) {{
                                    return {{
                                        matches: !!window._titanIsLight,
                                        media: q,
                                        onchange: null,
                                        addListener: function() {{}},
                                        removeListener: function() {{}},
                                        addEventListener: function() {{}},
                                        removeEventListener: function() {{}},
                                        dispatchEvent: function() {{ return false; }}
                                    }};
                                }}
                            }}
                            return origMM ? origMM.call(window, q) : {{ matches: false, media: q }};
                        }};
                    }}
                }} catch(e) {{}}

                // 3. YouTube specific attribute
                const isYouTube = host.includes('youtube.com') || host.includes('youtu.be');
                if (isYouTube) {{
                    try {{
                        if (isLight) {{
                            document.documentElement.removeAttribute('dark');
                        }} else {{
                            document.documentElement.setAttribute('dark', 'true');
                        }}
                    }} catch(e) {{}}
                }}

                // 4. ONLY if Universal Webpage Theme Adaptation is explicitly ENABLED
                if (forceAdaptation) {{
                    function adaptTheme() {{
                        const isNativelyAdaptive = host.includes('google.') || host.includes('youtube.com') || host.includes('youtu.be') || host.includes('github.com') || host.includes('gitlab.com') || host.includes('reddit.com') || host.includes('duckduckgo.com') || host.includes('bing.com') || host.includes('x.com') || host.includes('twitter.com');
                        if (isNativelyAdaptive) return;

                        let el = document.getElementById('titan-theme-adaptation-style');
                        if (!isLight) {{
                            if (!el) {{
                                el = document.createElement('style');
                                el.id = 'titan-theme-adaptation-style';
                                el.textContent = 'html {{ filter: invert(100%) hue-rotate(180deg) contrast(96%) brightness(96%) !important; background-color: #121316 !important; }} img, video, canvas, svg, iframe, [style*="background-image"], .html5-video-player, picture {{ filter: invert(100%) hue-rotate(180deg) contrast(104%) brightness(104%) !important; }}';
                                (document.head || document.documentElement).appendChild(el);
                            }}
                        }} else {{
                            if (el) el.remove();
                        }}
                    }}

                    if (document.readyState === 'complete') {{
                        adaptTheme();
                    }} else {{
                        window.addEventListener('load', adaptTheme, {{ once: true }});
                    }}
                }}
            }})();
            "#,
            is_light = is_light,
            force_adaptation = force_adaptation
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

    pub fn create_tab(&mut self, target_url: &str) -> u32 {
        let tab_id = self.next_tab_id;
        self.next_tab_id += 1;

        let is_settings = Self::is_settings_url(target_url);
        let is_themes = target_url == "titan://themes" || target_url == "about:themes";

        let normalized_url = if is_settings {
            "titan://settings".to_string()
        } else {
            normalize_or_search_url_with_engine(target_url, &self.settings.search_engine)
        };

        let proxy_ipc = self.proxy.clone();
        let proxy_load = self.proxy.clone();
        let tab_id_copy = tab_id;
        let theme_script = if is_settings {
            "".to_string()
        } else {
            self.get_theme_injection_script()
        };

        let init_script = format!(
            r#"
            (function() {{
                const tabId = {tab_id};
                let lastUrl = '';
                let lastTitle = '';
                let notifyTimer = null;

                function notify() {{
                    clearTimeout(notifyTimer);
                    notifyTimer = setTimeout(() => {{
                        const curUrl = window.location.href;
                        const curTitle = document.title || window.location.hostname || 'New Tab';
                        if (curUrl !== lastUrl || curTitle !== lastTitle) {{
                            lastUrl = curUrl;
                            lastTitle = curTitle;
                            try {{
                                window.ipc.postMessage(JSON.stringify({{
                                    type: 'TabStateUpdate',
                                    tab_id: tabId,
                                    url: curUrl,
                                    title: curTitle,
                                    can_go_back: window.history.length > 1,
                                    can_go_forward: true
                                }}));
                            }} catch(e) {{}}
                        }}
                    }}, 400);
                }}

                window.addEventListener('popstate', notify);
                window.addEventListener('load', notify);
                document.addEventListener('visibilitychange', notify);
                setTimeout(notify, 500);

                {theme_script}
            }})();
            "#,
            tab_id = tab_id,
            theme_script = theme_script
        );

        let content_bounds = self.get_content_bounds();
        let bg_color = if is_settings {
            self.get_theme_background_color()
        } else {
            (255, 255, 255, 255)
        };

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

        let internal_html = if is_settings {
            let section = if is_themes { "themes" } else { "general" };
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

        let default_title = if is_themes {
            "Themes".to_string()
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
            is_loading: !is_settings,
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

        // 1. Show, reposition and focus the target active tab FIRST
        if let Some(active_tab) = self.tabs.iter_mut().find(|t| t.id == target_id) {
            let _ = active_tab.webview.set_bounds(content_bounds);
            let _ = active_tab.webview.set_visible(true);
            let _ = active_tab.webview.focus();
            if let Ok(current_url) = active_tab.webview.url() {
                if !current_url.is_empty() && current_url != "about:blank" {
                    active_tab.url = current_url;
                }
            }
        }

        // 2. Hide all other tabs (without collapsing their bounds)
        for tab in &mut self.tabs {
            if tab.id != target_id {
                let _ = tab.webview.set_visible(false);
            }
        }

        self.active_tab_id = Some(target_id);
        self.sync_full_state();
    }

    pub fn close_tab(&mut self, target_id: u32) {
        if let Some(pos) = self.tabs.iter().position(|t| t.id == target_id) {
            let was_active = self.active_tab_id == Some(target_id);
            self.tabs.remove(pos);

            if self.tabs.is_empty() {
                // If all tabs closed, create a fresh New Tab
                self.create_tab("https://www.google.com");
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
        if Self::is_settings_url(input) {
            let is_themes = input == "titan://themes" || input == "about:themes";
            let section = if is_themes { "themes" } else { "general" };
            let html = self.get_settings_html(section);
            if let Some(active_id) = self.active_tab_id {
                if let Some(tab) = self.tabs.iter_mut().find(|t| t.id == active_id) {
                    tab.url = "titan://settings".into();
                    tab.title = if is_themes { "Themes".into() } else { "Settings".into() };
                    tab.is_loading = false;
                    let _ = tab.webview.load_html(&html);
                    let _ = tab.webview.focus();
                }
                self.sync_tab_update(active_id);
            }
            return;
        }

        let normalized = normalize_or_search_url_with_engine(input, &self.settings.search_engine);
        if let Some(active_id) = self.active_tab_id {
            if let Some(tab) = self.tabs.iter_mut().find(|t| t.id == active_id) {
                tab.url = normalized.clone();
                tab.is_loading = true;
                let _ = tab.webview.load_url(&normalized);
                let _ = tab.webview.focus();
            }
            self.sync_tab_update(active_id);
        }
    }

    pub fn go_back(&mut self) {
        if let Some(active_id) = self.active_tab_id {
            if let Some(tab) = self.tabs.iter_mut().find(|t| t.id == active_id) {
                let _ = tab.webview.evaluate_script("window.history.back();");
                let _ = tab.webview.focus();
            }
        }
    }

    pub fn go_forward(&mut self) {
        if let Some(active_id) = self.active_tab_id {
            if let Some(tab) = self.tabs.iter_mut().find(|t| t.id == active_id) {
                let _ = tab.webview.evaluate_script("window.history.forward();");
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

            if Self::is_settings_url(&tab_url) {
                let html = self.get_settings_html("general");
                if let Some(tab) = self.tabs.iter_mut().find(|t| t.id == active_id) {
                    let _ = tab.webview.load_html(&html);
                    let _ = tab.webview.focus();
                }
            } else {
                if let Some(tab) = self.tabs.iter_mut().find(|t| t.id == active_id) {
                    let _ = tab.webview.evaluate_script("window.location.reload();");
                    let _ = tab.webview.focus();
                }
            }
        }
    }

    pub fn go_home(&mut self) {
        self.navigate_active_tab("https://www.youtube.com");
    }

    pub fn set_zoom(&mut self, zoom: f64) {
        self.zoom = zoom.clamp(0.4, 3.0);
        if let Some(active_id) = self.active_tab_id {
            if let Some(tab) = self.tabs.iter().find(|t| t.id == active_id) {
                let _ = tab.webview.zoom(self.zoom);
            }
        }
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
        let state_json = serde_json::to_string(&serde_json::json!({
            "settings": self.settings,
            "modules": self.modules,
        }))
        .unwrap_or_else(|_| "{}".into());

        let script = format!("window.initSettings && window.initSettings({});", state_json);
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

    pub fn on_page_load_started(&mut self, tab_id: u32, url: String) {
        if let Some(tab) = self.tabs.iter_mut().find(|t| t.id == tab_id) {
            tab.is_loading = true;
            if !url.is_empty() && url != "about:blank" {
                tab.url = url;
            }
        }
        self.sync_tab_update(tab_id);
    }

    pub fn on_page_load_finished(&mut self, tab_id: u32, url: String) {
        let theme_script = self.get_theme_injection_script();

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

            // Inject matching theme script for web content
            if !Self::is_internal_url(&tab.url) {
                let _ = tab.webview.evaluate_script(&theme_script);
            }
        }
        self.sync_tab_update(tab_id);
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
            let is_active = Some(tab.id) == active_id;
            let _ = tab.webview.set_bounds(content_bounds);
            let _ = tab.webview.set_visible(is_active);
        }
    }

    pub fn handle_incoming_ipc(&mut self, msg_str: &str) {
        if let Ok(msg) = serde_json::from_str::<IpcIncoming>(msg_str) {
            match msg {
                IpcIncoming::UiReady => {
                    self.sync_full_state();
                }
                IpcIncoming::NewTab { url } => {
                    let default_url = url.unwrap_or_else(|| "https://www.google.com".into());
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
                IpcIncoming::OpenThemes => {
                    if let Some(pos) = self.tabs.iter().position(|t| Self::is_settings_url(&t.url)) {
                        let tab_id = self.tabs[pos].id;
                        self.switch_tab(tab_id);
                        let _ = self.tabs[pos].webview.evaluate_script("window.switchView && window.switchView('themes');");
                    } else {
                        self.create_tab("titan://themes");
                    }
                }
                IpcIncoming::OpenSettings => {
                    if let Some(pos) = self.tabs.iter().position(|t| Self::is_settings_url(&t.url)) {
                        let tab_id = self.tabs[pos].id;
                        self.switch_tab(tab_id);
                        let _ = self.tabs[pos].webview.evaluate_script("window.switchView && window.switchView('general');");
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
            };

            if let Ok(json) = serde_json::to_string(&state) {
                let script = format!("window.onBrowserState && window.onBrowserState({});", json);
                let _ = header.evaluate_script(&script);
            }
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
                if let Ok(json) = serde_json::to_string(&info) {
                    let script = format!("window.onTabUpdate && window.onTabUpdate({});", json);
                    let _ = header.evaluate_script(&script);
                }
            }
        }
    }
}
