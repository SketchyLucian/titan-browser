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
    pub is_private: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HistoryEntry {
    pub title: String,
    pub url: String,
    pub last_visited_ms: u64,
    pub visit_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SessionTab {
    pub url: String,
    pub title: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct BrowserSession {
    pub tabs: Vec<SessionTab>,
    pub active_index: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DownloadRecord {
    pub id: u64,
    pub url: String,
    pub file_path: Option<String>,
    pub status: String,
    pub started_ms: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ExtensionPopupAnchor {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
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
    #[serde(default = "default_filter_lists")]
    pub adblock_filter_lists: Vec<String>,
    #[serde(default)]
    pub adblock_custom_rules: Vec<String>,
    #[serde(default = "default_true")]
    pub auto_update_enabled: bool,
    #[serde(default)]
    pub privacy_migration_version: u8,
}

pub fn default_filter_lists() -> Vec<String> {
    vec![
        "easylist".into(),
        "easyprivacy".into(),
        "ublock_filters".into(),
        "ublock_badware".into(),
        "ublock_privacy".into(),
        "ublock_quick_fixes".into(),
        "turtlecute_test".into(),
    ]
}

fn default_true() -> bool {
    true
}

pub fn default_blocked_domains() -> Vec<String> {
    crate::privacy::BLOCKED_TELEMETRY_DOMAINS
        .iter()
        .map(|domain| (*domain).to_string())
        .collect()
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
            strip_tracking_parameters: false,
            block_webrtc_leak: false,
            block_fingerprinting: false,
            block_hyperlink_auditing: false,
            telemetry_disabled: false,
            blocked_domains: vec![],
            whitelisted_domains: vec![],
            adblock_enabled: false,
            adblock_block_video_ads: false,
            adblock_cosmetic_filtering: false,
            adblock_block_popups: false,
            adblock_aggressive_mode: false,
            adblock_blocked_domains: vec![],
            adblock_whitelisted_domains: vec![],
            adblock_filter_lists: vec![],
            adblock_custom_rules: vec![],
            auto_update_enabled: false,
            privacy_migration_version: 1,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
pub enum IpcIncoming {
    UiReady,
    NewTab {
        url: Option<String>,
    },
    NewPrivateTab,
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
    FocusAddressBar,
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
    SetAutoUpdate {
        enabled: bool,
    },
    CheckForUpdates,
    OpenUpdateDownload,
    ToggleFilterList {
        list_id: String,
        enabled: bool,
    },
    AddCustomFilterRule {
        rule: String,
    },
    RemoveCustomFilterRule {
        rule: String,
    },
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
    OpenHistory,
    ClearHistory,
    OpenDownloads,
    ClearDownloads,
    OpenDownload {
        download_id: u64,
    },
    OpenDefaultBrowserSettings,
    OpenThemes,
    OpenPrivacy,
    OpenAdblock,
    OpenExtensions,
    InstallExtension {
        id_or_url: String,
        source: Option<String>,
    },
    UninstallExtension {
        id: String,
    },
    ToggleExtension {
        id: String,
        enabled: bool,
    },
    LoadUnpackedExtension {
        path: String,
    },
    OpenExtensionOptions {
        id: String,
    },
    OpenExtensionPopup {
        id: String,
        #[serde(default)]
        anchor: Option<ExtensionPopupAnchor>,
    },
    ResizeExtensionPopup {
        width: f64,
        height: f64,
    },
    CloseExtensionPopup,
    SetHeaderExpanded {
        expanded: bool,
    },
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
        assert!(!settings.adblock_enabled);
        assert!(!settings.adblock_block_video_ads);
        assert!(!settings.adblock_cosmetic_filtering);
        assert!(!settings.adblock_block_popups);
        assert!(!settings.adblock_aggressive_mode);
        assert!(settings.adblock_blocked_domains.is_empty());
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

    #[test]
    fn parses_focus_address_bar_command() {
        let incoming = serde_json::from_str::<IpcIncoming>(r#"{"type":"FocusAddressBar"}"#);

        assert!(matches!(incoming, Ok(IpcIncoming::FocusAddressBar)));
    }
}
