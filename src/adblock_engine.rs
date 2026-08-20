use adblock::engine::Engine;
use adblock::lists::{FilterSet, ParseOptions};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::RwLock;

/// Represents an ad/tracker filter list subscription
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterListConfig {
    pub id: String,
    pub name: String,
    pub description: String,
    pub count: usize,
    pub enabled: bool,
}

/// Statistics and metrics about the adblocking engine
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AdblockStats {
    pub total_rules: usize,
    pub blocked_requests_count: u64,
    pub cosmetic_elements_hidden_count: u64,
    pub scriptlets_injected_count: u64,
    pub estimated_bandwidth_saved_bytes: u64,
}

/// Result of evaluating a network request against uBO filter rules
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkFilterDecision {
    pub matched: bool,
    pub is_exception: bool,
    pub rule: Option<String>,
    pub redirect: Option<String>,
}

/// Core uBlock Origin / Adblock Plus manager
pub struct AdblockEngineManager {
    engine: RwLock<Engine>,
    custom_rules: RwLock<Vec<String>>,
    enabled_lists: RwLock<HashSet<String>>,
    stats: RwLock<AdblockStats>,
}

impl Default for AdblockEngineManager {
    fn default() -> Self {
        Self::new()
    }
}

impl AdblockEngineManager {
    /// Creates a new AdblockEngineManager and compiles the standard filter lists
    pub fn new() -> Self {
        let mut enabled_lists = HashSet::new();
        enabled_lists.insert("easylist".to_string());
        enabled_lists.insert("easyprivacy".to_string());
        enabled_lists.insert("ublock_filters".to_string());
        enabled_lists.insert("ublock_badware".to_string());
        enabled_lists.insert("ublock_privacy".to_string());
        enabled_lists.insert("ublock_quick_fixes".to_string());

        let custom_rules = Vec::new();
        let engine = Self::build_engine(&enabled_lists, &custom_rules);
        let total_rules = Self::count_rules(&enabled_lists, &custom_rules);

        let stats = AdblockStats {
            total_rules,
            blocked_requests_count: 0,
            cosmetic_elements_hidden_count: 0,
            scriptlets_injected_count: 0,
            estimated_bandwidth_saved_bytes: 0,
        };

        Self {
            engine: RwLock::new(engine),
            custom_rules: RwLock::new(custom_rules),
            enabled_lists: RwLock::new(enabled_lists),
            stats: RwLock::new(stats),
        }
    }

    /// Rebuilds the underlying adblock engine with current active lists and custom rules
    fn build_engine(enabled_lists: &HashSet<String>, custom_rules: &[String]) -> Engine {
        let mut filter_set = FilterSet::new(true);
        let parse_options = ParseOptions::default();

        if enabled_lists.contains("easylist") {
            let rules = Self::get_easylist_rules().join("\n");
            filter_set.add_filter_list(rules, parse_options);
        }
        if enabled_lists.contains("easyprivacy") {
            let rules = Self::get_easyprivacy_rules().join("\n");
            filter_set.add_filter_list(rules, parse_options);
        }
        if enabled_lists.contains("ublock_filters") {
            let rules = Self::get_ublock_core_rules().join("\n");
            filter_set.add_filter_list(rules, parse_options);
        }
        if enabled_lists.contains("ublock_badware") {
            let rules = Self::get_ublock_badware_rules().join("\n");
            filter_set.add_filter_list(rules, parse_options);
        }
        if enabled_lists.contains("ublock_privacy") {
            let rules = Self::get_ublock_privacy_rules().join("\n");
            filter_set.add_filter_list(rules, parse_options);
        }
        if enabled_lists.contains("ublock_quick_fixes") {
            let rules = Self::get_ublock_quick_fixes_rules().join("\n");
            filter_set.add_filter_list(rules, parse_options);
        }

        if !custom_rules.is_empty() {
            let rules = custom_rules.join("\n");
            filter_set.add_filter_list(rules, parse_options);
        }

        Engine::new_with_filter_set(filter_set)
    }

    fn count_rules(enabled_lists: &HashSet<String>, custom_rules: &[String]) -> usize {
        let mut count = custom_rules.len();
        if enabled_lists.contains("easylist") {
            count += Self::get_easylist_rules().len();
        }
        if enabled_lists.contains("easyprivacy") {
            count += Self::get_easyprivacy_rules().len();
        }
        if enabled_lists.contains("ublock_filters") {
            count += Self::get_ublock_core_rules().len();
        }
        if enabled_lists.contains("ublock_badware") {
            count += Self::get_ublock_badware_rules().len();
        }
        if enabled_lists.contains("ublock_privacy") {
            count += Self::get_ublock_privacy_rules().len();
        }
        if enabled_lists.contains("ublock_quick_fixes") {
            count += Self::get_ublock_quick_fixes_rules().len();
        }
        count
    }

    /// Check if a network request URL is blocked according to active filter rules
    #[allow(dead_code)]
    pub fn check_network_request(
        &self,
        url: &str,
        source_url: &str,
        request_type: &str,
    ) -> NetworkFilterDecision {
        let engine = match self.engine.read() {
            Ok(e) => e,
            Err(_) => {
                return NetworkFilterDecision {
                    matched: false,
                    is_exception: false,
                    rule: None,
                    redirect: None,
                }
            }
        };

        let request = match adblock::request::Request::new(url, source_url, request_type, "GET") {
            Ok(r) => r,
            Err(_) => {
                return NetworkFilterDecision {
                    matched: false,
                    is_exception: false,
                    rule: None,
                    redirect: None,
                }
            }
        };

        let blocker_result = engine.check_network_request(&request);

        if blocker_result.should_block() {
            if let Ok(mut stats) = self.stats.write() {
                stats.blocked_requests_count += 1;
                // Estimate ~45KB average bandwidth saved per blocked ad/tracker request
                stats.estimated_bandwidth_saved_bytes += 46_080;
            }
            NetworkFilterDecision {
                matched: true,
                is_exception: false,
                rule: blocker_result.filter.map(|f| format!("{:?}", f)),
                redirect: blocker_result.redirect,
            }
        } else if blocker_result.exception.is_some() {
            NetworkFilterDecision {
                matched: false,
                is_exception: true,
                rule: blocker_result.exception.map(|f| format!("{:?}", f)),
                redirect: None,
            }
        } else {
            NetworkFilterDecision {
                matched: false,
                is_exception: false,
                rule: None,
                redirect: None,
            }
        }
    }

    /// Retrieve host-specific cosmetic CSS selectors and scriptlets for a given webpage URL
    pub fn get_cosmetic_resources(&self, url: &str) -> (Vec<String>, String) {
        let engine = match self.engine.read() {
            Ok(e) => e,
            Err(_) => return (Vec::new(), String::new()),
        };

        let resources = engine.url_cosmetic_resources(url);
        let hide_selectors: Vec<String> = resources.hide_selectors.into_iter().collect();
        let injected_script = resources.injected_script;

        if let Ok(mut stats) = self.stats.write() {
            stats.cosmetic_elements_hidden_count += hide_selectors.len() as u64;
            if !injected_script.is_empty() {
                stats.scriptlets_injected_count += 1;
            }
        }

        (hide_selectors, injected_script)
    }

    /// Add a custom uBO filter rule (e.g. `||ads.example.com^$script`, `example.com##.ad-banner`)
    pub fn add_custom_rule(&self, rule: String) -> bool {
        let rule = rule.trim().to_string();
        if rule.is_empty() {
            return false;
        }

        let mut custom_rules = match self.custom_rules.write() {
            Ok(r) => r,
            Err(_) => return false,
        };

        if !custom_rules.contains(&rule) {
            custom_rules.push(rule);
            self.rebuild_engine_internal(&custom_rules);
            true
        } else {
            false
        }
    }

    /// Remove a custom uBO filter rule
    pub fn remove_custom_rule(&self, rule: &str) -> bool {
        let mut custom_rules = match self.custom_rules.write() {
            Ok(r) => r,
            Err(_) => return false,
        };

        let prev_len = custom_rules.len();
        custom_rules.retain(|r| r != rule);
        let changed = custom_rules.len() != prev_len;

        if changed {
            self.rebuild_engine_internal(&custom_rules);
        }
        changed
    }

    /// Get all configured custom filter rules
    pub fn get_custom_rules(&self) -> Vec<String> {
        self.custom_rules
            .read()
            .map(|r| r.clone())
            .unwrap_or_default()
    }

    /// Toggle an official filter list on or off
    pub fn toggle_filter_list(&self, list_id: &str, enabled: bool) {
        let mut enabled_lists = match self.enabled_lists.write() {
            Ok(l) => l,
            Err(_) => return,
        };

        if enabled {
            enabled_lists.insert(list_id.to_string());
        } else {
            enabled_lists.remove(list_id);
        }

        let custom_rules = self
            .custom_rules
            .read()
            .map(|r| r.clone())
            .unwrap_or_default();
        let new_engine = Self::build_engine(&enabled_lists, &custom_rules);
        let total_rules = Self::count_rules(&enabled_lists, &custom_rules);

        if let Ok(mut engine) = self.engine.write() {
            *engine = new_engine;
        }
        if let Ok(mut stats) = self.stats.write() {
            stats.total_rules = total_rules;
        }
    }

    /// Get all available filter lists with their current state and rule counts
    pub fn get_filter_lists_info(&self) -> Vec<FilterListConfig> {
        let enabled_lists = self
            .enabled_lists
            .read()
            .map(|l| l.clone())
            .unwrap_or_default();

        vec![
            FilterListConfig {
                id: "easylist".to_string(),
                name: "EasyList".to_string(),
                description: "Primary filter list blocking advertisements across English & international websites.".to_string(),
                count: Self::get_easylist_rules().len(),
                enabled: enabled_lists.contains("easylist"),
            },
            FilterListConfig {
                id: "easyprivacy".to_string(),
                name: "EasyPrivacy".to_string(),
                description: "Eliminates web tracking, analytics telemetry, fingerprinting scripts, and beacons.".to_string(),
                count: Self::get_easyprivacy_rules().len(),
                enabled: enabled_lists.contains("easyprivacy"),
            },
            FilterListConfig {
                id: "ublock_filters".to_string(),
                name: "uBlock Filters – Ads & Annoyances".to_string(),
                description: "uBlock Origin core rules for high-frequency ad servers and cosmetic element cleanups.".to_string(),
                count: Self::get_ublock_core_rules().len(),
                enabled: enabled_lists.contains("ublock_filters"),
            },
            FilterListConfig {
                id: "ublock_badware".to_string(),
                name: "uBlock Filters – Badware & Malware".to_string(),
                description: "Protects against malicious domains, cryptominers, fraudulent popups, and scam redirects.".to_string(),
                count: Self::get_ublock_badware_rules().len(),
                enabled: enabled_lists.contains("ublock_badware"),
            },
            FilterListConfig {
                id: "ublock_privacy".to_string(),
                name: "uBlock Filters – Privacy & Telemetry".to_string(),
                description: "Specific defusers and blocks for advanced tracking techniques and browser telemetry.".to_string(),
                count: Self::get_ublock_privacy_rules().len(),
                enabled: enabled_lists.contains("ublock_privacy"),
            },
            FilterListConfig {
                id: "ublock_quick_fixes".to_string(),
                name: "uBlock Filters – Quick Fixes & YouTube".to_string(),
                description: "Rapidly updated rules for anti-adblock circumvention, YouTube video ads, and site breakage fixes.".to_string(),
                count: Self::get_ublock_quick_fixes_rules().len(),
                enabled: enabled_lists.contains("ublock_quick_fixes"),
            },
        ]
    }

    /// Retrieve live statistics
    pub fn get_stats(&self) -> AdblockStats {
        self.stats.read().map(|s| s.clone()).unwrap_or_default()
    }

    fn rebuild_engine_internal(&self, custom_rules: &[String]) {
        let enabled_lists = self
            .enabled_lists
            .read()
            .map(|l| l.clone())
            .unwrap_or_default();
        let new_engine = Self::build_engine(&enabled_lists, custom_rules);
        let total_rules = Self::count_rules(&enabled_lists, custom_rules);

        if let Ok(mut engine) = self.engine.write() {
            *engine = new_engine;
        }
        if let Ok(mut stats) = self.stats.write() {
            stats.total_rules = total_rules;
        }
    }

    // ==========================================
    // BUNDLED COMPREHENSIVE FILTER LIST DATA
    // ==========================================

    pub fn get_easylist_rules() -> Vec<String> {
        vec![
            // Google Ads & DoubleClick
            "||doubleclick.net^".into(),
            "||googleadservices.com^".into(),
            "||googlesyndication.com^".into(),
            "||adservice.google.com^".into(),
            "||pagead2.googlesyndication.com^".into(),
            "||partner.googleadservices.com^".into(),
            "||tpc.googlesyndication.com^".into(),
            "||2mdn.net^".into(),
            // Major Ad Exchanges & Networks
            "||adnxs.com^".into(),
            "||advertising.com^".into(),
            "||rubiconproject.com^".into(),
            "||pubmatic.com^".into(),
            "||criteo.com^".into(),
            "||criteo.net^".into(),
            "||openx.net^".into(),
            "||smartadserver.com^".into(),
            "||casalemedia.com^".into(),
            "||bidswitch.net^".into(),
            "||indexexchange.com^".into(),
            "||amazon-adsystem.com^".into(),
            "||adroll.com^".into(),
            "||media.net^".into(),
            "||moatads.com^".into(),
            "||outbrain.com^".into(),
            "||taboola.com^".into(),
            "||revcontent.com^".into(),
            "||mgid.com^".into(),
            "||inmobi.com^".into(),
            "||flashtalking.com^".into(),
            "||exponential.com^".into(),
            "||adform.net^".into(),
            "||adcolony.com^".into(),
            "||applovin.com^".into(),
            "||unityads.unity3d.com^".into(),
            "||vungle.com^".into(),
            "||chartbeat.com^".into(),
            "||scorecardresearch.com^".into(),
            "||zedo.com^".into(),
            "||adblade.com^".into(),
            "||yieldmo.com^".into(),
            "||sharethrough.com^".into(),
            "||triplelift.com^".into(),
            "||teads.tv^".into(),
            "||undertone.com^".into(),
            "||spotxchange.com^".into(),
            "||spotx.tv^".into(),
            "||sovrn.com^".into(),
            "||sonobi.com^".into(),
            "||gumgum.com^".into(),
            "||quantserve.com^".into(),
            "||quantcount.com^".into(),
            "||lijit.com^".into(),
            "||adtechus.com^".into(),
            "||tribalfusion.com^".into(),
            "||clicksor.com^".into(),
            "||buysellads.com^".into(),
            "||carbonads.net^".into(),
            "||serving-sys.com^".into(),
            "||bs.serving-sys.com^".into(),
            "||eyeota.net^".into(),
            "||simpli.fi^".into(),
            "||advertising.amazon.com^".into(),
            "||aax-us-east.amazon-adsystem.com^".into(),
            // Popunder, Push & Streaming Ad Networks
            "||monetag.com^".into(),
            "||monetag.net^".into(),
            "||propu.sh^".into(),
            "||onclickbright.com^".into(),
            "||onclicksuper.com^".into(),
            "||deloton.com^".into(),
            "||highperformancegate.com^".into(),
            "||adsterra.com^".into(),
            "||adsterratech.com^".into(),
            "||hilltopads.com^".into(),
            "||hilltopads.net^".into(),
            "||clickadu.com^".into(),
            "||clckr.com^".into(),
            "||popcash.net^".into(),
            "||popads.net^".into(),
            "||exoclick.com^".into(),
            "||juicyads.com^".into(),
            "||tsyndicate.com^".into(),
            "||ad-maven.com^".into(),
            "||trafficstars.com^".into(),
            "||etahub.com^".into(),
            "||bignox.com^".into(),
            "||histats.com^".into(),
            "||dtscout.com^".into(),
            "||al5smvpt45.com^".into(),
            "||feedify.net^".into(),
            "||ad-delivery.net^".into(),
            "||pushwoosh.com^".into(),
            "||pushassist.com^".into(),
            "||syndication.exoclick.com^".into(),
            "||trafficjunky.net^".into(),
            "||rtmark.net^".into(),
            "||in-page-push.com^".into(),
            // Path & Parameter Patterns
            "/ads.js".into(),
            "/advertisement.".into(),
            "/ad-banner.".into(),
            "/banner_ads/".into(),
            "/pagead/js/adsbygoogle.js".into(),
            "/pagead/show_ads.js".into(),
            "&ad_type=".into(),
            "&ad_url=".into(),
            "&adurl=".into(),
            "/wp-content/plugins/adrotate/".into(),
            "/wp-content/plugins/ad-inserter/".into(),
            // Generic Cosmetic Selectors: Ads, Banners & Sponsored Content
            "##ins.adsbygoogle".into(),
            "##[id^=\"google_ads_\"]".into(),
            "##[id*=\"google_ads_iframe\"]".into(),
            "##[id*=\"ScriptRoot\"]".into(),
            "##[class*=\"sponsored-post\"]".into(),
            "##[class*=\"ad-container\"]".into(),
            "##[class*=\"ad_container\"]".into(),
            "##[id*=\"banner-ad\"]".into(),
            "##[id*=\"ad-banner\"]".into(),
            "##[class*=\"ad-banner\"]".into(),
            "##[class*=\"ad-wrapper\"]".into(),
            "##[id*=\"ad-wrapper\"]".into(),
            "##[class*=\"ad-slot\"]".into(),
            "##[id*=\"ad-slot\"]".into(),
            "##[class*=\"ad-placement\"]".into(),
            "##[aria-label=\"advertisement\"]".into(),
            "##[aria-label=\"Sponsored\"]".into(),
            "##.trc_rbox_div".into(),
            "##.OUTBRAIN".into(),
            "##.taboola-placeholder".into(),
            // Generic Cosmetic Selectors: Social Floating Bars & Annoyance Docks
            "##[class*=\"share-bar\"]".into(),
            "##[class*=\"floating-share\"]".into(),
            "##[class*=\"shares-box\"]".into(),
            "##.social-share".into(),
            "##[id*=\"share-buttons\"]".into(),
            "##[class*=\"share-container\"]".into(),
            "##div[class*=\"share-sidebar\"]".into(),
            "##.at-share-dock".into(),
            "##.addthis_floating_style".into(),
            "##.sharethis-inline-share-buttons".into(),
            "##.st-sticky-share-buttons".into(),
            "##div[class*=\"shares\"]".into(),
            // Generic Cosmetic Selectors: Fake Captcha, Robot Verification, Scam Overlays & Modals
            "##[class*=\"captcha-modal\"]".into(),
            "##[id*=\"captcha-modal\"]".into(),
            "##[class*=\"robot-modal\"]".into(),
            "##[class*=\"robot-check\"]".into(),
            "##[class*=\"verify-robot\"]".into(),
            "##[id*=\"robot-check\"]".into(),
            "##[class*=\"notification-prompt\"]".into(),
            "##[class*=\"push-prompt\"]".into(),
            "##[class*=\"push-modal\"]".into(),
            "##[class*=\"ad-modal\"]".into(),
            "##[class*=\"popup-modal\"]".into(),
            "##[id*=\"popup-modal\"]".into(),
            "##[class*=\"interstitial\"]".into(),
            "##[id*=\"interstitial\"]".into(),
            "##[class*=\"overlay-backdrop\"]".into(),
            "##[id*=\"overlay-backdrop\"]".into(),
            "##[class*=\"ad-overlay\"]".into(),
            "##[id*=\"ad-overlay\"]".into(),
            "##div[style*=\"z-index: 2147483647\"]".into(),
            "##div[style*=\"z-index: 999999\"]".into(),
            "##div[style*=\"z-index: 99999\"]".into(),
        ]
    }

    pub fn get_easyprivacy_rules() -> Vec<String> {
        vec![
            // Google Analytics & Tag Manager
            "||google-analytics.com^".into(),
            "||analytics.google.com^".into(),
            "||googletagmanager.com^".into(),
            "||googletagservices.com^".into(),
            "||stats.g.doubleclick.net^".into(),
            // Microsoft Telemetry
            "||pipe.aria.microsoft.com^".into(),
            "||events.data.microsoft.com^".into(),
            "||telemetry.microsoft.com^".into(),
            "||watson.telemetry.microsoft.com^".into(),
            "||mobile.pipe.aria.microsoft.com^".into(),
            "||clarity.ms^".into(),
            // User Session & Heatmap Trackers
            "||hotjar.com^".into(),
            "||hotjar.io^".into(),
            "||static.hotjar.com^".into(),
            "||fullstory.com^".into(),
            "||fs.fullstory.com^".into(),
            "||mouseflow.com^".into(),
            "||luckyorange.com^".into(),
            "||crazyegg.com^".into(),
            "||inspectlet.com^".into(),
            "||logrocket.io^".into(),
            "||smartlook.com^".into(),
            // Product & Event Analytics
            "||segment.io^".into(),
            "||segment.com^".into(),
            "||api.segment.io^".into(),
            "||cdn.segment.com^".into(),
            "||mixpanel.com^".into(),
            "||api.mixpanel.com^".into(),
            "||amplitude.com^".into(),
            "||api.amplitude.com^".into(),
            "||heap.io^".into(),
            "||heapanalytics.com^".into(),
            "||cdn.heapanalytics.com^".into(),
            "||kissmetrics.com^".into(),
            "||woopra.com^".into(),
            "||branch.io^".into(),
            "||appsflyer.com^".into(),
            "||adjust.com^".into(),
            "||kochava.com^".into(),
            "||singular.net^".into(),
            // Error Tracking & Telemetry (Trackers mode)
            "||browser.sentry-cdn.com^".into(),
            "||datadoghq.com^$third-party".into(),
            "||browser-intake-datadoghq.com^".into(),
            "||loggly.com^".into(),
            "||bugsnag.com^".into(),
            "||crashlytics.com^".into(),
            // Facebook / Meta Pixels
            "||connect.facebook.net^*/fbevents.js".into(),
            "||facebook.com/tr/^".into(),
            "||pixel.facebook.com^".into(),
            // Twitter / X Tracking
            "||static.ads-twitter.com^".into(),
            "||ads-twitter.com^".into(),
            "||t.co^$third-party".into(),
            // TikTok & Pinterest Pixels
            "||analytics.tiktok.com^".into(),
            "||ct.pinterest.com^".into(),
            "||trk.pinterest.com^".into(),
            "||tr.snapchat.com^".into(),
            "||sc-static.net/scevent.min.js".into(),
            // Common Tracker URLs
            "/beacon.js".into(),
            "/tracker.js".into(),
            "/telemetry.js".into(),
            "/ping.gif".into(),
            "/pixel.gif".into(),
            "/collect?v=".into(),
        ]
    }

    pub fn get_ublock_core_rules() -> Vec<String> {
        vec![
            // uBlock Origin Core Resource Defusers & Shields
            "||popads.net^".into(),
            "||popcash.net^".into(),
            "||propellerads.com^".into(),
            "||adcash.com^".into(),
            "||adcash.net^".into(),
            "||onclickads.net^".into(),
            "||yllix.com^".into(),
            "||hilltopads.net^".into(),
            "||bidvertiser.com^".into(),
            "||exoclick.com^".into(),
            "||trafficjunky.com^".into(),
            "||juicyads.com^".into(),
            "||ero-advertising.com^".into(),
            // High-Value Site Specific Rules
            "youtube.com##ytd-promoted-video-renderer".into(),
            "youtube.com##ytd-promoted-sparkles-web-renderer".into(),
            "youtube.com##ytd-display-ad-renderer".into(),
            "youtube.com##ytd-statement-banner-renderer".into(),
            "youtube.com##ytd-in-feed-ad-layout-renderer".into(),
            "youtube.com##ytd-banner-promo-renderer".into(),
            "youtube.com###masthead-ad".into(),
            "youtube.com###player-ads".into(),
            "youtube.com###offer-module".into(),
            "youtube.com##.ytp-ad-overlay-container".into(),
            "youtube.com##.ytp-ad-message-container".into(),
            "youtube.com##.ytp-ad-overlay-slot".into(),
            "youtube.com##.ytp-ad-action-interstitial".into(),
            "youtube.com##.video-ads".into(),
            "youtube.com##.ytp-ad-module".into(),
            "youtube.com##tp-yt-paper-dialog:has(#feedback.ytd-enforcement-message-view-model)".into(),
            "reddit.com##.promotedlink".into(),
            "reddit.com##shreddit-comment-ad".into(),
            "reddit.com##[data-adclicklocation]".into(),
            "twitter.com##article:has(svg [d*=\"M19.498 3h-15C3.475 3 2.67 3.805 2.67 4.802v14.396\"])".into(),
            "x.com##article:has(svg [d*=\"M19.498 3h-15C3.475 3 2.67 3.805 2.67 4.802v14.396\"])".into(),
            "facebook.com##[data-pagelet*=\"FeedUnit_\"]:has(span:has-text(Sponsored))".into(),
            // Streaming / Movie Site Specific Rules
            "shuttletv.gd##.modal".into(),
            "shuttletv.gd##.backdrop".into(),
            "shuttletv.gd##[class*=\"share\"]".into(),
            "bobmovies.gd##.modal".into(),
            "bobmovies.gd##.backdrop".into(),
            "bobmovies.gd##[class*=\"share\"]".into(),
            "flixnetwork.to##.modal".into(),
            "flixnetwork.to##.backdrop".into(),
        ]
    }

    pub fn get_ublock_badware_rules() -> Vec<String> {
        vec![
            // Coin Miners & Cryptojackers
            "||coinhive.com^".into(),
            "||coin-hive.com^".into(),
            "||authedmine.com^".into(),
            "||cryptoloot.pro^".into(),
            "||webminepool.com^".into(),
            "||minr.pw^".into(),
            "||crypto-loot.com^".into(),
            // Scam & Malicious Redirect Networks
            "||trackvoluum.com^".into(),
            "||voluumtrk.com^".into(),
            "||zerohedge.bid^".into(),
            "||install.stream^".into(),
            "||pushcrew.com^".into(),
            "||pushengage.com^".into(),
            "||onesignal.com^$third-party".into(),
            "||webpushr.com^".into(),
            "||pushassist.com^".into(),
            // Fake update / Scareware popups
            "||system-update-center.com^".into(),
            "||critical-alert-center.com^".into(),
            "||mac-cleaner-alert.com^".into(),
            "||windows-defender-notice.com^".into(),
        ]
    }

    pub fn get_ublock_privacy_rules() -> Vec<String> {
        vec![
            // CNAME Cloaking & Telemetry Defusers
            "||cname.trackers.net^".into(),
            "||telemetry.*.com^".into(),
            "||stats.*.org^".into(),
            "||beacons.gcp.gvt2.com^".into(),
            "||app-measurement.com^".into(),
            "||firebase-logging.googleapis.com^".into(),
            "||firebaselogging-pa.googleapis.com^".into(),
            "||browser-intake-datadoghq.eu^".into(),
            "||sdk.split.io^".into(),
            "||app.launchdarkly.com^".into(),
            "||events.launchdarkly.com^".into(),
            "||clientstream.launchdarkly.com^".into(),
        ]
    }

    pub fn get_ublock_quick_fixes_rules() -> Vec<String> {
        vec![
            // Anti-Adblock Defusers & Fast YouTube fixes
            "||googlevideo.com/videoplayback$domain=youtube.com,xhr,redirect=noopjs".into(),
            "@@||youtube.com/api/stats/playback".into(),
            "@@||youtube.com/api/stats/delayplay".into(),
            "@@||youtube.com/api/stats/watchtime".into(),
            "@@||youtube.com/youtubei/v1/player".into(),
            "@@||youtube.com/youtubei/v1/next".into(),
            "||youtube.com/youtubei/v1/log_event^$xhr".into(),
            "||youtube.com/pagead/^".into(),
            "||youtube.com/api/stats/ads^".into(),
            "||youtube.com/api/stats/qoe?*&adformat=".into(),
            "||youtube.com/ptracking^".into(),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_initialization() {
        let manager = AdblockEngineManager::new();
        let stats = manager.get_stats();
        assert!(stats.total_rules > 100);
    }

    #[test]
    fn test_network_request_blocking() {
        let manager = AdblockEngineManager::new();

        // Should block doubleclick
        let decision = manager.check_network_request(
            "https://ad.doubleclick.net/ddm/trackclk/N1234",
            "https://www.example.com",
            "script",
        );
        assert!(decision.matched, "DoubleClick should be blocked");

        // Should block google-analytics
        let decision = manager.check_network_request(
            "https://www.google-analytics.com/analytics.js",
            "https://www.example.com",
            "script",
        );
        assert!(decision.matched, "Google Analytics should be blocked");

        // Legitimate site should not be blocked
        let decision = manager.check_network_request(
            "https://en.wikipedia.org/wiki/Rust_(programming_language)",
            "https://en.wikipedia.org",
            "main_frame",
        );
        assert!(!decision.matched, "Wikipedia should not be blocked");
    }

    #[test]
    fn test_cosmetic_resources() {
        let manager = AdblockEngineManager::new();
        let (hide_selectors, _) =
            manager.get_cosmetic_resources("https://www.youtube.com/watch?v=123");
        assert!(
            !hide_selectors.is_empty(),
            "Should return cosmetic rules for YouTube"
        );
    }

    #[test]
    fn test_custom_rule_management() {
        let manager = AdblockEngineManager::new();

        let custom_rule = "||custom-evil-tracker.example.org^".to_string();
        let added = manager.add_custom_rule(custom_rule.clone());
        assert!(added);

        let decision = manager.check_network_request(
            "https://custom-evil-tracker.example.org/pixel.png",
            "https://example.com",
            "image",
        );
        assert!(decision.matched, "Custom rule should block target request");

        let removed = manager.remove_custom_rule(&custom_rule);
        assert!(removed);

        let decision_after = manager.check_network_request(
            "https://custom-evil-tracker.example.org/pixel.png",
            "https://example.com",
            "image",
        );
        assert!(
            !decision_after.matched,
            "Custom rule should no longer match after removal"
        );
    }
}
