// Titan Browser - Android Adblock Page Script (TypeScript)
// Browser APIs are patched intentionally, so the DOM monkey patches below are checked at runtime.
// @ts-nocheck

interface TitanAndroidAdblockConfig {
  blockVideoAds: boolean;
  cosmeticFiltering: boolean;
  blockPopups: boolean;
  aggressiveMode: boolean;
  whitelistedDomains: string[];
  blockedDomains: string[];
  filterRuleHosts: string[];
  staticSelectors: string[];
}

declare const __TITAN_ANDROID_ADBLOCK_CONFIG__: TitanAndroidAdblockConfig;

const config = __TITAN_ANDROID_ADBLOCK_CONFIG__;

(function() {
    try {
        const blockVideoAds = config.blockVideoAds;
        const cosmeticFiltering = config.cosmeticFiltering;
        const blockPopups = config.blockPopups;
        const aggressiveMode = config.aggressiveMode;
        const AD_WHITELIST = config.whitelistedDomains;
        const AD_DOMAINS = config.blockedDomains;
        const FILTER_RULE_HOSTS = config.filterRuleHosts;
        const STATIC_SELECTORS = config.staticSelectors;
        const KNOWN_BLOCKED_HOSTS = [
            'googletagmanager.com',
            'google-analytics.com',
            'analytics.google.com',
            'pagead2.googlesyndication.com',
            'googlesyndication.com',
            'googleadservices.com',
            'an.yandex.ru',
            'mc.yandex.ru',
            'static.hotjar.com',
            'hotjar.com',
            'browser.sentry-cdn.com',
            'js.sentry-cdn.com',
            'sentry-cdn.com',
            'bugsnag.com',
            'd2wy8f7a9ursnm.cloudfront.net',
            'ymatuhin.ru',
            'tagivi.com',
            'fellowearnwave.com',
            'sharethis.com',
            't.sharethis.com',
            'static.cloudflareinsights.com'
        ];
        const currentHost = (window.location.hostname || '').toLowerCase();
        const currentHref = (window.location.href || '').toLowerCase();

        if (!currentHost || currentHref.startsWith('titan://') || currentHref.startsWith('about:')) return;
        if (currentHost.includes('browserbench.org') || currentHost.includes('speedometer') || currentHost.includes('krakenbenchmark') || currentHost.includes('webglreport')) return;
        if (AD_WHITELIST.some(d => d && (currentHost === d.toLowerCase() || currentHost.endsWith('.' + d.toLowerCase())))) return;

        function matchesDomain(host, domain) {
            const cleanHost = String(host || '').toLowerCase().replace(/^www\./, '');
            const cleanDomain = String(domain || '').toLowerCase().replace(/^\|\|/, '').replace(/\^$/, '').replace(/^www\./, '');
            return cleanDomain && (cleanHost === cleanDomain || cleanHost.endsWith('.' + cleanDomain));
        }

        function isBlockedUrl(testUrl) {
            if (!testUrl || typeof testUrl !== 'string') return false;
            const lower = testUrl.toLowerCase();
            if (lower.startsWith('data:') || lower.startsWith('blob:') || lower.startsWith('about:') || lower.startsWith('file:') || lower.startsWith('titan:')) return false;
            let parsedHost = '';
            try {
                parsedHost = new URL(testUrl, window.location.href).hostname.toLowerCase().replace(/^www\./, '');
            } catch(e) {}
            for (const w of AD_WHITELIST) {
                const wl = (w || '').toLowerCase();
                if (parsedHost && matchesDomain(parsedHost, wl)) return false;
            }
            for (const blockedHost of KNOWN_BLOCKED_HOSTS) {
                if (parsedHost && (parsedHost === blockedHost || parsedHost.endsWith('.' + blockedHost))) return true;
            }
            for (const blockedHost of FILTER_RULE_HOSTS) {
                if (parsedHost && (parsedHost === blockedHost || parsedHost.endsWith('.' + blockedHost))) return true;
            }
            for (const d of AD_DOMAINS) {
                const domain = (d || '').toLowerCase();
                if (parsedHost && matchesDomain(parsedHost, domain)) return true;
            }
            const isThirdParty = parsedHost && currentHost && !matchesDomain(parsedHost, currentHost);
            const genericAdPath = lower.includes('/ads/') || lower.includes('/ad/') || lower.includes('/banners/') || lower.includes('/banner/');
            const highConfidenceAdPath = lower.includes('popunder') || lower.includes('pr_advertising_ads_banner') || lower.includes('adsbygoogle.js') || lower.includes('metrika/tag.js') || lower.includes('bugsnag.min.js');
            const thirdPartyBeacon = lower.includes('ren.gif') || lower.includes('impr.gif') || lower.includes('context.js');
            if ((isThirdParty && (genericAdPath || thirdPartyBeacon)) || highConfidenceAdPath) return true;
            if (aggressiveMode && (lower.includes('adservice') || lower.includes('adserver') || lower.includes('telemetry') || lower.includes('tracking') || lower.includes('analytics') || lower.includes('pixel') || lower.includes('click') || lower.includes('promo'))) return true;
            return false;
        }

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

        try {
            const blockedDialogText = /not a robot|verify you are human|click allow|press allow|tap allow|payment has increased|hurry up|get your money|you won|claim prize|confirm you are|notification|file overload|delete files|cleanup is advised|virus|infected|warning/i;
            const originalAlert = window.alert;
            const originalConfirm = window.confirm;
            const originalPrompt = window.prompt;
            window.alert = function(message) {
                if (blockedDialogText.test(String(message || ''))) return undefined;
                return originalAlert.call(this, message);
            };
            window.confirm = function(message) {
                if (blockedDialogText.test(String(message || ''))) return false;
                return originalConfirm.call(this, message);
            };
            window.prompt = function(message, value) {
                if (blockedDialogText.test(String(message || ''))) return null;
                return originalPrompt.call(this, message, value);
            };
        } catch(e) {}

        if (window.fetch) {
            const origFetch = window.fetch;
            window.fetch = async function(...args) {
                const reqUrl = typeof args[0] === 'string' ? args[0] : (args[0] && args[0].url) || '';
                if (isBlockedUrl(reqUrl)) {
                    throw new TypeError('Failed to fetch');
                }
                return origFetch.apply(this, args);
            };
        }

        if (window.XMLHttpRequest) {
            const origOpen = XMLHttpRequest.prototype.open;
            XMLHttpRequest.prototype.open = function(method, url, ...rest) {
                this._titanReqUrl = url;
                return origOpen.call(this, method, url, ...rest);
            };

            const origSend = XMLHttpRequest.prototype.send;
            XMLHttpRequest.prototype.send = function(...args) {
                if (this._titanReqUrl && isBlockedUrl(this._titanReqUrl)) {
                    try {
                        Object.defineProperty(this, 'readyState', { value: 4, configurable: true });
                        Object.defineProperty(this, 'status', { value: 0, configurable: true });
                        Object.defineProperty(this, 'statusText', { value: '', configurable: true });
                        Object.defineProperty(this, 'responseText', { value: '', configurable: true });
                        Object.defineProperty(this, 'response', { value: '', configurable: true });
                        this.dispatchEvent(new Event('readystatechange'));
                        this.dispatchEvent(new Event('error'));
                        this.dispatchEvent(new Event('loadend'));
                    } catch(e) {}
                    return;
                }
                return origSend.apply(this, args);
            };
        }

        function hookElementSrc(proto, dummyUrl) {
            const descriptor = Object.getOwnPropertyDescriptor(proto, 'src');
            if (descriptor && descriptor.set) {
                Object.defineProperty(proto, 'src', {
                    set: function(val) {
                        if (isBlockedUrl(val)) {
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
        if (window.HTMLScriptElement) hookElementSrc(HTMLScriptElement.prototype, 'https://titan-blocked.invalid/ads/blocked.js');
        if (window.HTMLIFrameElement) hookElementSrc(HTMLIFrameElement.prototype, 'about:blank');
        if (window.HTMLImageElement) hookElementSrc(HTMLImageElement.prototype, 'data:image/gif;base64,R0lGODlhAQABAIAAAAAAAP///yH5BAEAAAAALAAAAAABAAEAAAIBRAA7');

        if (blockPopups && window.open) {
            const origOpen = window.open;
            window.open = function(url, target, features) {
                if (!url || url === 'about:blank' || url === '' || isBlockedUrl(url)) return null;
                try {
                    const parsed = new URL(url, window.location.href);
                    if (isBlockedUrl(parsed.href)) return null;
                } catch(e) {}
                return origOpen.call(this, url, target, features);
            };
        }

        if (cosmeticFiltering) {
            const scamSelectors = [
                '[class*="captcha"]',
                '[id*="captcha"]',
                '[class*="robot"]',
                '[id*="robot"]',
                '[class*="verify"]',
                '[id*="verify"]',
                '[class*="bonus"]',
                '[id*="bonus"]',
                '[class*="reward"]',
                '[id*="reward"]',
                '[class*="modal"][style*="z-index"]',
                '[class*="overlay"][style*="z-index"]',
                'iframe[src*="captcha"]',
                'iframe[src*="recaptcha"]',
                'iframe[src*="verify"]',
                'iframe[src*="bonus"]',
                'iframe[src*="reward"]',
                'object[type*="shockwave"]',
                'embed[type*="shockwave"]',
                'object[data*="/banners/"]',
                'embed[src*="/banners/"]',
                'object[data*="pr_advertising_ads_banner"]',
                'embed[src*="pr_advertising_ads_banner"]',
                '#tg-toast',
                '[id*="telegram"][class*="modal"]',
                '[class*="telegram"][class*="modal"]',
                '[class*="tg-toast"]',
                '.social-sharing'
            ];
            const allSelectors = aggressiveMode ? STATIC_SELECTORS.concat(scamSelectors) : STATIC_SELECTORS;
            const adCss = allSelectors.length > 0
                ? allSelectors.join(',\n') + ' { display: none !important; visibility: hidden !important; height: 0 !important; min-height: 0 !important; max-height: 0 !important; width: 0 !important; opacity: 0 !important; pointer-events: none !important; overflow: hidden !important; }'
                : '';

            function injectAdStyle() {
                if (document.getElementById('titan-adblock-style')) return;
                const style = document.createElement('style');
                style.id = 'titan-adblock-style';
                style.textContent = adCss;
                (document.head || document.documentElement).appendChild(style);
            }

            function cleanAnnoyances() {
                try {
                    const blockedText = [
                        'confirm you are not a robot',
                        "i'm not a robot",
                        'im not a robot',
                        'verify you are human',
                        'click the button',
                        'click allow',
                        'kein roboter',
                        'not a robot',
                        'press allow',
                        'tap allow',
                        'payment has increased',
                        'file overload warning',
                        'delete files',
                        'cleanup is advised',
                        'swift cleanup',
                        'detected. swift cleanup',
                        'security warning',
                        'virus detected',
                        'device infected',
                        'storage full',
                        'clean your device',
                        'the site may close tomorrow',
                        'telegram channel',
                        'join now while',
                        'hurry up to get your money',
                        'get your money now',
                        'hurry up',
                        'claim your bonus',
                        'get bonus',
                        'before time runs out',
                        'claim your prize',
                        'you won',
                        'congratulations',
                        'continue',
                        'warning'
                    ];
                    const nodes = document.querySelectorAll('body *');
                    const viewportArea = Math.max(1, window.innerWidth * window.innerHeight);

                    function hideElement(el) {
                        if (!el || el === document.body || el === document.documentElement) return;
                        const marker = ((el.id || '') + ' ' + (el.className || '')).toString().toLowerCase();
                        if (el.id === 'app' || el.id === 'wrapper' || marker.includes('container') || marker.includes('content')) return;
                        try {
                            el.style.setProperty('display', 'none', 'important');
                            el.style.setProperty('visibility', 'hidden', 'important');
                            el.style.setProperty('opacity', '0', 'important');
                            el.style.setProperty('pointer-events', 'none', 'important');
                            el.setAttribute('data-titan-adblock-hidden', 'true');
                        } catch(e) {}
                    }

                    function overlayAncestor(el) {
                        let candidate = el;
                        for (let depth = 0; candidate && depth < 8; depth++) {
                            if (candidate === document.body || candidate === document.documentElement) break;
                            const style = window.getComputedStyle(candidate);
                            const rect = candidate.getBoundingClientRect();
                            const z = parseInt(style.zIndex || '0', 10) || 0;
                            const area = Math.max(0, rect.width * rect.height);
                            const isOverlay = style.position === 'fixed' || style.position === 'sticky' || z >= 100;
                            if (isOverlay) return candidate;
                            candidate = candidate.parentElement;
                        }
                        candidate = el;
                        for (let depth = 0; candidate && depth < 8; depth++) {
                            if (candidate === document.body || candidate === document.documentElement) break;
                            const marker = ((candidate.id || '') + ' ' + (candidate.className || '')).toString().toLowerCase();
                            if (marker.includes('modal') || marker.includes('overlay') || marker.includes('popup') || marker.includes('captcha') || marker.includes('robot') || marker.includes('verify')) {
                                return candidate;
                            }
                            candidate = candidate.parentElement;
                        }
                        return el.parentElement || el;
                    }

                    for (const el of nodes) {
                        const marker = ((el.id || '') + ' ' + (el.className || '')).toString().toLowerCase();
                        const text = ((el.innerText || el.textContent || '') + ' ' + (el.getAttribute('aria-label') || '') + ' ' + (el.getAttribute('title') || '') + ' ' + marker).toLowerCase();
                        const src = ((el.getAttribute('src') || '') + ' ' + (el.getAttribute('data-src') || '')).toLowerCase();
                        const style = window.getComputedStyle(el);
                        const rect = el.getBoundingClientRect();
                        const z = parseInt(style.zIndex || '0', 10) || 0;
                        const area = Math.max(0, rect.width * rect.height);
                        const textCandidate = text.length < 800 &&
                            (style.position === 'fixed' || style.position === 'sticky' || z >= 100 ||
                                marker.includes('modal') || marker.includes('popup') || marker.includes('toast') || marker.includes('overlay'));
                        if (marker.includes('tg-toast')) {
                            hideElement(el);
                            continue;
                        }

                        const objectData = ((el.getAttribute('data') || '') + ' ' + (el.getAttribute('type') || '') + ' ' + src).toLowerCase();
                        if ((el.tagName === 'OBJECT' || el.tagName === 'EMBED') &&
                            (objectData.includes('shockwave') || objectData.includes('/banners/') || objectData.includes('pr_advertising_ads_banner'))) {
                            hideElement(el);
                            if (el.parentElement && el.parentElement.className && String(el.parentElement.className).toLowerCase().includes('include')) {
                                hideElement(el.parentElement);
                            }
                            continue;
                        }

                        const markerLooksBad = marker.includes('captcha') || marker.includes('robot') || marker.includes('verify-human') || marker.includes('push') || marker.includes('interstitial') || marker.includes('bonus') || marker.includes('reward') || marker.includes('pop') || marker.includes('modal') || marker.includes('overlay') || marker.includes('tg-toast') || marker.includes('telegram');
                        const looksLikeScamPrompt = (textCandidate && blockedText.some(token => text.includes(token))) || markerLooksBad || (el.tagName === 'IFRAME' && (src.includes('captcha') || src.includes('recaptcha') || src.includes('verify')));
                        if (looksLikeScamPrompt && el.parentElement) {
                            hideElement(overlayAncestor(el));
                            if (document.body) {
                                document.body.style.removeProperty('overflow');
                                document.body.style.removeProperty('position');
                                document.body.classList.remove('modal-open', 'no-scroll');
                            }
                        }
                        const isHighOverlay = (style.position === 'fixed' || style.position === 'sticky') && z >= 1000 && area > viewportArea * 0.18;
                        const isLargeBottomOverlay = (style.position === 'fixed' || style.position === 'sticky') &&
                            area > viewportArea * 0.20 &&
                            rect.height > 140 &&
                            rect.bottom > window.innerHeight * 0.70;
                        const isSuspiciousFrame = el.tagName === 'IFRAME' &&
                            (area > viewportArea * 0.12 || src.includes('bonus') || src.includes('reward') || src.includes('promo') || src.includes('offer'));
                        const isTransparentTrap = isHighOverlay && parseFloat(style.opacity || '1') <= 0.05;
                        const isDimBackdrop = isHighOverlay && (style.backgroundColor || '').includes('0, 0, 0');
                        if (isTransparentTrap || isDimBackdrop || isLargeBottomOverlay || isSuspiciousFrame) {
                            hideElement(el);
                        }
                    }
                } catch(e) {}
            }

            injectAdStyle();
            if (document.readyState === 'loading') {
                document.addEventListener('DOMContentLoaded', injectAdStyle, { once: true });
            }

            if (aggressiveMode) {
                const observer = new MutationObserver(cleanAnnoyances);
                if (document.documentElement) observer.observe(document.documentElement, { childList: true, subtree: true });
                [50, 150, 300, 600, 1200, 2400, 4800].forEach(delay => setTimeout(cleanAnnoyances, delay));
                let cleanRuns = 0;
                const cleanerInterval = setInterval(() => {
                    cleanRuns += 1;
                    cleanAnnoyances();
                    if (cleanRuns > 80) clearInterval(cleanerInterval);
                }, 500);
            }
        }

        if (blockVideoAds && (currentHost.includes('youtube.com') || currentHost.includes('youtu.be'))) {
            function handleVideoAds() {
                let foundAdUi = false;
                try {
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

                    for (const sel of skipSelectors) {
                        const btn = document.querySelector(sel);
                        if (btn) {
                            foundAdUi = true;
                            btn.click();
                        }
                    }

                    const adElements = document.querySelectorAll('.ad-showing, .ad-interrupting, .ytp-ad-player-overlay');
                    if (adElements.length > 0) {
                        foundAdUi = true;
                        const videos = document.querySelectorAll('video');
                        videos.forEach(v => {
                            if (v && !isNaN(v.duration) && v.duration > 0) {
                                v.muted = true;
                                v.playbackRate = 16.0;
                                v.currentTime = v.duration;
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
                    const nextDelay = document.hidden ? 5000 : (foundAdUi ? 250 : 1500);
                    scheduleVideoAdCheck(nextDelay);
                }, delay);
            }
            document.addEventListener('visibilitychange', () => scheduleVideoAdCheck(0));
            scheduleVideoAdCheck(0);
        }
    } catch(e) {}
})();
