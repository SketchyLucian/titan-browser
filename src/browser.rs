use crate::ipc::{Bookmark, BrowserModule, BrowserSettings, IpcBrowserState, IpcIncoming, IpcTabInfo};
use crate::storage::StorageManager;
use crate::url_utils::normalize_or_search_url_with_engine;
use tauri::{
    AppHandle, Emitter, Manager, WebviewUrl, Window,
};
#[cfg(desktop)]
use tauri::{LogicalPosition, LogicalSize, Position, Size, Webview, WebviewBuilder};

pub const HEADER_HEIGHT_COLLAPSED: f64 = 102.0;
pub const HEADER_HEIGHT_EXPANDED: f64 = 138.0;

pub struct Tab {
    pub id: u32,
    pub url: String,
    pub title: String,
    pub is_loading: bool,
    pub can_go_back: bool,
    pub can_go_forward: bool,
    pub label: String,
    #[cfg(desktop)]
    pub webview: Webview,
}

pub struct BrowserManager {
    pub app: AppHandle,
    pub window: Window,
    pub storage: StorageManager,
    #[cfg(desktop)]
    pub header_webview: Option<Webview>,
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
    pub fn new(app: AppHandle, window: Window) -> Self {
        let storage = StorageManager::new();
        let bookmarks = storage.load_bookmarks();
        let modules = storage.load_modules();
        let settings = storage.load_settings();

        #[cfg(desktop)]
        let win_size = window
            .inner_size()
            .map(|s| {
                let scale = window.scale_factor().unwrap_or(1.0);
                (s.width as f64 / scale, s.height as f64 / scale)
            })
            .unwrap_or((1300.0, 850.0));

        #[cfg(not(desktop))]
        let win_size = (800.0, 600.0);

        Self {
            app,
            window,
            storage,
            #[cfg(desktop)]
            header_webview: None,
            tabs: Vec::new(),
            active_tab_id: None,
            next_tab_id: 1,
            bookmarks,
            modules,
            settings,
            zoom: 1.0,
            window_size: win_size,
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
        #[cfg(target_os = "windows")]
        {
            let (r, g, b, _) = self.get_theme_background_color();
            crate::drag_util::apply_dark_window_attributes(&self.window, (r, g, b));
        }

        #[cfg(desktop)]
        {
            let header_builder = WebviewBuilder::new("header", WebviewUrl::App("index.html".into()))
                .initialization_script(
                    r#"
                    window.__TITAN_IPC__ = true;
                    if (!window.ipc) {
                        window.ipc = {
                            postMessage: function(msg) {
                                if (window.__TAURI__ && window.__TAURI__.core) {
                                    window.__TAURI__.core.invoke('handle_ipc', { message: msg });
                                }
                            }
                        };
                    }
                    "#,
                );

            let (width, _) = self.window_size;
            let header_height = self.get_header_height();
            let header = self
                .window
                .add_child(
                    header_builder,
                    LogicalPosition::new(0.0, 0.0),
                    LogicalSize::new(width, header_height),
                )
                .expect("Failed to create header webview");

            self.header_webview = Some(header);
        }

        // Open default initial tab (YouTube)
        self.create_tab("https://www.youtube.com");
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
        #[cfg(desktop)]
        {
            if let (Ok(scale), Ok(size)) = (self.window.scale_factor(), self.window.inner_size()) {
                let logical_w = size.width as f64 / scale;
                let logical_h = size.height as f64 / scale;
                if logical_w > 10.0 && logical_h > 10.0 {
                    return (logical_w, logical_h);
                }
            }
        }
        self.window_size
    }

    #[cfg(desktop)]
    fn get_content_position_and_size(&self) -> (LogicalPosition<f64>, LogicalSize<f64>) {
        let (width, height) = self.get_current_window_size();
        let header_height = self.get_header_height();
        let content_height = (height - header_height).max(10.0);
        (
            LogicalPosition::new(0.0, header_height),
            LogicalSize::new(width, content_height),
        )
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

                try {{
                    const applyDarkPre = () => {{
                        try {{
                            if (!isLight) {{
                                if (document.documentElement) {{
                                    document.documentElement.style.backgroundColor = '#121318';
                                    document.documentElement.style.colorScheme = 'dark';
                                    if (!document.getElementById('titan-pre-dark-canvas')) {{
                                        const preStyle = document.createElement('style');
                                        preStyle.id = 'titan-pre-dark-canvas';
                                        preStyle.textContent = `
                                            html, :root {{
                                                background-color: #121318 !important;
                                                color-scheme: dark !important;
                                            }}
                                        `;
                                        (document.head || document.documentElement).appendChild(preStyle);
                                    }}
                                }}
                            }} else {{
                                const pre = document.getElementById('titan-pre-dark-canvas');
                                if (pre) pre.remove();
                            }}
                        }} catch(e) {{}}
                    }};

                    applyDarkPre();
                    if (document.readyState === 'loading') {{
                        document.addEventListener('DOMContentLoaded', applyDarkPre, {{ once: true }});
                    }}
                }} catch(e) {{}}

                try {{
                    const isInternal = window.location.protocol === 'titan:' || 
                                       window.location.protocol === 'about:' || 
                                       window.location.href.includes('settings.html') ||
                                       window.location.href.includes('themes.html') ||
                                       window.location.href.startsWith('tauri://') ||
                                       window.location.href.startsWith('http://tauri.localhost');
                    if (isInternal) return;

                    const STYLE_ID = 'titan-universal-dark-theme-engine';
                    let existing = document.getElementById(STYLE_ID);

                    if (forceAdaptation && !isLight) {{
                        if (!existing) {{
                            const style = document.createElement('style');
                            style.id = STYLE_ID;
                            style.textContent = `
                                html {{
                                    filter: invert(90%) hue-rotate(180deg) contrast(95%) brightness(95%) !important;
                                    background-color: #121318 !important;
                                }}
                                img, video, canvas, iframe, svg, [style*="background-image"], picture, embed, object {{
                                    filter: invert(100%) hue-rotate(180deg) !important;
                                }}
                            `;
                            (document.head || document.documentElement).appendChild(style);
                        }}
                    }} else {{
                        if (existing) existing.remove();
                    }}
                }} catch(e) {{}}
            }})();
            "#,
            is_light = is_light,
            force_adaptation = force_adaptation
        )
    }

    pub fn inject_theme_into_all_tabs(&self) {
        let script = self.get_theme_injection_script();
        #[cfg(desktop)]
        for tab in &self.tabs {
            let _ = tab.webview.eval(&script);
        }
        #[cfg(not(desktop))]
        if let Some(win) = self.app.get_webview_window("main") {
            let _ = win.eval(&script);
        }
    }

    pub fn inject_theme_into_tab(&self, tab_id: u32) {
        let script = self.get_theme_injection_script();
        #[cfg(desktop)]
        if let Some(tab) = self.tabs.iter().find(|t| t.id == tab_id) {
            let _ = tab.webview.eval(&script);
        }
        #[cfg(not(desktop))]
        {
            let _ = tab_id;
            if let Some(win) = self.app.get_webview_window("main") {
                let _ = win.eval(&script);
            }
        }
    }

    #[allow(unused_variables)]
    pub fn create_tab(&mut self, initial_url: &str) -> u32 {
        let tab_id = self.next_tab_id;
        self.next_tab_id += 1;

        let final_url = if initial_url.is_empty() || initial_url == "about:blank" {
            "https://www.youtube.com".to_string()
        } else {
            initial_url.to_string()
        };

        let is_internal = Self::is_internal_url(&final_url);
        let target_url = if is_internal {
            final_url.clone()
        } else {
            normalize_or_search_url_with_engine(&final_url, &self.settings.search_engine)
        };

        let label = format!("tab-{}", tab_id);
        let webview_url = if is_internal {
            if final_url.contains("themes") {
                WebviewUrl::App("themes.html".into())
            } else {
                WebviewUrl::App("settings.html".into())
            }
        } else {
            match target_url.parse() {
                Ok(u) => WebviewUrl::External(u),
                Err(_) => WebviewUrl::External("https://www.google.com".parse().unwrap()),
            }
        };

        let app_handle = self.app.clone();
        let tab_label = label.clone();
        let theme_script = self.get_theme_injection_script();

        #[cfg(desktop)]
        let builder = WebviewBuilder::new(&label, webview_url)
            .initialization_script(&theme_script)
            .initialization_script(
                r#"
                window.__TITAN_IPC__ = true;
                if (!window.ipc) {
                    window.ipc = {
                        postMessage: function(msg) {
                            if (window.__TAURI__ && window.__TAURI__.core) {
                                window.__TAURI__.core.invoke('handle_ipc', { message: msg });
                            }
                        }
                    };
                }
                "#,
            )
            .on_page_load(move |_webview, payload| {
                let current_url = payload.url().to_string();
                let is_finished = match payload.event() {
                    tauri::webview::PageLoadEvent::Started => false,
                    tauri::webview::PageLoadEvent::Finished => true,
                };
                let _ = app_handle.emit(
                    "page-load-event",
                    serde_json::json!({
                        "tab_id": tab_id,
                        "tab_label": tab_label,
                        "url": current_url,
                        "finished": is_finished
                    }),
                );
            });

        #[cfg(desktop)]
        let webview = {
            let (pos, size) = self.get_content_position_and_size();
            self.window
                .add_child(builder, pos, size)
                .expect("Failed to create tab webview")
        };

        // Default zoom
        #[cfg(desktop)]
        {
            let _ = webview.set_zoom(self.zoom);
            if self.active_tab_id.is_some() {
                let _ = webview.set_position(Position::Logical(LogicalPosition::new(-9999.0, -9999.0)));
                let _ = webview.hide();
            }
        }

        let initial_title = if is_internal {
            "Settings - Titan".to_string()
        } else if target_url.contains("youtube.com") {
            "YouTube".to_string()
        } else {
            "New Tab".to_string()
        };

        let tab = Tab {
            id: tab_id,
            url: target_url,
            title: initial_title,
            is_loading: true,
            can_go_back: false,
            can_go_forward: false,
            label,
            #[cfg(desktop)]
            webview,
        };

        self.tabs.push(tab);
        self.switch_tab(tab_id);

        tab_id
    }

    pub fn switch_tab(&mut self, tab_id: u32) {
        #[cfg(desktop)]
        {
            let (pos, size) = self.get_content_position_and_size();

            for tab in &mut self.tabs {
                if tab.id == tab_id {
                    let _ = tab.webview.set_position(Position::Logical(pos));
                    let _ = tab.webview.set_size(Size::Logical(size));
                    let _ = tab.webview.show();
                } else {
                    let _ = tab
                        .webview
                        .set_position(Position::Logical(LogicalPosition::new(-9999.0, -9999.0)));
                    let _ = tab.webview.hide();
                }
            }
        }

        self.active_tab_id = Some(tab_id);
        self.inject_theme_into_tab(tab_id);
        self.sync_ui_state();
    }

    pub fn close_tab(&mut self, tab_id: u32) {
        let index = self.tabs.iter().position(|t| t.id == tab_id);
        if let Some(idx) = index {
            let removed_tab = self.tabs.remove(idx);
            #[cfg(desktop)]
            {
                let _ = removed_tab.webview.close();
            }
            #[cfg(not(desktop))]
            {
                let _ = removed_tab;
            }

            if self.tabs.is_empty() {
                let _ = self.window.close();
                return;
            }

            if self.active_tab_id == Some(tab_id) {
                let new_active_index = if idx >= self.tabs.len() {
                    self.tabs.len() - 1
                } else {
                    idx
                };
                let new_active_id = self.tabs[new_active_index].id;
                self.switch_tab(new_active_id);
            } else {
                self.sync_ui_state();
            }
        }
    }

    pub fn navigate(&mut self, url: &str) {
        if let Some(active_id) = self.active_tab_id {
            let is_internal = Self::is_internal_url(url);
            let final_url = if is_internal {
                url.to_string()
            } else {
                normalize_or_search_url_with_engine(url, &self.settings.search_engine)
            };

            if let Some(tab) = self.tabs.iter_mut().find(|t| t.id == active_id) {
                tab.url = final_url.clone();
                tab.is_loading = true;

                #[cfg(desktop)]
                {
                    if is_internal {
                        let page_name = if final_url.contains("themes") {
                            "themes.html"
                        } else {
                            "settings.html"
                        };
                        let _ = tab.webview.eval(&format!("window.location.href = '{}';", page_name));
                    } else if let Ok(parsed) = final_url.parse() {
                        let _ = tab.webview.navigate(parsed);
                    }
                }

                #[cfg(not(desktop))]
                {
                    if let Some(win) = self.app.get_webview_window("main") {
                        if is_internal {
                            let page_name = if final_url.contains("themes") {
                                "themes.html"
                            } else {
                                "settings.html"
                            };
                            let _ = win.eval(&format!("window.location.href = '{}';", page_name));
                        } else if let Ok(parsed) = final_url.parse() {
                            let _ = win.navigate(parsed);
                        }
                    }
                }
            }

            self.sync_ui_state();
        }
    }

    #[allow(unused_variables)]
    pub fn go_back(&mut self) {
        if let Some(active_id) = self.active_tab_id {
            #[cfg(desktop)]
            if let Some(tab) = self.tabs.iter().find(|t| t.id == active_id) {
                let _ = tab.webview.eval("window.history.back();");
            }
            #[cfg(not(desktop))]
            if let Some(win) = self.app.get_webview_window("main") {
                let _ = win.eval("window.history.back();");
            }
        }
    }

    #[allow(unused_variables)]
    pub fn go_forward(&mut self) {
        if let Some(active_id) = self.active_tab_id {
            #[cfg(desktop)]
            if let Some(tab) = self.tabs.iter().find(|t| t.id == active_id) {
                let _ = tab.webview.eval("window.history.forward();");
            }
            #[cfg(not(desktop))]
            if let Some(win) = self.app.get_webview_window("main") {
                let _ = win.eval("window.history.forward();");
            }
        }
    }

    #[allow(unused_variables)]
    pub fn reload(&mut self) {
        if let Some(active_id) = self.active_tab_id {
            #[cfg(desktop)]
            if let Some(tab) = self.tabs.iter().find(|t| t.id == active_id) {
                let _ = tab.webview.eval("window.location.reload();");
            }
            #[cfg(not(desktop))]
            if let Some(win) = self.app.get_webview_window("main") {
                let _ = win.eval("window.location.reload();");
            }
        }
    }

    pub fn go_home(&mut self) {
        self.navigate("https://www.youtube.com");
    }

    pub fn set_zoom(&mut self, zoom: f64) {
        self.zoom = zoom.clamp(0.25, 5.0);
        #[cfg(desktop)]
        for tab in &self.tabs {
            let _ = tab.webview.set_zoom(self.zoom);
        }
        #[cfg(not(desktop))]
        if let Some(win) = self.app.get_webview_window("main") {
            let _ = win.set_zoom(self.zoom);
        }
        self.sync_ui_state();
    }

    pub fn toggle_bookmark(&mut self, title: String, url: String) {
        if let Some(idx) = self.bookmarks.iter().position(|b| b.url == url) {
            self.bookmarks.remove(idx);
        } else {
            self.bookmarks.push(Bookmark { title, url });
        }
        self.storage.save_bookmarks(&self.bookmarks);
        self.update_header_layout();
        self.sync_ui_state();
    }

    pub fn remove_bookmark(&mut self, url: String) {
        self.bookmarks.retain(|b| b.url != url);
        self.storage.save_bookmarks(&self.bookmarks);
        self.update_header_layout();
        self.sync_ui_state();
    }

    pub fn toggle_module(&mut self, module_id: String, enabled: bool) {
        if let Some(module) = self.modules.iter_mut().find(|m| m.id == module_id) {
            module.enabled = enabled;
        }
        self.storage.save_modules(&self.modules);
        self.inject_theme_into_all_tabs();
        self.sync_ui_state();
    }

    pub fn set_theme(&mut self, theme: String) {
        self.settings.theme = theme;
        self.storage.save_settings(&self.settings);

        #[cfg(target_os = "windows")]
        {
            let (r, g, b, _) = self.get_theme_background_color();
            crate::drag_util::apply_dark_window_attributes(&self.window, (r, g, b));
        }

        self.inject_theme_into_all_tabs();
        self.sync_ui_state();
    }

    pub fn set_accent_color(&mut self, color: String) {
        self.settings.accent_color = color;
        self.storage.save_settings(&self.settings);
        self.sync_ui_state();
    }

    pub fn set_search_engine(&mut self, engine: String) {
        self.settings.search_engine = engine;
        self.storage.save_settings(&self.settings);
        self.sync_ui_state();
    }

    pub fn set_show_bookmarks_bar(&mut self, show: bool) {
        self.settings.show_bookmarks_bar = show;
        self.storage.save_settings(&self.settings);
        self.update_header_layout();
        self.sync_ui_state();
    }

    pub fn update_header_layout(&mut self) {
        #[cfg(desktop)]
        {
            let (width, _) = self.get_current_window_size();
            let header_height = self.get_header_height();

            if let Some(ref header) = self.header_webview {
                let _ = header.set_size(Size::Logical(LogicalSize::new(width, header_height)));
            }

            if let Some(active_id) = self.active_tab_id {
                if let Some(tab) = self.tabs.iter().find(|t| t.id == active_id) {
                    let (pos, size) = self.get_content_position_and_size();
                    let _ = tab.webview.set_position(Position::Logical(pos));
                    let _ = tab.webview.set_size(Size::Logical(size));
                }
            }
        }
    }

    pub fn resize(&mut self, width: f64, height: f64) {
        if width <= 10.0 || height <= 10.0 {
            return;
        }
        self.window_size = (width, height);

        #[cfg(desktop)]
        {
            let header_height = self.get_header_height();

            if let Some(ref header) = self.header_webview {
                let _ = header.set_size(Size::Logical(LogicalSize::new(width, header_height)));
            }

            if let Some(active_id) = self.active_tab_id {
                if let Some(tab) = self.tabs.iter().find(|t| t.id == active_id) {
                    let content_height = (height - header_height).max(10.0);
                    let _ = tab
                        .webview
                        .set_position(Position::Logical(LogicalPosition::new(0.0, header_height)));
                    let _ = tab
                        .webview
                        .set_size(Size::Logical(LogicalSize::new(width, content_height)));
                }
            }
        }
    }

    pub fn on_page_load_event(&mut self, tab_id: u32, url: String, finished: bool) {
        let script = self.get_theme_injection_script();

        if let Some(tab) = self.tabs.iter_mut().find(|t| t.id == tab_id) {
            tab.url = url.clone();
            tab.is_loading = !finished;

            if finished {
                // Determine title from URL if empty
                if tab.title == "New Tab" || tab.title.is_empty() {
                    if url.contains("youtube.com") {
                        tab.title = "YouTube".into();
                    } else if let Ok(parsed) = url::Url::parse(&url) {
                        if let Some(host) = parsed.host_str() {
                            tab.title = host.replace("www.", "");
                        }
                    }
                }

                // Inject theme script upon load finish
                #[cfg(desktop)]
                {
                    let _ = tab.webview.eval(&script);
                    let _ = tab.webview.eval(
                        r#"
                        (function() {
                            if (window.ipc) {
                                window.ipc.postMessage(JSON.stringify({
                                    type: "TabStateUpdate",
                                    url: window.location.href,
                                    title: document.title || window.location.hostname,
                                    can_go_back: window.history.length > 1,
                                    can_go_forward: false
                                }));
                            }
                        })();
                        "#,
                    );
                }

                #[cfg(not(desktop))]
                if let Some(win) = self.app.get_webview_window("main") {
                    let _ = win.eval(&script);
                }
            }
        }
        self.sync_ui_state();
    }

    pub fn get_state(&self) -> IpcBrowserState {
        let tabs_info: Vec<IpcTabInfo> = self
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
            .collect();

        #[cfg(desktop)]
        let is_maximized = self.window.is_maximized().unwrap_or(false);

        #[cfg(not(desktop))]
        let is_maximized = false;

        IpcBrowserState {
            tabs: tabs_info,
            active_tab_id: self.active_tab_id,
            bookmarks: self.bookmarks.clone(),
            modules: self.modules.clone(),
            settings: self.settings.clone(),
            zoom: self.zoom,
            search_engine: self.settings.search_engine.clone(),
            is_maximized,
        }
    }

    pub fn sync_ui_state(&self) {
        let state = self.get_state();
        let _ = self.app.emit("browser-state-update", &state);

        #[cfg(desktop)]
        if let Some(ref header) = self.header_webview {
            if let Ok(json) = serde_json::to_string(&state) {
                let js = format!("window.updateBrowserState && window.updateBrowserState({});", json);
                let _ = header.eval(&js);
            }
        }
    }

    pub fn handle_incoming_ipc(&mut self, raw: &str) {
        let incoming: Result<IpcIncoming, _> = serde_json::from_str(raw);
        let msg = match incoming {
            Ok(m) => m,
            Err(_) => return,
        };

        match msg {
            IpcIncoming::UiReady => {
                self.sync_ui_state();
            }
            IpcIncoming::NewTab { url } => {
                let initial = url.as_deref().unwrap_or("https://www.youtube.com");
                self.create_tab(initial);
            }
            IpcIncoming::CloseTab { tab_id } => {
                self.close_tab(tab_id);
            }
            IpcIncoming::SwitchTab { tab_id } => {
                self.switch_tab(tab_id);
            }
            IpcIncoming::Navigate { url } => {
                self.navigate(&url);
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
                self.remove_bookmark(url);
            }
            IpcIncoming::ShowBookmarkContextMenu { url } => {
                #[cfg(target_os = "windows")]
                {
                    if let Some(action) =
                        crate::menu_util::show_native_bookmark_context_menu(&self.window)
                    {
                        match action {
                            1 => {
                                self.create_tab(&url);
                            }
                            2 => {
                                crate::menu_util::copy_to_clipboard(&url);
                            }
                            3 => {
                                self.remove_bookmark(url);
                            }
                            _ => (),
                        }
                    }
                }
                #[cfg(not(target_os = "windows"))]
                {
                    let _ = url;
                }
            }
            IpcIncoming::ToggleModule { module_id, enabled } => {
                self.toggle_module(module_id, enabled);
            }
            IpcIncoming::SetTheme { theme } => {
                self.set_theme(theme);
            }
            IpcIncoming::SetAccentColor { color } => {
                self.set_accent_color(color);
            }
            IpcIncoming::SetSearchEngine { engine } => {
                self.set_search_engine(engine);
            }
            IpcIncoming::SetShowBookmarksBar { show } => {
                self.set_show_bookmarks_bar(show);
            }
            IpcIncoming::OpenSettings => {
                self.create_tab("titan://settings");
            }
            IpcIncoming::OpenThemes => {
                self.create_tab("titan://themes");
            }
            IpcIncoming::TabStateUpdate {
                tab_id,
                url,
                title,
                can_go_back,
                can_go_forward,
            } => {
                let target_id = tab_id.or(self.active_tab_id);
                if let Some(id) = target_id {
                    if let Some(tab) = self.tabs.iter_mut().find(|t| t.id == id) {
                        tab.url = url;
                        if !title.is_empty() {
                            tab.title = title;
                        }
                        if let Some(back) = can_go_back {
                            tab.can_go_back = back;
                        }
                        if let Some(fwd) = can_go_forward {
                            tab.can_go_forward = fwd;
                        }
                    }
                    self.sync_ui_state();
                }
            }
            IpcIncoming::DragWindow => {
                #[cfg(desktop)]
                {
                    let _ = self.window.start_dragging();
                }
            }
            IpcIncoming::MinimizeWindow => {
                #[cfg(desktop)]
                {
                    let _ = self.window.minimize();
                }
            }
            IpcIncoming::ToggleMaximizeWindow => {
                #[cfg(desktop)]
                if let Ok(is_max) = self.window.is_maximized() {
                    if is_max {
                        let _ = self.window.unmaximize();
                    } else {
                        let _ = self.window.maximize();
                    }
                }
            }
            IpcIncoming::CloseWindow => {
                let _ = self.window.close();
            }
        }
    }
}
