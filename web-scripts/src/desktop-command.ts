// Titan Browser - Desktop Native-to-Web Command Dispatcher (TypeScript)

type TitanDesktopCommand =
  | { type: 'goBack' }
  | { type: 'goForward' }
  | { type: 'reload' }
  | { type: 'focusAddressBar' }
  | { type: 'setZoom'; zoom: number }
  | { type: 'initializeSettings'; state: unknown }
  | { type: 'switchSettingsView'; view: string }
  | { type: 'clearBrowsingData'; cookies: boolean; cache: boolean; localStorage: boolean }
  | { type: 'browserState'; state: unknown }
  | { type: 'tabUpdate'; tab: unknown };

declare const __TITAN_DESKTOP_COMMAND__: TitanDesktopCommand;

interface Window {
  initSettings?: (state: unknown) => void;
  switchView?: (view: string) => void;
  onBrowserState?: (state: unknown) => void;
  onTabUpdate?: (tab: unknown) => void;
}

(function () {
  const command = __TITAN_DESKTOP_COMMAND__;

  switch (command.type) {
    case 'goBack':
      window.history.back();
      break;
    case 'goForward':
      window.history.forward();
      break;
    case 'reload':
      window.location.reload();
      break;
    case 'focusAddressBar': {
      const addressBar = document.getElementById('urlInput') as HTMLInputElement | null;
      addressBar?.focus();
      addressBar?.select();
      break;
    }
    case 'setZoom':
      document.body.style.zoom = String(command.zoom);
      break;
    case 'initializeSettings':
      window.initSettings?.(command.state);
      break;
    case 'switchSettingsView':
      window.switchView?.(command.view);
      break;
    case 'browserState':
      window.onBrowserState?.(command.state);
      break;
    case 'tabUpdate':
      window.onTabUpdate?.(command.tab);
      break;
    case 'clearBrowsingData':
      if (command.localStorage) {
        try {
          window.localStorage.clear();
          window.sessionStorage.clear();
        } catch (_) {
          // Storage can be disabled for an origin.
        }
      }

      if (command.cookies) {
        try {
          document.cookie.split(';').forEach((cookie) => {
            document.cookie = cookie
              .replace(/^ +/, '')
              .replace(/=.*/, `=;expires=${new Date().toUTCString()};path=/`);
          });
        } catch (_) {
          // Cookie access can be disabled for an origin.
        }
      }

      if (command.cache && window.caches) {
        void caches.keys().then((keys) => Promise.all(keys.map((key) => caches.delete(key))));
      }
      break;
  }
})();
