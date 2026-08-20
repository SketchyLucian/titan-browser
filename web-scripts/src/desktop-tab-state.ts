// Titan Browser - Desktop Tab State Page Script (TypeScript)
// @ts-nocheck

interface TitanDesktopTabStateConfig {
  tabId: number;
}

declare const __TITAN_DESKTOP_TAB_STATE_CONFIG__: TitanDesktopTabStateConfig;

(function() {
    const config = __TITAN_DESKTOP_TAB_STATE_CONFIG__;
    const tabId = config.tabId;
    let lastUrl = '';
    let lastTitle = '';
    let notifyTimer = null;

    function notify() {
        clearTimeout(notifyTimer);
        notifyTimer = setTimeout(() => {
            const curUrl = window.location.href;
            const curTitle = document.title || window.location.hostname || 'New Tab';
            if (curUrl !== lastUrl || curTitle !== lastTitle) {
                lastUrl = curUrl;
                lastTitle = curTitle;
                try {
                    window.ipc.postMessage(JSON.stringify({
                        type: 'TabStateUpdate',
                        tab_id: tabId,
                        url: curUrl,
                        title: curTitle,
                        can_go_back: window.history.length > 1,
                        can_go_forward: true
                    }));
                } catch(e) {}
            }
        }, 400);
    }

    window.addEventListener('popstate', notify);
    window.addEventListener('load', notify);
    document.addEventListener('visibilitychange', notify);
    setTimeout(notify, 500);
})();
