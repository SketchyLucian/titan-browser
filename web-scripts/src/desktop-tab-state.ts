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

    function postCommand(command) {
        try {
            window.ipc.postMessage(JSON.stringify(command));
        } catch (_) {
            // The native bridge is unavailable in ordinary web contexts.
        }
    }

    window.addEventListener('keydown', (event) => {
        const key = event.key.toLowerCase();

        if (event.altKey && !event.ctrlKey && !event.metaKey && !event.shiftKey) {
            if (key === 'arrowleft') {
                event.preventDefault();
                postCommand({ type: 'GoBack' });
            } else if (key === 'arrowright') {
                event.preventDefault();
                postCommand({ type: 'GoForward' });
            }
            return;
        }

        if (!event.ctrlKey || event.altKey || event.metaKey || event.repeat) return;

        if (key === 'n' && event.shiftKey) {
            event.preventDefault();
            postCommand({ type: 'NewPrivateTab' });
        } else if (key === 't' && !event.shiftKey) {
            event.preventDefault();
            postCommand({ type: 'NewTab', url: 'titan://newtab' });
        } else if (key === 'w' && !event.shiftKey) {
            event.preventDefault();
            postCommand({ type: 'CloseTab', tab_id: tabId });
        } else if (key === 'l') {
            event.preventDefault();
            postCommand({ type: 'FocusAddressBar' });
        } else if (key === 'r') {
            event.preventDefault();
            postCommand({ type: 'Reload' });
        } else if (key === 'h' && !event.shiftKey) {
            event.preventDefault();
            postCommand({ type: 'OpenHistory' });
        } else if (key === 'j' && !event.shiftKey) {
            event.preventDefault();
            postCommand({ type: 'OpenDownloads' });
        } else if (key === ',' && !event.shiftKey) {
            event.preventDefault();
            postCommand({ type: 'OpenSettings' });
        }
    }, true);

    window.addEventListener('pointerdown', () => {
        postCommand({ type: 'CloseExtensionPopup' });
    }, true);

    window.addEventListener('popstate', notify);
    window.addEventListener('load', notify);
    document.addEventListener('visibilitychange', notify);
    setTimeout(notify, 500);
})();
