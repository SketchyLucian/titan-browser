interface Tab {
  id: number;
  url: string;
  title: string;
  is_loading: boolean;
  can_go_back: boolean;
  can_go_forward: boolean;
  is_private: boolean;
  favicon?: string;
}

interface Bookmark {
  id?: string;
  title: string;
  url: string;
  favicon?: string;
}

interface Module {
  id: string;
  name: string;
  description: string;
  enabled: boolean;
}

interface BlockedRequestLog {
  domain: string;
  url: string;
  req_type: string;
  timestamp: string;
}

interface FilterListConfig {
  id: string;
  name: string;
  description: string;
  count: number;
  enabled: boolean;
}

interface AdblockStats {
  total_rules: number;
  blocked_requests_count: number;
  cosmetic_elements_hidden_count: number;
  scriptlets_injected_count: number;
  estimated_bandwidth_saved_bytes: number;
}

type UpdateStatus = 'Idle' | 'Checking' | 'UpdateAvailable' | 'UpToDate' | 'Error';

interface UpdateState {
  current_version: string;
  latest_version?: string | null;
  release_url?: string | null;
  status: UpdateStatus;
  message: string;
}

interface BrowserSettings {
  search_engine: string;
  theme: string;
  accent_color: string;
  show_bookmarks_bar: boolean;
  do_not_track?: boolean;
  global_privacy_control?: boolean;
  strip_tracking_parameters?: boolean;
  block_webrtc_leak?: boolean;
  block_fingerprinting?: boolean;
  block_hyperlink_auditing?: boolean;
  telemetry_disabled?: boolean;
  blocked_domains?: string[];
  whitelisted_domains?: string[];
  adblock_enabled?: boolean;
  adblock_block_video_ads?: boolean;
  adblock_cosmetic_filtering?: boolean;
  adblock_block_popups?: boolean;
  adblock_aggressive_mode?: boolean;
  adblock_blocked_domains?: string[];
  adblock_whitelisted_domains?: string[];
  adblock_filter_lists?: string[];
  adblock_custom_rules?: string[];
  auto_update_enabled?: boolean;
}

interface ExtensionInfo {
  id: string;
  name: string;
  version: string;
  description: string;
  icon?: string | null;
  enabled: boolean;
  source: string;
  path: string;
  manifest_version: number;
  options_page?: string | null;
  popup_page?: string | null;
  homepage_url?: string | null;
  runtime_id?: string | null;
}

interface ExtensionPopupAnchor {
  x: number;
  y: number;
  width: number;
  height: number;
}

interface BrowserState {
  tabs: Tab[];
  active_tab_id?: number | null;
  activeTabId?: number | null;
  zoom: number;
  bookmarks: Bookmark[];
  modules: Module[];
  settings: BrowserSettings;
  searchEngine?: string;
  is_maximized?: boolean;
  blocked_logs?: BlockedRequestLog[];
  adblock_logs?: BlockedRequestLog[];
  adblock_filter_lists?: FilterListConfig[];
  adblock_stats?: AdblockStats;
  update_state?: UpdateState;
  extensions?: ExtensionInfo[];
}

interface SettingsInitState extends Partial<BrowserState> {
  active_section?: string;
  mandatory_blocked_domains?: string[];
  blocked_logs?: BlockedRequestLog[];
  adblock_logs?: BlockedRequestLog[];
  adblock_filter_lists?: FilterListConfig[];
  adblock_stats?: AdblockStats;
  adblock_custom_rules?: string[];
  extensions?: ExtensionInfo[];
}

type IpcOutMessage =
  | { type: 'UiReady' }
  | { type: 'NewTab'; url?: string }
  | { type: 'NewPrivateTab' }
  | { type: 'CloseTab'; tab_id: number }
  | { type: 'SwitchTab'; tab_id: number }
  | { type: 'Navigate'; url: string }
  | { type: 'GoBack' }
  | { type: 'GoForward' }
  | { type: 'Reload' }
  | { type: 'GoHome' }
  | { type: 'SetZoom'; zoom: number }
  | { type: 'ToggleBookmark'; title: string; url: string }
  | { type: 'RemoveBookmark'; url: string }
  | { type: 'ToggleModule'; module_id: string; enabled: boolean }
  | { type: 'SetTheme'; theme: string }
  | { type: 'SetAccentColor'; color: string }
  | { type: 'SetSearchEngine'; engine: string }
  | { type: 'SetShowBookmarksBar'; show: boolean }
  | { type: 'SetPrivacySetting'; key: string; enabled: boolean }
  | { type: 'SetAdblockSetting'; key: string; enabled: boolean }
  | { type: 'ClearBrowsingData'; cookies: boolean; cache: boolean; local_storage: boolean }
  | { type: 'AddBlockedDomain'; domain: string }
  | { type: 'RemoveBlockedDomain'; domain: string }
  | { type: 'AddWhitelistedDomain'; domain: string }
  | { type: 'RemoveWhitelistedDomain'; domain: string }
  | { type: 'ResetPrivacyRules' }
  | { type: 'AddAdblockDomain'; domain: string }
  | { type: 'RemoveAdblockDomain'; domain: string }
  | { type: 'AddAdblockWhitelist'; domain: string }
  | { type: 'RemoveAdblockWhitelist'; domain: string }
  | { type: 'ResetAdblockRules' }
  | { type: 'ClearAdblockLogs' }
  | { type: 'SetAutoUpdate'; enabled: boolean }
  | { type: 'CheckForUpdates' }
  | { type: 'OpenUpdateDownload' }
  | { type: 'ToggleFilterList'; list_id: string; enabled: boolean }
  | { type: 'AddCustomFilterRule'; rule: string }
  | { type: 'RemoveCustomFilterRule'; rule: string }
  | { type: 'ReportBlockedRequest'; domain: string; url: string; req_type: string }
  | { type: 'ReportBlockedAd'; domain: string; url: string; req_type: string }
  | { type: 'OpenThemes' }
  | { type: 'OpenPrivacy' }
  | { type: 'OpenAdblock' }
  | { type: 'OpenExtensions' }
  | { type: 'OpenSettings' }
  | { type: 'InstallExtension'; id_or_url: string; source?: string }
  | { type: 'UninstallExtension'; id: string }
  | { type: 'ToggleExtension'; id: string; enabled: boolean }
  | { type: 'LoadUnpackedExtension'; path: string }
  | { type: 'OpenExtensionOptions'; id: string }
  | { type: 'OpenExtensionPopup'; id: string; anchor?: ExtensionPopupAnchor }
  | { type: 'CloseExtensionPopup' }
  | { type: 'SetHeaderExpanded'; expanded: boolean }
  | { type: 'OpenHistory' }
  | { type: 'ClearHistory' }
  | { type: 'OpenDownloads' }
  | { type: 'ClearDownloads' }
  | { type: 'OpenDownload'; download_id: number }
  | { type: 'OpenDefaultBrowserSettings' }
  | { type: 'ShowBookmarkContextMenu'; url: string }
  | { type: 'TabStateUpdate'; tab_id: number; url: string; title: string; can_go_back: boolean; can_go_forward: boolean }
  | { type: 'DragWindow' }
  | { type: 'MinimizeWindow' }
  | { type: 'ToggleMaximizeWindow' }
  | { type: 'CloseWindow' };

interface Window {
  ipc?: {
    postMessage: (msg: string) => void;
  };
  __TITAN_BROWSER_STATE__?: BrowserState;
  updateBrowserState?: (state: BrowserState) => void;
  onBrowserState?: (state: Partial<BrowserState>) => void;
  onTabUpdate?: (tabUpdate: Partial<Tab> & { id: number }) => void;
  initSettings?: (state: SettingsInitState) => void;
  switchView?: (tabName: string) => void;
  changeSearchEngine?: (engine: string) => void;
  toggleBookmarksBar?: (show: boolean) => void;
  selectTheme?: (themeId: string) => void;
  selectAccent?: (color: string) => void;
  toggleDarkReader?: (enabled: boolean) => void;
  setPrivacySetting?: (key: string, enabled: boolean) => void;
  setAdblockSetting?: (key: string, enabled: boolean) => void;
  setAutoUpdate?: (enabled: boolean) => void;
  checkForUpdates?: () => void;
  openUpdateDownload?: () => void;
  clearBrowsingData?: () => void;
  addBlockedDomain?: () => void;
  removeBlockedDomain?: (domain: string) => void;
  addWhitelistedDomain?: () => void;
  removeWhitelistedDomain?: (domain: string) => void;
  resetPrivacyRules?: () => void;
  filterBlockedRules?: (query: string) => void;
  addAdblockDomain?: () => void;
  removeAdblockDomain?: (domain: string) => void;
  addAdblockWhitelist?: () => void;
  removeAdblockWhitelist?: (domain: string) => void;
  resetAdblockRules?: () => void;
  clearAdblockLogs?: () => void;
  filterAdblockRules?: (query: string) => void;
  installExtensionFromInput?: () => void;
  loadUnpackedExtensionPrompt?: () => void;
  toggleExtensionItem?: (id: string, enabled: boolean) => void;
  uninstallExtensionItem?: (id: string) => void;
  openExtensionOptionsItem?: (id: string) => void;
}
