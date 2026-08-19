use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bookmark {
    pub title: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserModule {
    pub id: String,
    pub name: String,
    pub description: String,
    pub icon: String,
    pub enabled: bool,
    pub stats: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcTabInfo {
    pub id: u32,
    pub url: String,
    pub title: String,
    pub is_loading: bool,
    pub can_go_back: bool,
    pub can_go_forward: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockedRequestLog {
    pub domain: String,
    pub url: String,
    pub req_type: String,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserSettings {
    pub theme: String,
    pub accent_color: String,
    pub search_engine: String,
    pub show_bookmarks_bar: bool,
    #[serde(default = "default_true")]
    pub do_not_track: bool,
    #[serde(default = "default_true")]
    pub global_privacy_control: bool,
    #[serde(default = "default_true")]
    pub strip_tracking_parameters: bool,
    #[serde(default = "default_true")]
    pub block_webrtc_leak: bool,
    #[serde(default = "default_true")]
    pub block_fingerprinting: bool,
    #[serde(default = "default_true")]
    pub block_hyperlink_auditing: bool,
    #[serde(default = "default_true")]
    pub telemetry_disabled: bool,
    #[serde(default = "default_blocked_domains")]
    pub blocked_domains: Vec<String>,
    #[serde(default)]
    pub whitelisted_domains: Vec<String>,
    #[serde(default = "default_true")]
    pub adblock_enabled: bool,
    #[serde(default = "default_true")]
    pub adblock_block_video_ads: bool,
    #[serde(default = "default_true")]
    pub adblock_cosmetic_filtering: bool,
    #[serde(default = "default_true")]
    pub adblock_block_popups: bool,
    #[serde(default)]
    pub adblock_aggressive_mode: bool,
    #[serde(default = "default_adblock_domains")]
    pub adblock_blocked_domains: Vec<String>,
    #[serde(default)]
    pub adblock_whitelisted_domains: Vec<String>,
}

fn default_true() -> bool {
    true
}

pub fn default_blocked_domains() -> Vec<String> {
    vec![
        "pipe.aria.microsoft.com".into(),
        "events.data.microsoft.com".into(),
        "telemetry.microsoft.com".into(),
        "watson.telemetry.microsoft.com".into(),
        "mobile.pipe.aria.microsoft.com".into(),
        "google-analytics.com".into(),
        "analytics.google.com".into(),
        "googletagmanager.com".into(),
        "stats.g.doubleclick.net".into(),
        "sentry.io".into(),
        "browser.sentry-cdn.com".into(),
        "segment.io".into(),
        "segment.com".into(),
        "mixpanel.com".into(),
        "amplitude.com".into(),
        "clarity.ms".into(),
        "hotjar.com".into(),
        "datadoghq.com".into(),
        "browser-intake-datadoghq.com".into(),
        "loggly.com".into(),
        "bugsnag.com".into(),
        "crashlytics.com".into(),
        "scorecardresearch.com".into(),
        "criteo.com".into(),
        "outbrain.com".into(),
        "taboola.com".into(),
    ]
}

pub fn default_adblock_domains() -> Vec<String> {
    vec![
        "doubleclick.net".into(),
        "googleadservices.com".into(),
        "googlesyndication.com".into(),
        "adservice.google.com".into(),
        "pagead2.googlesyndication.com".into(),
        "adnxs.com".into(),
        "advertising.com".into(),
        "rubiconproject.com".into(),
        "pubmatic.com".into(),
        "criteo.com".into(),
        "outbrain.com".into(),
        "taboola.com".into(),
        "popads.net".into(),
        "popcash.net".into(),
        "propellerads.com".into(),
        "adcash.com".into(),
        "bidswitch.net".into(),
        "casalemedia.com".into(),
        "openx.net".into(),
        "smartadserver.com".into(),
        "zedo.com".into(),
        "amazon-adsystem.com".into(),
        "adroll.com".into(),
        "media.net".into(),
        "moatads.com".into(),
        "quantserve.com".into(),
        "scorecardresearch.com".into(),
        "adform.net".into(),
        "ads-twitter.com".into(),
        "revcontent.com".into(),
        "mgid.com".into(),
        "inmobi.com".into(),
        "flashtalking.com".into(),
        "exponential.com".into(),
        "adcolony.com".into(),
        "unityads.unity3d.com".into(),
        "applovin.com".into(),
        "vungle.com".into(),
        "ironsrc.com".into(),
        "chartboost.com".into(),
        "adservice.com".into(),
        "adserver.com".into(),
    ]
}

impl Default for BrowserSettings {
    fn default() -> Self {
        Self {
            theme: "titan-dark".into(),
            accent_color: "#4e7cf6".into(),
            search_engine: "Google".into(),
            show_bookmarks_bar: false,
            do_not_track: true,
            global_privacy_control: true,
            strip_tracking_parameters: true,
            block_webrtc_leak: true,
            block_fingerprinting: true,
            block_hyperlink_auditing: true,
            telemetry_disabled: true,
            blocked_domains: default_blocked_domains(),
            whitelisted_domains: vec![],
            adblock_enabled: true,
            adblock_block_video_ads: true,
            adblock_cosmetic_filtering: true,
            adblock_block_popups: true,
            adblock_aggressive_mode: false,
            adblock_blocked_domains: default_adblock_domains(),
            adblock_whitelisted_domains: vec![],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcBrowserState {
    pub tabs: Vec<IpcTabInfo>,
    pub active_tab_id: Option<u32>,
    pub bookmarks: Vec<Bookmark>,
    pub modules: Vec<BrowserModule>,
    pub settings: BrowserSettings,
    pub zoom: f64,
    pub search_engine: String,
    pub is_maximized: bool,
    pub blocked_logs: Vec<BlockedRequestLog>,
    pub adblock_logs: Vec<BlockedRequestLog>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
pub enum IpcIncoming {
    UiReady,
    NewTab {
        url: Option<String>,
    },
    CloseTab {
        tab_id: u32,
    },
    SwitchTab {
        tab_id: u32,
    },
    Navigate {
        url: String,
    },
    GoBack,
    GoForward,
    Reload,
    GoHome,
    SetZoom {
        zoom: f64,
    },
    ToggleBookmark {
        title: String,
        url: String,
    },
    RemoveBookmark {
        url: String,
    },
    ShowBookmarkContextMenu {
        url: String,
    },
    ToggleModule {
        module_id: String,
        enabled: bool,
    },
    SetTheme {
        theme: String,
    },
    SetAccentColor {
        color: String,
    },
    SetSearchEngine {
        engine: String,
    },
    SetShowBookmarksBar {
        show: bool,
    },
    SetPrivacySetting {
        key: String,
        enabled: bool,
    },
    SetAdblockSetting {
        key: String,
        enabled: bool,
    },
    ClearBrowsingData {
        cookies: bool,
        cache: bool,
        local_storage: bool,
    },
    AddBlockedDomain {
        domain: String,
    },
    RemoveBlockedDomain {
        domain: String,
    },
    AddWhitelistedDomain {
        domain: String,
    },
    RemoveWhitelistedDomain {
        domain: String,
    },
    ResetPrivacyRules,
    AddAdblockDomain {
        domain: String,
    },
    RemoveAdblockDomain {
        domain: String,
    },
    AddAdblockWhitelist {
        domain: String,
    },
    RemoveAdblockWhitelist {
        domain: String,
    },
    ResetAdblockRules,
    ClearAdblockLogs,
    ReportBlockedRequest {
        domain: String,
        url: String,
        req_type: String,
    },
    ReportBlockedAd {
        domain: String,
        url: String,
        req_type: String,
    },
    OpenSettings,
    OpenThemes,
    OpenPrivacy,
    OpenAdblock,
    TabStateUpdate {
        tab_id: Option<u32>,
        url: String,
        title: String,
        can_go_back: Option<bool>,
        can_go_forward: Option<bool>,
    },
    DragWindow,
    MinimizeWindow,
    ToggleMaximizeWindow,
    CloseWindow,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adblock_default_settings() {
        let settings = BrowserSettings::default();
        assert!(settings.adblock_enabled);
        assert!(settings.adblock_block_video_ads);
        assert!(settings.adblock_cosmetic_filtering);
        assert!(settings.adblock_block_popups);
        assert!(!settings.adblock_aggressive_mode);
        assert!(!settings.adblock_blocked_domains.is_empty());
        assert!(settings.adblock_blocked_domains.contains(&"doubleclick.net".to_string()));
        assert!(settings.adblock_blocked_domains.contains(&"pagead2.googlesyndication.com".to_string()));
    }

    #[test]
    fn test_adblock_ipc_serialization() {
        let json_str = r#"{"type":"SetAdblockSetting","key":"adblock_enabled","enabled":false}"#;
        let incoming: Result<IpcIncoming, _> = serde_json::from_str(json_str);
        assert!(incoming.is_ok());
        if let Ok(IpcIncoming::SetAdblockSetting { key, enabled }) = incoming {
            assert_eq!(key, "adblock_enabled");
            assert!(!enabled);
        } else {
            panic!("Failed to parse SetAdblockSetting");
        }
    }
}
