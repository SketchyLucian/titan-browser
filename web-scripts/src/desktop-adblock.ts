// Titan Browser - Desktop Adblock Page Script (TypeScript)
// @ts-nocheck

interface TitanDesktopAdblockConfig {
  enabled: boolean;
  blockVideoAds: boolean;
  cosmeticFiltering: boolean;
  blockPopups: boolean;
  aggressiveMode: boolean;
  whitelistedDomains: string[];
  blockedDomains: string[];
  dynamicSelectors: string[];
  scriptletCode: string;
}

declare const __TITAN_DESKTOP_ADBLOCK_CONFIG__: TitanDesktopAdblockConfig;

(function() {
    const config = __TITAN_DESKTOP_ADBLOCK_CONFIG__;
    try {
        const enabled = config.enabled;
        if (!enabled) return;

        const blockVideoAds = config.blockVideoAds;
        const cosmeticFiltering = config.cosmeticFiltering;
        const blockPopups = config.blockPopups;
        const aggressiveMode = config.aggressiveMode;
        const AD_WHITELIST = config.whitelistedDomains;
        const AD_DOMAINS = config.blockedDomains;
        const DYNAMIC_SELECTORS = config.dynamicSelectors;
        const SCRIPTLET_CODE = config.scriptletCode;

        const currentHost = (window.location.hostname || '').toLowerCase();
        const currentHref = (window.location.href || '').toLowerCase();
        const isYouTube = currentHost.includes('youtube.com') || currentHost.includes('youtu.be');

        if (!currentHost || currentHref.startsWith('titan://') || currentHref.startsWith('about:')) return;

        // Check if current website is whitelisted
        if (AD_WHITELIST.some(d => d && (currentHost === d.toLowerCase() || currentHost.endsWith('.' + d.toLowerCase())))) {
            return; // Ad blocking disabled for this whitelisted site
        }

        // Comprehensive Ad & Tracker Network Patterns (uBlock / EasyList / Popunder engines)
        const AD_NETWORKS_REGEX = /(^|\.)(monetag|adsterra|propellerads|hilltopads|clickadu|clckr|popcash|popads|exoclick|juicyads|trafficstars|ad-maven|adcash|smartadserver|histats|dtscout|propu\.sh|onclickbright|onclicksuper|deloton|highperformancegate|al5smvpt45|feedify|pushwoosh|pushassist|tsyndicate|etahub|bignox|trafficjunky|rtmark|in-page-push|adrotate|doubleclick|googleadservices|googlesyndication|adservice\.google|pagead|2mdn|adnxs|advertising\.com|rubiconproject|pubmatic|criteo|openx|bidswitch|indexexchange|amazon-adsystem|adroll|media\.net|moatads|outbrain|taboola|mgid|revcontent|inmobi|flashtalking|exponential|adform|adcolony|applovin|unityads|vungle|chartbeat|scorecardresearch|zedo|adblade|yieldmo|sharethrough|triplelift|teads|undertone|spotx|sovrn|sonobi|gumgum|quantserve|quantcount|lijit|adtech|tribalfusion|clicksor|buysellads|carbonads|serving-sys|eyeota|simpli\.fi|clarity\.ms|hotjar|fullstory|mouseflow|luckyorange|crazyegg|inspectlet|logrocket|smartlook|segment\.io|mixpanel|amplitude|heapanalytics|kissmetrics|woopra|branch\.io|appsflyer|adjust\.com|kochava|singular\.net)(\.|\/|$)/i;

        const AD_PATH_REGEX = /(\/ads?\.js|\/ad-banner\.|\/advertisement\.|\/banner_ads\/|\/pagead\/|&ad_type=|&ad_url=|&adurl=|\/popunder|\/pop_under|\/floating_ad|\/interstitial)/i;

        // YouTube carries ad metadata inside otherwise legitimate player responses.
        // Remove only the ad objects; blocking googlevideo wholesale also blocks the real video.
        const YOUTUBE_AD_KEYS = new Set([
            'adPlacements',
            'playerAds',
            'adSlots',
            'adBreakHeartbeatParams',
            'adBreakParams',
            'adPlacementRenderer',
            'linearAdSequenceRenderer',
            'playerLegacyDesktopWatchAdsRenderer',
            'playerAdParams'
        ]);

        function stripYouTubeAdMetadata(value, seen) {
            if (!value || typeof value !== 'object') return value;
            const visited = seen || new WeakSet();
            if (visited.has(value)) return value;
            visited.add(value);

            if (Array.isArray(value)) {
                for (const item of value) stripYouTubeAdMetadata(item, visited);
                return value;
            }

            for (const key of Object.keys(value)) {
                if (YOUTUBE_AD_KEYS.has(key)) {
                    try { delete value[key]; } catch(e) {}
                    continue;
                }
                stripYouTubeAdMetadata(value[key], visited);
            }
            return value;
        }

        function looksLikeYouTubePlayerPayload(text) {
            return typeof text === 'string' &&
                (text.includes('"adPlacements"') || text.includes('"playerAds"') || text.includes('"adSlots"'));
        }

        if (blockVideoAds && isYouTube) {
            const originalJsonParse = JSON.parse;
            JSON.parse = function(text, reviver) {
                const parsed = originalJsonParse.call(this, text, reviver);
                return looksLikeYouTubePlayerPayload(text)
                    ? stripYouTubeAdMetadata(parsed)
                    : parsed;
            };

            if (window.Response && Response.prototype.json) {
                const originalResponseJson = Response.prototype.json;
                Response.prototype.json = async function() {
                    const parsed = await originalResponseJson.call(this);
                    return this.url && this.url.includes('/youtubei/v1/player')
                        ? stripYouTubeAdMetadata(parsed)
                        : parsed;
                };
            }

            try {
                let initialPlayerResponse;
                Object.defineProperty(window, 'ytInitialPlayerResponse', {
                    configurable: true,
                    get: function() { return initialPlayerResponse; },
                    set: function(value) {
                        initialPlayerResponse = stripYouTubeAdMetadata(value);
                    }
                });
            } catch(e) {}
        }

        // Helper to match URL against ad/tracker rules
        function isBlockedUrl(testUrl, reqType) {
            if (!testUrl || typeof testUrl !== 'string') return false;
            const lower = testUrl.toLowerCase();
            if (lower.startsWith('data:') || lower.startsWith('blob:') || lower.startsWith('about:')) return false;

            // Check whitelisted domains
            for (const w of AD_WHITELIST) {
                if (w && (lower.includes('://' + w.toLowerCase()) || lower.includes('.' + w.toLowerCase() + '/'))) return false;
            }

            // Match full ad networks regex
            if (AD_NETWORKS_REGEX.test(lower) || AD_PATH_REGEX.test(lower)) {
                return true;
            }

            // Check user custom domain list
            for (const d of AD_DOMAINS) {
                if (d && (lower.includes('://' + d.toLowerCase()) || lower.includes('.' + d.toLowerCase()) || lower.includes('/' + d.toLowerCase()))) {
                    return true;
                }
            }

            if (aggressiveMode) {
                if (lower.includes('adservice') || lower.includes('adserver') || lower.includes('telemetry') || lower.includes('tracking') || lower.includes('analytics') || lower.includes('pixel')) {
                    return true;
                }
            }

            return false;
        }

        function reportBlocked(url, reqType) {
            try {
                let domain = '';
                try {
                    domain = new URL(url, window.location.href).hostname;
                } catch(e) {
                    domain = currentHost;
                }
                if (window.ipc && window.ipc.postMessage) {
                    window.ipc.postMessage(JSON.stringify({
                        type: 'ReportBlockedAd',
                        domain: domain || currentHost,
                        url: (url || '').substring(0, 300),
                        req_type: reqType || 'other'
                    }));
                }
            } catch(e) {}
        }

        // 1. Defuse Notification Spam & Scam Prompts (Fake Robot Captchas)
        if (window.Notification) {
            try {
                window.Notification.requestPermission = function() {
                    return Promise.resolve('denied');
                };
                Object.defineProperty(window.Notification, 'permission', {
                    get: function() { return 'denied'; },
                    configurable: true
                });
            } catch(e) {}
        }

        // 2. Network Level Interception: window.fetch Hook
        if (window.fetch) {
            const origFetch = window.fetch;
            window.fetch = async function(...args) {
                const reqUrl = typeof args[0] === 'string' ? args[0] : (args[0] && args[0].url) || '';
                if (isBlockedUrl(reqUrl, 'fetch')) {
                    reportBlocked(reqUrl, 'fetch');
                    throw new TypeError('Failed to fetch');
                }
                return origFetch.apply(this, args);
            };
        }

        // 3. Network Level Interception: XMLHttpRequest Hook
        if (window.XMLHttpRequest) {
            const origOpen = XMLHttpRequest.prototype.open;
            XMLHttpRequest.prototype.open = function(method, url, ...rest) {
                this._titanReqUrl = url;
                return origOpen.call(this, method, url, ...rest);
            };

            const origSend = XMLHttpRequest.prototype.send;
            XMLHttpRequest.prototype.send = function(...args) {
                if (this._titanReqUrl && isBlockedUrl(this._titanReqUrl, 'xhr')) {
                    reportBlocked(this._titanReqUrl, 'xhr');
                    try {
                        Object.defineProperty(this, 'readyState', { value: 4, configurable: true });
                        Object.defineProperty(this, 'status', { value: 200, configurable: true });
                        Object.defineProperty(this, 'statusText', { value: 'OK', configurable: true });
                        Object.defineProperty(this, 'responseText', { value: '', configurable: true });
                        Object.defineProperty(this, 'response', { value: '', configurable: true });
                        this.dispatchEvent(new Event('readystatechange'));
                        this.dispatchEvent(new Event('load'));
                        this.dispatchEvent(new Event('loadend'));
                    } catch(e) {}
                    return;
                }
                return origSend.apply(this, args);
            };
        }

        // 4. Network Level Interception: Script & Frame DOM Element hooking
        function hookElementSrc(proto, reqType, dummyUrl) {
            const descriptor = Object.getOwnPropertyDescriptor(proto, 'src');
            if (descriptor && descriptor.set) {
                Object.defineProperty(proto, 'src', {
                    set: function(val) {
                        if (isBlockedUrl(val, reqType)) {
                            reportBlocked(val, reqType);
                            descriptor.set.call(this, dummyUrl || '');
                            return;
                        }
                        descriptor.set.call(this, val);
                    },
                    get: descriptor.get,
                    configurable: true
                });
            }
        }
        if (window.HTMLScriptElement) hookElementSrc(HTMLScriptElement.prototype, 'script', 'data:text/javascript;base64,');
        if (window.HTMLIFrameElement) hookElementSrc(HTMLIFrameElement.prototype, 'subdocument', 'about:blank');
        if (window.HTMLImageElement) hookElementSrc(HTMLImageElement.prototype, 'image', 'data:image/gif;base64,R0lGODlhAQABAIAAAAAAAP///yH5BAEAAAAALAAAAAABAAEAAAIBRAA7');

        // 5. Pop-up & Popunder Blocker
        if (blockPopups && window.open) {
            const origOpen = window.open;
            window.open = function(url, target, features) {
                if (!url || url === 'about:blank' || url === '' || isBlockedUrl(url, 'popup')) {
                    reportBlocked(url || 'popunder', 'popup');
                    return null;
                }
                try {
                    const parsed = new URL(url, window.location.href);
                    if (isBlockedUrl(parsed.href, 'popup')) {
                        reportBlocked(parsed.href, 'popup');
                        return null;
                    }
                } catch(e) {}
                return origOpen.call(this, url, target, features);
            };
        }

        // 6. Universal Cosmetic Element Hiding (Ads, Social Floating Bars & Fake Captcha Modals)
        if (cosmeticFiltering) {
            let staticSelectors = [
                // Ads & Banners
                'ins.adsbygoogle',
                '[id^="google_ads_"]',
                '[id*="google_ads_iframe"]',
                '[id*="ScriptRoot"]',
                '[class*="sponsored-post"]',
                '[class*="ad-container"]',
                '[class*="ad_container"]',
                '[id*="banner-ad"]',
                '[id*="ad-banner"]',
                '[class*="ad-banner"]',
                '[class*="ad-wrapper"]',
                '[id*="ad-wrapper"]',
                '[class*="ad-slot"]',
                '[id*="ad-slot"]',
                '[class*="ad-placement"]',
                '[aria-label="advertisement"]',
                '[aria-label="Sponsored"]',
                '.trc_rbox_div',
                '.OUTBRAIN',
                '.taboola-placeholder',
                '[class*="adbox"]',
                '[id*="adbox"]',
                '[class*="ad-frame"]',

                // Social Floating Bars & Annoyance Docks
                '[class*="share-bar"]',
                '[class*="floating-share"]',
                '[class*="shares-box"]',
                '.social-share',
                '[id*="share-buttons"]',
                '[class*="share-container"]',
                'div[class*="share-sidebar"]',
                '.at-share-dock',
                '.addthis_floating_style',
                '.sharethis-inline-share-buttons',
                '.st-sticky-share-buttons',
                'div[class*="shares"]',
                'div[class*="share-btn"]',
                'div[class*="social-buttons"]',
                'div[class*="ShareBar"]',
                'div[class*="ShareButtons"]',

                // Fake Robot Captchas, Scam Modals & Push Notification Prompts
                '[class*="captcha-modal"]',
                '[id*="captcha-modal"]',
                '[class*="robot-modal"]',
                '[class*="robot-check"]',
                '[class*="verify-robot"]',
                '[id*="robot-check"]',
                '[class*="notification-prompt"]',
                '[class*="push-prompt"]',
                '[class*="push-modal"]',
                '[class*="ad-modal"]',
                '[class*="popup-modal"]',
                '[id*="popup-modal"]',
                '[class*="interstitial"]',
                '[id*="interstitial"]',
                '[class*="overlay-backdrop"]',
                '[id*="overlay-backdrop"]',
                '[class*="ad-overlay"]',
                '[id*="ad-overlay"]',
                '.modal-backdrop',
                '.popup-backdrop',
                'div[style*="z-index: 2147483647"]',
                'div[style*="z-index: 999999"]',
                'div[style*="z-index: 99999"]',

                // Video Platform Ad Overlays
                'ytd-promoted-video-renderer',
                'ytd-promoted-sparkles-web-renderer',
                'ytd-display-ad-renderer',
                'ytd-statement-banner-renderer',
                'ytd-in-feed-ad-layout-renderer',
                'ytd-banner-promo-renderer',
                '#masthead-ad',
                '#player-ads',
                '#offer-module',
                '.ytp-ad-overlay-container',
                '.ytp-ad-message-container',
                '.ytp-ad-overlay-slot',
                '.ytp-ad-action-interstitial'
            ];

            if (Array.isArray(DYNAMIC_SELECTORS) && DYNAMIC_SELECTORS.length > 0) {
                staticSelectors = staticSelectors.concat(DYNAMIC_SELECTORS);
            }

            const adCss = staticSelectors.join(',\n') + ' { display: none !important; visibility: hidden !important; height: 0 !important; min-height: 0 !important; max-height: 0 !important; width: 0 !important; opacity: 0 !important; pointer-events: none !important; overflow: hidden !important; }';

            function injectAdStyle() {
                if (document.getElementById('titan-adblock-style')) return;
                const style = document.createElement('style');
                style.id = 'titan-adblock-style';
                style.textContent = adCss;
                (document.head || document.documentElement).appendChild(style);
            }

            injectAdStyle();
            if (document.readyState === 'loading') {
                document.addEventListener('DOMContentLoaded', injectAdStyle, { once: true });
            }

            // Active Realtime Annoyance & Fake Robot Modal Cleaner
            function cleanAnnoyances() {
                try {
                    const modals = document.querySelectorAll('div, dialog, section');
                    for (const el of modals) {
                        const text = (el.innerText || '').toLowerCase();
                        if (
                            (text.includes('kein roboter') || text.includes('not a robot') || text.includes('verify you are human') || text.includes('click allow') || text.includes('klicken sie auf den button')) &&
                            (el.querySelector('img, svg, button') || el.classList.contains('modal') || (el.style && (el.style.position === 'fixed' || el.style.position === 'absolute')))
                        ) {
                            if (el !== document.body && el !== document.documentElement && el.parentElement) {
                                el.style.setProperty('display', 'none', 'important');
                                el.style.setProperty('pointer-events', 'none', 'important');
                                el.style.setProperty('visibility', 'hidden', 'important');
                                if (document.body) {
                                    document.body.style.removeProperty('overflow');
                                    document.body.classList.remove('modal-open', 'no-scroll');
                                }
                            }
                        }

                        // Remove transparent full-screen click traps
                        if (el.tagName === 'DIV' && el.parentElement === document.body && el.style) {
                            if (el.style.position === 'fixed' && (el.style.zIndex === '2147483647' || parseInt(el.style.zIndex, 10) > 9999) && el.style.opacity === '0') {
                                el.style.setProperty('display', 'none', 'important');
                                el.remove();
                            }
                        }
                    }
                } catch(e) {}
            }

            const observer = new MutationObserver(() => {
                cleanAnnoyances();
            });
            if (document.documentElement) {
                observer.observe(document.documentElement, { childList: true, subtree: true });
            } else {
                document.addEventListener('DOMContentLoaded', () => {
                    if (document.documentElement) observer.observe(document.documentElement, { childList: true, subtree: true });
                }, { once: true });
            }
            setTimeout(cleanAnnoyances, 200);
            setTimeout(cleanAnnoyances, 600);
            setTimeout(cleanAnnoyances, 1200);
        }

        // 7. Execute uBO Scriptlet Defusers
        if (SCRIPTLET_CODE && typeof SCRIPTLET_CODE === 'string' && SCRIPTLET_CODE.trim()) {
            try {
                const fn = new Function(SCRIPTLET_CODE);
                fn();
            } catch(e) {}
        }

        // 8. Video Ad Auto-Skipper & Fast-Forward (Active on YouTube)
        if (blockVideoAds && isYouTube) {
            function handleVideoAds() {
                let foundAdUi = false;
                try {
                    const skipSelectors = [
                        'button.ytp-ad-skip-button',
                        'button.ytp-ad-skip-button-modern',
                        'button.ytp-skip-ad-button',
                        '.ytp-ad-skip-button-slot button',
                        '.ytp-ad-skip-button-container button',
                        '[class*="ytp-ad-skip-button"] button',
                        '.ytp-ad-overlay-close-button',
                        '.videoAdUiSkipButton',
                        '[id^="skip-button"] button',
                        'button[id^="skip-button"]'
                    ];

                    for (const sel of skipSelectors) {
                        const buttons = document.querySelectorAll(sel);
                        for (const btn of buttons) {
                            foundAdUi = true;
                            if (typeof btn.click === 'function') btn.click();
                        }
                    }

                    const adElements = document.querySelectorAll('.ad-showing, .ad-interrupting, .ytp-ad-player-overlay');
                    if (adElements.length > 0) {
                        foundAdUi = true;
                        const videos = document.querySelectorAll('video');
                        videos.forEach(v => {
                            if (!v) return;
                            v.muted = true;
                            v.playbackRate = 16.0;
                            if (Number.isFinite(v.duration) && v.duration > 0) {
                                v.currentTime = v.duration;
                            } else if (v.seekable && v.seekable.length > 0) {
                                v.currentTime = v.seekable.end(v.seekable.length - 1);
                            }
                        });
                    }
                } catch(e) {}
                return foundAdUi;
            }

            let videoAdTimer = 0;
            function scheduleVideoAdCheck(delay) {
                window.clearTimeout(videoAdTimer);
                videoAdTimer = window.setTimeout(() => {
                    const foundAdUi = handleVideoAds();
                    const nextDelay = document.hidden ? 3000 : (foundAdUi ? 100 : 1000);
                    scheduleVideoAdCheck(nextDelay);
                }, delay);
            }

            const playerObserver = new MutationObserver(() => scheduleVideoAdCheck(0));
            if (document.documentElement) {
                playerObserver.observe(document.documentElement, { childList: true, subtree: true });
            }
            document.addEventListener('visibilitychange', () => scheduleVideoAdCheck(0));
            scheduleVideoAdCheck(0);
        }
    } catch(e) {}
})();
