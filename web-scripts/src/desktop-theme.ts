// Titan Browser - Desktop Theme Page Script (TypeScript)
// @ts-nocheck

interface TitanDesktopThemeConfig {
  isLight: boolean;
  forceAdaptation: boolean;
}

declare const __TITAN_DESKTOP_THEME_CONFIG__: TitanDesktopThemeConfig;

(function() {
    const config = __TITAN_DESKTOP_THEME_CONFIG__;
    const isLight = config.isLight;
    const forceAdaptation = config.forceAdaptation;
    const targetMode = isLight ? 'light' : 'dark';
    const removeMode = isLight ? 'dark' : 'light';
    const host = (window.location.hostname || '').toLowerCase();
    const href = (window.location.href || '').toLowerCase();

    // If on internal / blank URL, stop here
    if (!host || href.startsWith('titan://') || href.startsWith('about:')) return;

    // 1. Clean up any previous forced styles if adaptation is off
    try {
        const preCanvas = document.getElementById('titan-pre-dark-canvas');
        if (preCanvas) preCanvas.remove();
        const adaptStyle = document.getElementById('titan-theme-adaptation-style');
        if (!forceAdaptation && adaptStyle) adaptStyle.remove();
    } catch(e) {}

    // 2. Document Root color-scheme & Meta Tag
    try {
        const applyColorScheme = () => {
            if (document.documentElement) {
                document.documentElement.style.colorScheme = targetMode;
            }
        };
        applyColorScheme();
        if (document.readyState === 'loading') {
            document.addEventListener('DOMContentLoaded', applyColorScheme, { once: true });
        }
    } catch(e) {}

    // 3. Framework & Library Integration
    function applyFrameworkThemes() {
        try {
            const html = document.documentElement;
            const body = document.body;

            if (html) {
                html.classList.remove(removeMode);
                html.classList.add(targetMode);

                ['data-theme', 'data-color-mode', 'data-bs-theme', 'data-mode', 'data-theme-mode'].forEach(attr => {
                    if (html.hasAttribute(attr)) {
                        html.setAttribute(attr, targetMode);
                    }
                });
            }

            if (body) {
                if (body.classList.contains('dark') || body.classList.contains('light')) {
                    body.classList.remove(removeMode);
                    body.classList.add(targetMode);
                }
                ['data-theme', 'data-color-mode', 'data-bs-theme', 'data-mode'].forEach(attr => {
                    if (body.hasAttribute(attr)) {
                        body.setAttribute(attr, targetMode);
                    }
                });
            }
        } catch(e) {}
    }

    applyFrameworkThemes();
    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', applyFrameworkThemes, { once: true });
    }

    // 4. Major Website Adaptations
    try {
        const isYouTube = host.includes('youtube.com') || host.includes('youtu.be');
        if (isYouTube && document.documentElement) {
            if (isLight) {
                document.documentElement.removeAttribute('dark');
            } else {
                document.documentElement.setAttribute('dark', 'true');
            }
        }

        const isWikipedia = host.includes('wikipedia.org');
        if (isWikipedia && document.documentElement) {
            document.documentElement.classList.remove(isLight ? 'skin-theme-clientpref-night' : 'skin-theme-clientpref-day');
            document.documentElement.classList.add(isLight ? 'skin-theme-clientpref-day' : 'skin-theme-clientpref-night');
        }
    } catch(e) {}

    // 5. Universal Webpage Theme Adaptation (Dark Reader) for light-only sites
    if (forceAdaptation) {
        function adaptTheme() {
            const isNativelyAdaptive = host.includes('google.') || host.includes('youtube.com') || host.includes('youtu.be') || host.includes('github.com') || host.includes('gitlab.com') || host.includes('reddit.com') || host.includes('duckduckgo.com') || host.includes('bing.com') || host.includes('x.com') || host.includes('twitter.com') || host.includes('tauri.app') || host.includes('vitepress') || host.includes('docusaurus');
            if (isNativelyAdaptive) return;

            let el = document.getElementById('titan-theme-adaptation-style');
            if (!isLight) {
                if (!el) {
                    el = document.createElement('style');
                    el.id = 'titan-theme-adaptation-style';
                    el.textContent = 'html { filter: invert(100%) hue-rotate(180deg) contrast(96%) brightness(96%) !important; background-color: #121316 !important; } img, video, canvas, svg, iframe, [style*="background-image"], .html5-video-player, picture { filter: invert(100%) hue-rotate(180deg) contrast(104%) brightness(104%) !important; }';
                    (document.head || document.documentElement).appendChild(el);
                }
            } else {
                if (el) el.remove();
            }
        }

        if (document.readyState === 'complete') {
            adaptTheme();
        } else {
            window.addEventListener('load', adaptTheme, { once: true });
        }
    }
})();
