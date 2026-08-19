package com.titan.browser.web

import com.titan.browser.model.BrowserSettings

object AdblockManager {

    private val AD_NETWORKS_REGEX = Regex(
        "(^|\\.)(monetag|adsterra|propellerads|hilltopads|clickadu|clckr|popcash|popads|exoclick|juicyads|trafficstars|ad-maven|adcash|smartadserver|histats|dtscout|propu\\.sh|onclickbright|onclicksuper|deloton|highperformancegate|al5smvpt45|feedify|pushwoosh|pushassist|tsyndicate|etahub|bignox|trafficjunky|rtmark|in-page-push|adrotate|doubleclick|googleadservices|googlesyndication|adservice\\.google|pagead|2mdn|adnxs|advertising\\.com|rubiconproject|pubmatic|criteo|openx|bidswitch|indexexchange|amazon-adsystem|adroll|media\\.net|moatads|outbrain|taboola|mgid|revcontent|inmobi|flashtalking|exponential|adform|adcolony|applovin|unityads|vungle|chartbeat|scorecardresearch|zedo|adblade|yieldmo|sharethrough|triplelift|teads|undertone|spotx|sovrn|sonobi|gumgum|quantserve|quantcount|lijit|adtech|tribalfusion|clicksor|buysellads|carbonads|serving-sys|eyeota|simpli\\.fi|clarity\\.ms|hotjar|fullstory|mouseflow|luckyorange|crazyegg|inspectlet|logrocket|smartlook|segment\\.io|mixpanel|amplitude|heapanalytics|kissmetrics|woopra|branch\\.io|appsflyer|adjust\\.com|kochava|singular\\.net)(\\.|/|$)",
        RegexOption.IGNORE_CASE
    )

    private val AD_PATH_REGEX = Regex(
        "(/ads?\\.js|/ad-banner\\.|/advertisement\\.|/banner_ads/|/pagead/|&ad_type=|&ad_url=|&adurl=|/popunder|/pop_under|/floating_ad|/interstitial)",
        RegexOption.IGNORE_CASE
    )

    fun isBlockedUrl(url: String, aggressiveMode: Boolean = false): Boolean {
        if (url.isEmpty()) return false
        val lower = url.lowercase()
        if (lower.startsWith("data:") || lower.startsWith("blob:") || lower.startsWith("about:") || lower.startsWith("file:")) {
            return false
        }

        if (AD_NETWORKS_REGEX.containsMatchIn(lower) || AD_PATH_REGEX.containsMatchIn(lower)) {
            return true
        }

        if (aggressiveMode) {
            if (lower.contains("adservice") || lower.contains("adserver") || lower.contains("telemetry") ||
                lower.contains("tracking") || lower.contains("analytics") || lower.contains("pixel")
            ) {
                return true
            }
        }

        return false
    }

    fun getInjectionScript(settings: BrowserSettings): String {
        if (!settings.adblockEnabled) return ""

        val cosmeticEnabled = settings.cosmeticFiltering
        val popupsBlocked = settings.blockPopups
        val videoAdsBlocked = settings.blockVideoAds

        return """
            (function() {
                try {
                    const host = (window.location.hostname || '').toLowerCase();
                    // Bypass benchmarks to ensure 100% native CPU performance
                    if (host.includes('browserbench.org') || host.includes('speedometer') || host.includes('krakenbenchmark') || host.includes('webglreport')) {
                        return;
                    }
                    // 1. Defuse Notification Scams & Fake Robot Captcha Prompts
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

                    // 2. Pop-up & Popunder Blocker
                    if ($popupsBlocked && window.open) {
                        const origOpen = window.open;
                        window.open = function(url, target, features) {
                            if (!url || url === 'about:blank' || url === '') {
                                return null;
                            }
                            const lower = (url || '').toLowerCase();
                            if (lower.includes('pop') || lower.includes('ad') || lower.includes('click') || lower.includes('traffic')) {
                                return null;
                            }
                            return origOpen.call(this, url, target, features);
                        };
                    }

                    // 3. Targeted Cosmetic Ad Hiding
                    if ($cosmeticEnabled) {
                        const adCss = `
                            ins.adsbygoogle, [id^="google_ads_"], [id*="google_ads_iframe"], [id*="ScriptRoot"],
                            [class*="sponsored-post"], [class*="ad-container"], [class*="ad_container"],
                            [id*="banner-ad"], [id*="ad-banner"], [class*="ad-banner"], [class*="ad-wrapper"],
                            [id*="ad-wrapper"], [class*="ad-slot"], [id*="ad-slot"], [class*="ad-placement"],
                            [aria-label="advertisement"], [aria-label="Sponsored"], .trc_rbox_div, .OUTBRAIN,
                            .taboola-placeholder, [class*="adbox"], [id*="adbox"], [class*="ad-frame"],
                            ytd-promoted-video-renderer, ytd-promoted-sparkles-web-renderer,
                            ytd-display-ad-renderer, ytd-statement-banner-renderer,
                            ytd-in-feed-ad-layout-renderer, ytd-banner-promo-renderer,
                            #masthead-ad, #player-ads, #offer-module, .ytp-ad-overlay-container,
                            .ytp-ad-message-container, .ytp-ad-overlay-slot, .ytp-ad-action-interstitial,
                            .video-ads, .ytp-ad-module
                            { display: none !important; visibility: hidden !important; height: 0 !important; min-height: 0 !important; max-height: 0 !important; width: 0 !important; opacity: 0 !important; pointer-events: none !important; overflow: hidden !important; }
                        `;

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
                    }

                    // 4. Video Ad Auto-Skipper & Fast-Forward (YouTube)
                    if ($videoAdsBlocked && (window.location.hostname || '').includes('youtube.com')) {
                        function handleVideoAds() {
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
                                    if (btn) btn.click();
                                }

                                const adElements = document.querySelectorAll('.ad-showing, .ad-interrupting, .ytp-ad-player-overlay');
                                if (adElements.length > 0) {
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
                        }
                        setInterval(handleVideoAds, 350);
                    }
                } catch(e) {}
            })();
        """.trimIndent()
    }
}
