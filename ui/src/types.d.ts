interface Tab {
  id: number;
  url: string;
  title: string;
  is_loading: boolean;
  can_go_back: boolean;
  can_go_forward: boolean;
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
}

interface SettingsInitState extends Partial<BrowserState> {
  active_section?: string;
  blocked_logs?: BlockedRequestLog[];
}

type IpcOutMessage =
  | { type: 'UiReady' }
  | { type: 'NewTab'; url?: string }
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
  | { type: 'ClearBrowsingData'; cookies: boolean; cache: boolean; local_storage: boolean }
  | { type: 'AddBlockedDomain'; domain: string }
  | { type: 'RemoveBlockedDomain'; domain: string }
  | { type: 'AddWhitelistedDomain'; domain: string }
  | { type: 'RemoveWhitelistedDomain'; domain: string }
  | { type: 'ResetPrivacyRules' }
  | { type: 'ReportBlockedRequest'; domain: string; url: string; req_type: string }
  | { type: 'OpenThemes' }
  | { type: 'OpenSettings' }
  | { type: 'OpenPrivacy' }
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
  clearBrowsingData?: () => void;
  addBlockedDomain?: () => void;
  removeBlockedDomain?: (domain: string) => void;
  addWhitelistedDomain?: () => void;
  removeWhitelistedDomain?: (domain: string) => void;
  resetPrivacyRules?: () => void;
  filterBlockedRules?: (query: string) => void;
}
