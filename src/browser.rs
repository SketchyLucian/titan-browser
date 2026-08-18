use crate::ipc::{Bookmark, BrowserModule, IpcBrowserState, IpcIncoming, IpcTabInfo};
use crate::storage::StorageManager;
use crate::url_utils::normalize_or_search_url;
use std::sync::Arc;
use tao::dpi::{LogicalPosition, LogicalSize};
use tao::event_loop::EventLoopProxy;
use tao::window::Window;
use wry::{PageLoadEvent, Rect, WebView, WebViewBuilder};

pub const HEADER_HEIGHT: f64 = 102.0;

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
    pub zoom: f64,
    pub window_size: (f64, f64),
}

impl BrowserManager {
    pub fn new(window: Arc<Window>, proxy: EventLoopProxy<UserEvent>) -> Self {
        let storage = StorageManager::new();
        let bookmarks = storage.load_bookmarks();
        let modules = storage.load_modules();
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
            zoom: 1.0,
            window_size: (win_size.width, win_size.height),
        }
    }

    pub fn init(&mut self) {
        let (width, _) = self.window_size;
        let header_bounds = Rect {
            position: LogicalPosition::new(0.0, 0.0).into(),
            size: LogicalSize::new(width, HEADER_HEIGHT).into(),
        };

        let proxy_clone = self.proxy.clone();
        let html_content = Self::get_chrome_html();

        let header = WebViewBuilder::new()
            .with_bounds(header_bounds)
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
        let js = include_str!("../ui/app.js");

        html.replace(
            "<link rel=\"stylesheet\" href=\"style.css\" />",
            &format!("<style>{}</style>", css),
        )
        .replace(
            "<script src=\"app.js\"></script>",
            &format!("<script>{}</script>", js),
        )
    }

    pub fn get_modules_dashboard_html(modules: &[BrowserModule]) -> String {
        let mut cards_html = String::new();
        for m in modules {
            let checked = if m.enabled { "checked" } else { "" };
            let icon_svg = "<svg viewBox='0 0 24 24' width='22' height='22' stroke='currentColor' stroke-width='2' fill='none'><path d='M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z'></path></svg>";

            cards_html.push_str(&format!(
                r#"
                <div class="module-card">
                    <div class="module-left">
                        <div class="module-icon-box">{icon_svg}</div>
                        <div class="module-text">
                            <div class="module-title-row">
                                <span class="module-name">{name}</span>
                            </div>
                            <div class="module-desc">{desc}</div>
                        </div>
                    </div>
                    <label class="switch">
                        <input type="checkbox" onchange="toggleModule('{id}', this.checked)" {checked}>
                        <span class="slider"></span>
                    </label>
                </div>
                "#,
                icon_svg = icon_svg,
                name = m.name,
                desc = m.description,
                id = m.id,
                checked = checked
            ));
        }

        format!(
            r#"<!DOCTYPE html>
            <html>
            <head>
                <meta charset="UTF-8">
                <title>Titan Browser Settings</title>
                <style>
                    * {{ margin: 0; padding: 0; box-sizing: border-box; font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif; user-select: none; }}
                    body {{ background: #0f1015; color: #f0f2f8; padding: 40px 20px; min-height: 100vh; }}
                    .container {{ max-width: 680px; margin: 0 auto; }}
                    .header-box {{ display: flex; align-items: center; justify-content: space-between; margin-bottom: 24px; padding-bottom: 18px; border-bottom: 1px solid #232634; }}
                    .header-left {{ display: flex; align-items: center; gap: 12px; }}
                    .header-icon {{ width: 38px; height: 38px; color: #38bdf8; background: rgba(56, 189, 248, 0.12); border-radius: 8px; display: flex; align-items: center; justify-content: center; }}
                    .header-title {{ font-size: 20px; font-weight: 700; color: #ffffff; letter-spacing: 0.3px; }}
                    .header-subtitle {{ font-size: 12.5px; color: #8d92a6; margin-top: 2px; }}
                    .stats-pill {{ background: rgba(56, 189, 248, 0.12); color: #38bdf8; border: 1px solid rgba(56, 189, 248, 0.25); padding: 5px 12px; border-radius: 20px; font-size: 12px; font-weight: 600; }}
                    .module-list {{ display: flex; flex-direction: column; gap: 12px; }}
                    .module-card {{ display: flex; align-items: center; justify-content: space-between; background: #181a23; border: 1px solid #282c3c; border-radius: 10px; padding: 18px 22px; transition: all 0.2s ease; }}
                    .module-card:hover {{ background: #1f222e; border-color: #3d425b; }}
                    .module-left {{ display: flex; align-items: center; gap: 16px; flex: 1; }}
                    .module-icon-box {{ width: 44px; height: 44px; border-radius: 10px; background: #13141b; border: 1px solid #252838; display: flex; align-items: center; justify-content: center; color: #38bdf8; }}
                    .module-title-row {{ display: flex; align-items: center; gap: 8px; margin-bottom: 4px; }}
                    .module-name {{ font-size: 15px; font-weight: 600; color: #ffffff; }}
                    .module-desc {{ font-size: 13px; color: #9499ad; line-height: 1.4; }}
                    .switch {{ position: relative; display: inline-block; width: 44px; height: 24px; flex-shrink: 0; }}
                    .switch input {{ opacity: 0; width: 0; height: 0; }}
                    .slider {{ position: absolute; cursor: pointer; top: 0; left: 0; right: 0; bottom: 0; background-color: #313547; transition: 0.25s; border-radius: 24px; }}
                    .slider:before {{ position: absolute; content: ""; height: 18px; width: 18px; left: 3px; bottom: 3px; background-color: white; transition: 0.25s; border-radius: 50%; }}
                    input:checked + .slider {{ background-color: #4e7cf6; }}
                    input:checked + .slider:before {{ transform: translateX(20px); }}
                    .footer-note {{ margin-top: 30px; text-align: center; font-size: 11.5px; color: #5d6175; }}
                </style>
            </head>
            <body>
                <div class="container">
                    <div class="header-box">
                        <div class="header-left">
                            <div class="header-icon">
                                <svg viewBox="0 0 24 24" width="22" height="22" stroke="currentColor" stroke-width="2" fill="none">
                                    <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"></path>
                                </svg>
                            </div>
                            <div>
                                <div class="header-title">Universal Dark Mode</div>
                                <div class="header-subtitle">Toggle smart dark theme across all websites.</div>
                            </div>
                        </div>
                        <div class="stats-pill">⚡ Native Filter</div>
                    </div>

                    <div class="module-list">
                        {cards_html}
                    </div>

                    <div class="footer-note">
                        Titan Browser &bull; Minimalist, Fast, and Focused.
                    </div>
                </div>

                <script>
                    function toggleModule(id, enabled) {{
                        if (window.ipc && window.ipc.postMessage) {{
                            window.ipc.postMessage(JSON.stringify({{
                                type: 'ToggleModule',
                                module_id: id,
                                enabled: enabled
                            }}));
                        }}
                    }}
                </script>
            </body>
            </html>"#,
            cards_html = cards_html
        )
    }

    fn get_content_bounds(&self) -> Rect {
        let (width, height) = self.window_size;
        let content_height = (height - HEADER_HEIGHT).max(10.0);
        Rect {
            position: LogicalPosition::new(0.0, HEADER_HEIGHT).into(),
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

    pub fn create_tab(&mut self, target_url: &str) -> u32 {
        let tab_id = self.next_tab_id;
        self.next_tab_id += 1;

        let is_modules_page = target_url == "titan://modules" || target_url == "titan://darkmode" || target_url == "titan://settings";
        let normalized_url = if is_modules_page {
            "titan://modules".to_string()
        } else {
            normalize_or_search_url(target_url)
        };

        let bounds = self.get_content_bounds();

        let proxy_ipc = self.proxy.clone();
        let proxy_load = self.proxy.clone();
        let tab_id_copy = tab_id;

        let darkmode_enabled = self.is_module_enabled("dark_reader");

        // Clean, lightweight observer script with 0 video tampering
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
                    }}, 250);
                }}

                const titleEl = document.querySelector('title');
                if (titleEl) {{
                    new MutationObserver(notify).observe(titleEl, {{ subtree: true, characterData: true, childList: true }});
                }}
                window.addEventListener('popstate', notify);
                window.addEventListener('load', notify);
                setTimeout(notify, 500);

                const darkActive = {darkmode_enabled};
                if (darkActive) {{
                    document.addEventListener('DOMContentLoaded', () => {{
                        let el = document.getElementById('titan-dark-reader-style');
                        if (!el) {{
                            el = document.createElement('style');
                            el.id = 'titan-dark-reader-style';
                            el.textContent = 'html {{ filter: invert(90%) hue-rotate(180deg) !important; background: #111 !important; }} img, video, iframe, canvas, svg, [style*="background-image"] {{ filter: invert(100%) hue-rotate(180deg) !important; }}';
                            document.documentElement.appendChild(el);
                        }}
                    }});
                }}
            }})();
            "#
        );

        let builder = WebViewBuilder::new()
            .with_bounds(bounds)
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

        let modules_html = if is_modules_page {
            Some(Self::get_modules_dashboard_html(&self.modules))
        } else {
            None
        };

        let webview = if let Some(ref html) = modules_html {
            builder.with_html(html)
        } else {
            builder.with_url(&normalized_url)
        }
        .build(&*self.window)
        .expect("Failed to create content webview for tab");

        let default_title = if is_modules_page {
            "Universal Dark Mode".to_string()
        } else if normalized_url.contains("youtube.com") {
            "YouTube".to_string()
        } else {
            "New Tab".to_string()
        };

        let new_tab = Tab {
            id: tab_id,
            url: normalized_url,
            title: default_title,
            is_loading: !is_modules_page,
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

        // 1. Hide all other tabs first to avoid focus conflicts
        for tab in &mut self.tabs {
            if tab.id != target_id {
                let _ = tab.webview.set_visible(false);
            }
        }

        // 2. Show, reposition and focus the target active tab
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
        if input == "titan://modules" || input == "titan://darkmode" || input == "titan://settings" {
            if let Some(active_id) = self.active_tab_id {
                if let Some(tab) = self.tabs.iter_mut().find(|t| t.id == active_id) {
                    tab.url = "titan://modules".into();
                    tab.title = "Universal Dark Mode".into();
                    tab.is_loading = false;
                    let html = Self::get_modules_dashboard_html(&self.modules);
                    let _ = tab.webview.load_html(&html);
                }
                self.sync_tab_update(active_id);
            }
            return;
        }

        let normalized = normalize_or_search_url(input);
        if let Some(active_id) = self.active_tab_id {
            if let Some(tab) = self.tabs.iter_mut().find(|t| t.id == active_id) {
                tab.url = normalized.clone();
                tab.is_loading = true;
                let _ = tab.webview.load_url(&normalized);
            }
            self.sync_tab_update(active_id);
        }
    }

    pub fn go_back(&mut self) {
        if let Some(active_id) = self.active_tab_id {
            if let Some(tab) = self.tabs.iter().find(|t| t.id == active_id) {
                let _ = tab.webview.evaluate_script("window.history.back();");
            }
        }
    }

    pub fn go_forward(&mut self) {
        if let Some(active_id) = self.active_tab_id {
            if let Some(tab) = self.tabs.iter().find(|t| t.id == active_id) {
                let _ = tab.webview.evaluate_script("window.history.forward();");
            }
        }
    }

    pub fn reload(&mut self) {
        if let Some(active_id) = self.active_tab_id {
            if let Some(tab) = self.tabs.iter().find(|t| t.id == active_id) {
                if tab.url == "titan://modules" {
                    let html = Self::get_modules_dashboard_html(&self.modules);
                    let _ = tab.webview.load_html(&html);
                } else {
                    let _ = tab.webview.evaluate_script("window.location.reload();");
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
        self.sync_full_state();
    }

    pub fn remove_bookmark(&mut self, url: &str) {
        self.bookmarks.retain(|b| b.url != url);
        self.storage.save_bookmarks(&self.bookmarks);
        self.sync_full_state();
    }

    pub fn toggle_module(&mut self, module_id: &str, enabled: bool) {
        if let Some(module) = self.modules.iter_mut().find(|m| m.id == module_id) {
            module.enabled = enabled;
        }
        self.storage.save_modules(&self.modules);

        // Re-render any open titan://modules tab
        let modules_html = Self::get_modules_dashboard_html(&self.modules);
        for tab in &self.tabs {
            if tab.url == "titan://modules" {
                let _ = tab.webview.load_html(&modules_html);
            }
        }

        // Apply dark mode filter dynamically to all open web content tabs
        if module_id == "dark_reader" {
            let script = if enabled {
                r#"
                (function() {
                    let el = document.getElementById('titan-dark-reader-style');
                    if (!el) {
                        el = document.createElement('style');
                        el.id = 'titan-dark-reader-style';
                        el.textContent = 'html { filter: invert(90%) hue-rotate(180deg) !important; background: #111 !important; } img, video, iframe, canvas, svg, [style*="background-image"] { filter: invert(100%) hue-rotate(180deg) !important; }';
                        document.documentElement.appendChild(el);
                    }
                })();
                "#
            } else {
                r#"
                (function() {
                    const el = document.getElementById('titan-dark-reader-style');
                    if (el) el.remove();
                })();
                "#
            };

            for tab in &self.tabs {
                if tab.url != "titan://modules" {
                    let _ = tab.webview.evaluate_script(script);
                }
            }
        }

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
        let dark_mode_active = self.is_module_enabled("dark_reader");

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

            // If dark reader is enabled, inject into finished page
            if dark_mode_active && tab.url != "titan://modules" {
                let dark_script = r#"
                    (function() {
                        let el = document.getElementById('titan-dark-reader-style');
                        if (!el) {
                            el = document.createElement('style');
                            el.id = 'titan-dark-reader-style';
                            el.textContent = 'html { filter: invert(90%) hue-rotate(180deg) !important; background: #111 !important; } img, video, iframe, canvas, svg, [style*="background-image"] { filter: invert(100%) hue-rotate(180deg) !important; }';
                            document.documentElement.appendChild(el);
                        }
                    })();
                "#;
                let _ = tab.webview.evaluate_script(dark_script);
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

        let header_bounds = Rect {
            position: LogicalPosition::new(0.0, 0.0).into(),
            size: LogicalSize::new(width, HEADER_HEIGHT).into(),
        };

        if let Some(header) = &self.header_webview {
            let _ = header.set_bounds(header_bounds);
        }

        let content_bounds = self.get_content_bounds();
        if let Some(active_id) = self.active_tab_id {
            if let Some(tab) = self.tabs.iter_mut().find(|t| t.id == active_id) {
                let _ = tab.webview.set_bounds(content_bounds);
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
                zoom: self.zoom,
                search_engine: "Google".into(),
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
