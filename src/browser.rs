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
    pub blocked_logs: Vec<crate::ipc::BlockedRequestLog>,
    pub adblock_logs: Vec<crate::ipc::BlockedRequestLog>,
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
            blocked_logs: Vec::new(),
            adblock_logs: Vec::new(),
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
        let html_themed = html.replace("class=\"theme-titan-dark\"", &format!("class=\"{}\"", theme_class));
        let state_json = serde_json::to_string(&serde_json::json!({
            "settings": self.settings,
            "modules": self.modules,
            "blocked_logs": self.blocked_logs,
            "adblock_logs": self.adblock_logs,
            "active_section": active_section,
        }))
        .unwrap_or_else(|_| "{}".into());

        html_themed.replace(
            "<script src=\"settings.js\"></script>",
            &format!("<script>{}</script><script>(function(){{ function run(){{ window.initSettings && window.initSettings({}); }} if (document.readyState === 'loading') {{ window.addEventListener('DOMContentLoaded', run); }} else {{ run(); }} }})();</script>", js, state_json)
        )
    }

    pub fn get_newtab_html(&self) -> String {
        let html = include_str!("../ui/newtab.html");
        let js = include_str!("../ui/dist/newtab.js");
        let theme_class = format!("theme-{}", self.settings.theme);
        let html_themed = html.replace("class=\"theme-titan-dark\"", &format!("class=\"{}\"", theme_class));
        let state_json = serde_json::to_string(&serde_json::json!({
            "theme": self.settings.theme,
            "accent_color": self.settings.accent_color,
            "search_engine": self.settings.search_engine,
        }))
        .unwrap_or_else(|_| "{}".into());

        html_themed.replace(
            "<script src=\"newtab.js\"></script>",
            &format!("<script>{}</script><script>(function(){{ function run(){{ window.initNewTab && window.initNewTab({}); }} if (document.readyState === 'loading') {{ window.addEventListener('DOMContentLoaded', run); }} else {{ run(); }} }})();</script>", js, state_json)
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
        let is_light = self.settings.theme == "titan-light";
        let force_adaptation = self.is_module_enabled("dark_reader");

        format!(
            r#"
            (function() {{
                const isLight = {is_light};
                const forceAdaptation = {force_adaptation};
                const targetMode = isLight ? 'light' : 'dark';
                const removeMode = isLight ? 'dark' : 'light';
                const host = (window.location.hostname || '').toLowerCase();
                const href = (window.location.href || '').toLowerCase();

                // If on internal / blank URL, stop here
                if (!host || href.startsWith('titan://') || href.startsWith('about:')) return;

                // 1. Clean up any previous forced styles if adaptation is off
                try {{
                    const preCanvas = document.getElementById('titan-pre-dark-canvas');
                    if (preCanvas) preCanvas.remove();
                    const adaptStyle = document.getElementById('titan-theme-adaptation-style');
                    if (!forceAdaptation && adaptStyle) adaptStyle.remove();
                }} catch(e) {{}}

                // 2. Document Root color-scheme & Meta Tag
                try {{
                    const applyColorScheme = () => {{
                        if (document.documentElement) {{
                            document.documentElement.style.colorScheme = targetMode;
                        }}
                    }};
                    applyColorScheme();
                    if (document.readyState === 'loading') {{
                        document.addEventListener('DOMContentLoaded', applyColorScheme, {{ once: true }});
                    }}
                }} catch(e) {{}}

                // 3. Framework & Library Integration
                function applyFrameworkThemes() {{
                    try {{
                        const html = document.documentElement;
                        const body = document.body;

                        if (html) {{
                            html.classList.remove(removeMode);
                            html.classList.add(targetMode);

                            ['data-theme', 'data-color-mode', 'data-bs-theme', 'data-mode', 'data-theme-mode'].forEach(attr => {{
                                if (html.hasAttribute(attr)) {{
                                    html.setAttribute(attr, targetMode);
                                }}
                            }});
                        }}

                        if (body) {{
                            if (body.classList.contains('dark') || body.classList.contains('light')) {{
                                body.classList.remove(removeMode);
                                body.classList.add(targetMode);
                            }}
                            ['data-theme', 'data-color-mode', 'data-bs-theme', 'data-mode'].forEach(attr => {{
                                if (body.hasAttribute(attr)) {{
                                    body.setAttribute(attr, targetMode);
                                }}
                            }});
                        }}
                    }} catch(e) {{}}
                }}

                applyFrameworkThemes();
                if (document.readyState === 'loading') {{
                    document.addEventListener('DOMContentLoaded', applyFrameworkThemes, {{ once: true }});
                }}

                // 4. Major Website Adaptations
                try {{
                    const isYouTube = host.includes('youtube.com') || host.includes('youtu.be');
                    if (isYouTube && document.documentElement) {{
                        if (isLight) {{
                            document.documentElement.removeAttribute('dark');
                        }} else {{
                            document.documentElement.setAttribute('dark', 'true');
                        }}
                    }}

                    const isWikipedia = host.includes('wikipedia.org');
                    if (isWikipedia && document.documentElement) {{
                        document.documentElement.classList.remove(isLight ? 'skin-theme-clientpref-night' : 'skin-theme-clientpref-day');
                        document.documentElement.classList.add(isLight ? 'skin-theme-clientpref-day' : 'skin-theme-clientpref-night');
                    }}
                }} catch(e) {{}}

                // 5. Universal Webpage Theme Adaptation (Dark Reader) for light-only sites
                if (forceAdaptation) {{
                    function adaptTheme() {{
                        const isNativelyAdaptive = host.includes('google.') || host.includes('youtube.com') || host.includes('youtu.be') || host.includes('github.com') || host.includes('gitlab.com') || host.includes('reddit.com') || host.includes('duckduckgo.com') || host.includes('bing.com') || host.includes('x.com') || host.includes('twitter.com') || host.includes('tauri.app') || host.includes('vitepress') || host.includes('docusaurus');
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

    pub fn get_privacy_injection_script(&self) -> String {
        let dnt = self.settings.do_not_track;
        let gpc = self.settings.global_privacy_control;
        let block_webrtc = self.settings.block_webrtc_leak;

        format!(
            r#"
            (function() {{
                try {{
                    const dnt = {dnt};
                    const gpc = {gpc};
                    const blockWebrtc = {block_webrtc};

                    if (dnt) {{
                        try {{
                            Object.defineProperty(navigator, 'doNotTrack', {{ get: () => '1', configurable: true }});
                            Object.defineProperty(window, 'doNotTrack', {{ get: () => '1', configurable: true }});
                        }} catch(e) {{}}
                    }}

                    if (gpc) {{
                        try {{
                            Object.defineProperty(navigator, 'globalPrivacyControl', {{ get: () => true, configurable: true }});
                        }} catch(e) {{}}
                    }}

                    if (blockWebrtc) {{
                        try {{
                            if (window.RTCPeerConnection) {{
                                const origSetLocalDesc = window.RTCPeerConnection.prototype.setLocalDescription;
                                if (origSetLocalDesc) {{
                                    window.RTCPeerConnection.prototype.setLocalDescription = function(desc) {{
                                        if (desc && desc.sdp) {{
                                            desc.sdp = desc.sdp.replace(/a=candidate:.+typ host .+\r\n/g, '');
                                        }}
                                        return origSetLocalDesc.call(this, desc);
                                    }};
                                }}
                            }}
                        }} catch(e) {{}}
                    }}
                }} catch(e) {{}}
            }})();
            "#,
            dnt = dnt,
            gpc = gpc,
            block_webrtc = block_webrtc,
        )
    }

    pub fn get_adblock_injection_script(&self) -> String {
        let enabled = self.settings.adblock_enabled;
        let block_video_ads = self.settings.adblock_block_video_ads;
        let cosmetic_filtering = self.settings.adblock_cosmetic_filtering;
        let whitelisted_domains_json = serde_json::to_string(&self.settings.adblock_whitelisted_domains)
            .unwrap_or_else(|_| "[]".into());

        format!(
            r#"
            (function() {{
                try {{
                    const enabled = {enabled};
                    if (!enabled) return;

                    const blockVideoAds = {block_video_ads};
                    const cosmeticFiltering = {cosmetic_filtering};
                    const AD_WHITELIST = {whitelisted_domains_json};

                    const currentHost = (window.location.hostname || '').toLowerCase();
                    const currentHref = (window.location.href || '').toLowerCase();

                    if (!currentHost || currentHref.startsWith('titan://') || currentHref.startsWith('about:')) return;

                    // Check if current website is whitelisted
                    if (AD_WHITELIST.some(d => d && (currentHost === d.toLowerCase() || currentHost.endsWith('.' + d.toLowerCase())))) {{
                        return; // Ad blocking disabled for this whitelisted site
                    }}

                    // 1. Cosmetic Element Hiding (CSS rules)
                    if (cosmeticFiltering) {{
                        const adCss = `
                            ins.adsbygoogle,
                            [id^="google_ads_"],
                            [id*="google_ads_iframe"],
                            [id*="ScriptRoot"],
                            [class*="sponsored-post"],
                            [class*="ad-container"],
                            [class*="ad_container"],
                            [id*="banner-ad"],
                            [id*="ad-banner"],
                            [class*="ad-banner"],
                            [class*="ad-wrapper"],
                            [id*="ad-wrapper"],
                            [class*="ad-slot"],
                            [id*="ad-slot"],
                            [class*="ad-placement"],
                            [aria-label="advertisement"],
                            ytd-promoted-video-renderer,
                            ytd-promoted-sparkles-web-renderer,
                            ytd-display-ad-renderer,
                            ytd-statement-banner-renderer,
                            ytd-in-feed-ad-layout-renderer,
                            ytd-banner-promo-renderer,
                            #masthead-ad,
                            #player-ads,
                            #offer-module,
                            .ytp-ad-overlay-container,
                            .ytp-ad-message-container,
                            .ytp-ad-overlay-slot,
                            .ytp-ad-action-interstitial,
                            .video-ads,
                            .ytp-ad-module {{
                                display: none !important;
                                visibility: hidden !important;
                                height: 0 !important;
                                min-height: 0 !important;
                                max-height: 0 !important;
                                width: 0 !important;
                                opacity: 0 !important;
                                pointer-events: none !important;
                                overflow: hidden !important;
                            }}
                        `;

                        function injectAdStyle() {{
                            if (document.getElementById('titan-adblock-style')) return;
                            const style = document.createElement('style');
                            style.id = 'titan-adblock-style';
                            style.textContent = adCss;
                            (document.head || document.documentElement).appendChild(style);
                        }}

                        injectAdStyle();
                        if (document.readyState === 'loading') {{
                            document.addEventListener('DOMContentLoaded', injectAdStyle, {{ once: true }});
                        }}
                    }}

                    // 2. Video Ad Auto-Skipper & Fast-Forward (Only active on YouTube)
                    if (blockVideoAds && (currentHost.includes('youtube.com') || currentHost.includes('youtu.be'))) {{
                        function handleVideoAds() {{
                            try {{
                                const skipSelectors = [
                                    '.ytp-ad-skip-button',
                                    '.ytp-ad-skip-button-modern',
                                    '.ytp-skip-ad-button',
                                    '.ytp-ad-skip-button-slot',
                                    '.ytp-ad-overlay-close-button',
                                    '.videoAdUiSkipButton',
                                    '[id^="skip-button"]',
                                    '.ytp-ad-text.ytp-ad-preview-text'
                                ];

                                for (const sel of skipSelectors) {{
                                    const btn = document.querySelector(sel);
                                    if (btn) {{
                                        btn.click();
                                    }}
                                }}

                                const adElements = document.querySelectorAll('.ad-showing, .ad-interrupting, .ytp-ad-player-overlay');
                                if (adElements.length > 0) {{
                                    const videos = document.querySelectorAll('video');
                                    videos.forEach(v => {{
                                        if (v && !isNaN(v.duration) && v.duration > 0) {{
                                            v.muted = true;
                                            v.playbackRate = 16.0;
                                            v.currentTime = v.duration;
                                        }}
                                    }});
                                }}
                            }} catch(e) {{}}
                        }}

                        setInterval(handleVideoAds, 350);
                    }}
                }} catch(e) {{}}
            }})();
            "#,
            enabled = enabled,
            block_video_ads = block_video_ads,
            cosmetic_filtering = cosmetic_filtering,
            whitelisted_domains_json = whitelisted_domains_json,
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
        let adblock_script = self.get_adblock_injection_script();

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
                {privacy_script}
                {adblock_script}
            }})();
            "#,
            tab_id = tab_id,
            theme_script = theme_script,
            privacy_script = privacy_script,
            adblock_script = adblock_script,
        );

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

        let normalized = normalize_or_search_url_with_engine(&clean_input, &self.settings.search_engine);
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
        if let Some(active_id) = self.active_tab_id {
            if let Some(tab) = self.tabs.iter_mut().find(|t| t.id == active_id) {
                let _ = tab.webview.evaluate_script("window.history.back();");
            }
            if let Some(tab) = self.tabs.iter().find(|t| t.id == active_id) {
                let _ = tab.webview.focus();
            }
        }
    }

    pub fn go_forward(&mut self) {
        if let Some(active_id) = self.active_tab_id {
            if let Some(tab) = self.tabs.iter_mut().find(|t| t.id == active_id) {
                let _ = tab.webview.evaluate_script("window.history.forward();");
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
                if let Some(tab) = self.tabs.iter_mut().find(|t| t.id == active_id) {
                    let _ = tab.webview.evaluate_script("window.location.reload();");
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
        for tab in &self.tabs {
            let script = format!("document.body.style.zoom = '{}';", zoom_val);
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
        let state_json = serde_json::to_string(&serde_json::json!({
            "settings": self.settings,
            "modules": self.modules,
            "blocked_logs": self.blocked_logs,
            "adblock_logs": self.adblock_logs,
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
        let is_dark_reader = self.is_module_enabled("dark_reader");
        let theme_script = if is_dark_reader {
            Some(self.get_theme_injection_script())
        } else {
            None
        };

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

            // Inject matching theme script dynamically if dark reader is forced
            if !Self::is_internal_url(&tab.url) {
                if let Some(ref script) = theme_script {
                    let _ = tab.webview.evaluate_script(script);
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
                        "strip_tracking_parameters" => self.settings.strip_tracking_parameters = enabled,
                        "block_webrtc_leak" => self.settings.block_webrtc_leak = enabled,
                        "block_fingerprinting" => self.settings.block_fingerprinting = enabled,
                        "block_hyperlink_auditing" => self.settings.block_hyperlink_auditing = enabled,
                        "telemetry_disabled" => self.settings.telemetry_disabled = enabled,
                        _ => {}
                    }
                    self.storage.save_settings(&self.settings);
                    self.sync_settings_tabs();
                    self.sync_full_state();
                }
                IpcIncoming::SetAdblockSetting { key, enabled } => {
                    match key.as_str() {
                        "adblock_enabled" => self.settings.adblock_enabled = enabled,
                        "adblock_block_video_ads" => self.settings.adblock_block_video_ads = enabled,
                        "adblock_cosmetic_filtering" => self.settings.adblock_cosmetic_filtering = enabled,
                        "adblock_block_popups" => self.settings.adblock_block_popups = enabled,
                        "adblock_aggressive_mode" => self.settings.adblock_aggressive_mode = enabled,
                        _ => {}
                    }
                    self.storage.save_settings(&self.settings);
                    self.sync_settings_tabs();
                    self.sync_full_state();
                }
                IpcIncoming::AddBlockedDomain { domain } => {
                    let d = domain.trim().to_lowercase();
                    if !d.is_empty() && !self.settings.blocked_domains.contains(&d) {
                        self.settings.blocked_domains.push(d);
                        self.storage.save_settings(&self.settings);
                        self.sync_settings_tabs();
                    }
                }
                IpcIncoming::RemoveBlockedDomain { domain } => {
                    let d = domain.trim().to_lowercase();
                    self.settings.blocked_domains.retain(|item| item != &d);
                    self.storage.save_settings(&self.settings);
                    self.sync_settings_tabs();
                }
                IpcIncoming::AddWhitelistedDomain { domain } => {
                    let d = domain.trim().to_lowercase();
                    if !d.is_empty() && !self.settings.whitelisted_domains.contains(&d) {
                        self.settings.whitelisted_domains.push(d);
                        self.storage.save_settings(&self.settings);
                        self.sync_settings_tabs();
                    }
                }
                IpcIncoming::RemoveWhitelistedDomain { domain } => {
                    let d = domain.trim().to_lowercase();
                    self.settings.whitelisted_domains.retain(|item| item != &d);
                    self.storage.save_settings(&self.settings);
                    self.sync_settings_tabs();
                }
                IpcIncoming::ResetPrivacyRules => {
                    self.settings.blocked_domains = crate::ipc::default_blocked_domains();
                    self.settings.whitelisted_domains.clear();
                    self.storage.save_settings(&self.settings);
                    self.sync_settings_tabs();
                }
                IpcIncoming::AddAdblockDomain { domain } => {
                    let d = domain.trim().to_lowercase();
                    if !d.is_empty() && !self.settings.adblock_blocked_domains.contains(&d) {
                        self.settings.adblock_blocked_domains.push(d);
                        self.storage.save_settings(&self.settings);
                        self.sync_settings_tabs();
                    }
                }
                IpcIncoming::RemoveAdblockDomain { domain } => {
                    let d = domain.trim().to_lowercase();
                    self.settings.adblock_blocked_domains.retain(|item| item != &d);
                    self.storage.save_settings(&self.settings);
                    self.sync_settings_tabs();
                }
                IpcIncoming::AddAdblockWhitelist { domain } => {
                    let d = domain.trim().to_lowercase();
                    if !d.is_empty() && !self.settings.adblock_whitelisted_domains.contains(&d) {
                        self.settings.adblock_whitelisted_domains.push(d);
                        self.storage.save_settings(&self.settings);
                        self.sync_settings_tabs();
                    }
                }
                IpcIncoming::RemoveAdblockWhitelist { domain } => {
                    let d = domain.trim().to_lowercase();
                    self.settings.adblock_whitelisted_domains.retain(|item| item != &d);
                    self.storage.save_settings(&self.settings);
                    self.sync_settings_tabs();
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
                IpcIncoming::ReportBlockedRequest { domain, url, req_type } => {
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

                    self.blocked_logs.insert(0, crate::ipc::BlockedRequestLog {
                        domain,
                        url,
                        req_type,
                        timestamp: now,
                    });
                    if self.blocked_logs.len() > 50 {
                        self.blocked_logs.truncate(50);
                    }
                    self.sync_settings_tabs();
                }
                IpcIncoming::ReportBlockedAd { domain, url, req_type } => {
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

                    self.adblock_logs.insert(0, crate::ipc::BlockedRequestLog {
                        domain,
                        url,
                        req_type,
                        timestamp: now,
                    });
                    if self.adblock_logs.len() > 50 {
                        self.adblock_logs.truncate(50);
                    }
                    self.sync_settings_tabs();
                }
                IpcIncoming::ClearBrowsingData { cookies, cache, local_storage } => {
                    let mut script = String::new();
                    if local_storage {
                        script.push_str("try { localStorage.clear(); sessionStorage.clear(); } catch(e){}");
                    }
                    if cookies {
                        script.push_str("try { document.cookie.split(';').forEach(c => { document.cookie = c.replace(/^ +/, '').replace(/=.*/, '=;expires=' + new Date().toUTCString() + ';path=/'); }); } catch(e){}");
                    }
                    if cache {
                        script.push_str("try { if (window.caches) { caches.keys().then(keys => caches.delete(k))); } } catch(e){}");
                    }
                    for tab in &self.tabs {
                        if !Self::is_internal_url(&tab.url) {
                            let _ = tab.webview.evaluate_script(&script);
                        }
                    }
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
                IpcIncoming::OpenPrivacy => {
                    if let Some(pos) = self.tabs.iter().position(|t| Self::is_settings_url(&t.url)) {
                        let tab_id = self.tabs[pos].id;
                        self.switch_tab(tab_id);
                        let _ = self.tabs[pos].webview.evaluate_script("window.switchView && window.switchView('privacy');");
                    } else {
                        self.create_tab("titan://privacy");
                    }
                }
                IpcIncoming::OpenAdblock => {
                    if let Some(pos) = self.tabs.iter().position(|t| Self::is_settings_url(&t.url)) {
                        let tab_id = self.tabs[pos].id;
                        self.switch_tab(tab_id);
                        let _ = self.tabs[pos].webview.evaluate_script("window.switchView && window.switchView('adblock');");
                    } else {
                        self.create_tab("titan://adblock");
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
                blocked_logs: self.blocked_logs.clone(),
                adblock_logs: self.adblock_logs.clone(),
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
