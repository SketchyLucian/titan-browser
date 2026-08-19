"use strict";
// Titan Browser Client Chrome Controller (TypeScript)
(function () {
    let state = {
        tabs: [],
        activeTabId: 1,
        active_tab_id: 1,
        bookmarks: [],
        modules: [],
        settings: {
            theme: 'titan-dark',
            accent_color: '#4e7cf6',
            search_engine: 'Google',
            show_bookmarks_bar: false,
        },
        zoom: 1.0,
        searchEngine: 'Google',
        is_maximized: false,
    };
    // DOM Elements
    const tabsContainer = document.getElementById('tabsContainer');
    const newTabBtn = document.getElementById('newTabBtn');
    const backBtn = document.getElementById('backBtn');
    const forwardBtn = document.getElementById('forwardBtn');
    const reloadBtn = document.getElementById('reloadBtn');
    const homeBtn = document.getElementById('homeBtn');
    const urlForm = document.getElementById('urlForm');
    const urlInput = document.getElementById('urlInput');
    const sslBadge = document.getElementById('sslBadge');
    const bookmarkToggleBtn = document.getElementById('bookmarkToggleBtn');
    const bookmarksBar = document.getElementById('bookmarksBar');
    const bookmarksList = document.getElementById('bookmarksList');
    const zoomInBtn = document.getElementById('zoomInBtn');
    const zoomOutBtn = document.getElementById('zoomOutBtn');
    const zoomDisplay = document.getElementById('zoomDisplay');
    const windowDragRegion = document.getElementById('windowDragRegion');
    const tabStrip = document.getElementById('tabStrip');
    const winMinBtn = document.getElementById('winMinBtn');
    const winMaxBtn = document.getElementById('winMaxBtn');
    const winCloseBtn = document.getElementById('winCloseBtn');
    const settingsBtn = document.getElementById('settingsBtn');
    const brandLogo = document.getElementById('brandLogo');
    const searchEngineBadge = document.getElementById('searchEngineBadge');
    // Helper to send IPC messages to Rust
    function sendIpc(message) {
        if (window.ipc && window.ipc.postMessage) {
            window.ipc.postMessage(JSON.stringify(message));
        }
        else {
            console.log('IPC Out:', message);
        }
    }
    function getActiveTab() {
        const activeId = state.activeTabId ?? state.active_tab_id;
        return state.tabs.find((t) => t.id === activeId);
    }
    // Render Tabs
    function renderTabs() {
        if (!tabsContainer)
            return;
        tabsContainer.innerHTML = '';
        state.tabs.forEach((tab) => {
            const tabEl = document.createElement('div');
            const activeId = state.activeTabId ?? state.active_tab_id;
            const isActive = tab.id === activeId;
            tabEl.className = `tab-item ${isActive ? 'active' : ''} ${tab.is_loading ? 'loading' : ''}`;
            tabEl.dataset.id = String(tab.id);
            // Favicon or Spinner
            const faviconEl = document.createElement('div');
            faviconEl.className = 'tab-favicon';
            if (tab.is_loading) {
                faviconEl.innerHTML = `<svg class="spinner" viewBox="0 0 50 50"><circle class="path" cx="25" cy="25" r="20" fill="none" stroke-width="5"></circle></svg>`;
            }
            else if (tab.url.includes('youtube.com')) {
                faviconEl.innerHTML = `<svg viewBox="0 0 24 24" width="14" height="14" fill="#ff0000"><path d="M23.498 6.186a3.016 3.016 0 0 0-2.122-2.136C19.505 3.545 12 3.545 12 3.545s-7.505 0-9.377.505A3.017 3.017 0 0 0 .502 6.186C0 8.07 0 12 0 12s0 3.93.502 5.814a3.016 3.016 0 0 0 2.122 2.136c1.871.505 9.376.505 9.376.505s7.505 0 9.377-.505a3.015 3.015 0 0 0 2.122-2.136C24 15.93 24 12 24 12s0-3.93-.502-5.814zM9.545 15.568V8.432L15.818 12l-6.273 3.568z"/></svg>`;
            }
            else if (tab.url.includes('github.com')) {
                faviconEl.innerHTML = `<svg viewBox="0 0 24 24" width="14" height="14" fill="currentColor"><path d="M12 0C5.37 0 0 5.37 0 12c0 5.31 3.435 9.795 8.205 11.385.6.105.825-.255.825-.57 0-.285-.015-1.23-.015-2.235-3.015.555-3.795-.735-4.035-1.41-.135-.345-.72-1.41-1.23-1.695-.42-.225-1.02-.78-.015-.795.945-.015 1.62.87 1.845 1.23 1.08 1.815 2.805 1.305 3.495.99.105-.78.42-1.305.765-1.605-2.67-.3-5.46-1.335-5.46-5.925 0-1.305.465-2.385 1.23-3.225-.12-.3-.54-1.53.12-3.18 0 0 1.005-.315 3.3 1.23.96-.27 1.98-.405 3-.405s2.04.135 3 .405c2.295-1.56 3.3-1.23 3.3-1.23.66 1.65.24 2.88.12 3.18.765.84 1.23 1.905 1.23 3.225 0 4.605-2.805 5.625-5.475 5.925.435.375.81 1.095.81 2.22 0 1.605-.015 2.895-.015 3.3 0 .315.225.69.825.57A12.02 12.02 0 0 0 24 12c0-6.63-5.37-12-12-12z"/></svg>`;
            }
            else if (tab.url.startsWith('titan://settings') || tab.url.startsWith('titan://themes')) {
                faviconEl.innerHTML = `<svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="3"></circle><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"></path></svg>`;
            }
            else {
                faviconEl.innerHTML = `<svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"></circle><line x1="2" y1="12" x2="22" y2="12"></line><path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"></path></svg>`;
            }
            // Title
            const titleEl = document.createElement('span');
            titleEl.className = 'tab-title';
            titleEl.textContent = tab.title || (tab.url ? new URL(tab.url).hostname : 'New Tab');
            titleEl.title = tab.title || tab.url;
            // Close Button
            const closeBtn = document.createElement('button');
            closeBtn.className = 'tab-close-btn';
            closeBtn.title = 'Close Tab (Ctrl+W)';
            closeBtn.innerHTML = `<svg viewBox="0 0 16 16" width="10" height="10" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><path d="M4 4l8 8M12 4l-8 8"/></svg>`;
            closeBtn.addEventListener('click', (e) => {
                e.stopPropagation();
                sendIpc({ type: 'CloseTab', tab_id: tab.id });
            });
            tabEl.addEventListener('click', () => {
                if (urlInput)
                    urlInput.blur();
                sendIpc({ type: 'SwitchTab', tab_id: tab.id });
            });
            tabEl.appendChild(faviconEl);
            tabEl.appendChild(titleEl);
            tabEl.appendChild(closeBtn);
            tabsContainer.appendChild(tabEl);
        });
    }
    // Render Bookmarks
    function renderBookmarks() {
        if (!bookmarksBar || !bookmarksList)
            return;
        const show = state.settings.show_bookmarks_bar && state.bookmarks.length > 0;
        bookmarksBar.classList.toggle('visible', show);
        bookmarksList.innerHTML = '';
        state.bookmarks.forEach((bm) => {
            const item = document.createElement('div');
            item.className = 'bookmark-item';
            item.title = `${bm.title}\n${bm.url}`;
            const icon = document.createElement('span');
            icon.className = 'bookmark-icon';
            icon.innerHTML = `<svg viewBox="0 0 24 24" width="12" height="12" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"></circle></svg>`;
            const title = document.createElement('span');
            title.className = 'bookmark-title';
            title.textContent = bm.title || bm.url;
            item.appendChild(icon);
            item.appendChild(title);
            item.addEventListener('click', () => {
                sendIpc({ type: 'Navigate', url: bm.url });
            });
            item.addEventListener('contextmenu', (e) => {
                e.preventDefault();
                sendIpc({ type: 'ShowBookmarkContextMenu', url: bm.url });
            });
            bookmarksList.appendChild(item);
        });
    }
    // Update Navigation Bar State
    function updateNav() {
        const activeTab = getActiveTab();
        if (!activeTab)
            return;
        if (document.activeElement !== urlInput) {
            urlInput.value = activeTab.url === 'about:blank' ? '' : activeTab.url;
        }
        backBtn.disabled = !activeTab.can_go_back;
        forwardBtn.disabled = !activeTab.can_go_forward;
        // SSL Badge
        if (activeTab.url.startsWith('https://')) {
            sslBadge.className = 'ssl-badge secure';
            sslBadge.title = 'Secure Connection (HTTPS)';
        }
        else if (activeTab.url.startsWith('titan://')) {
            sslBadge.className = 'ssl-badge secure';
            sslBadge.title = 'Titan Internal Page';
        }
        else {
            sslBadge.className = 'ssl-badge warning';
            sslBadge.title = 'Not Secure';
        }
        // Bookmark Star State
        const isBookmarked = state.bookmarks.some((b) => b.url === activeTab.url);
        bookmarkToggleBtn.classList.toggle('active', isBookmarked);
        // Search Engine Badge
        if (searchEngineBadge) {
            searchEngineBadge.textContent = state.settings.search_engine || 'Google';
        }
    }
    // Apply Theme to Chrome UI
    function applyTheme(themeId, accentColor) {
        document.body.className = '';
        document.body.classList.add(`theme-${themeId}`);
        if (accentColor) {
            document.documentElement.style.setProperty('--accent-blue', accentColor);
        }
    }
    // Event Listeners
    brandLogo.addEventListener('click', () => {
        sendIpc({ type: 'OpenSettings' });
    });
    settingsBtn.addEventListener('click', () => {
        sendIpc({ type: 'OpenSettings' });
    });
    // Window Controls
    winMinBtn.addEventListener('click', (e) => {
        e.stopPropagation();
        sendIpc({ type: 'MinimizeWindow' });
    });
    winMaxBtn.addEventListener('click', (e) => {
        e.stopPropagation();
        sendIpc({ type: 'ToggleMaximizeWindow' });
    });
    winCloseBtn.addEventListener('click', (e) => {
        e.stopPropagation();
        sendIpc({ type: 'CloseWindow' });
    });
    // Window Dragging & Double-Click Maximize (strictly on empty titlebar drag region)
    function onDragStart(e) {
        if (e.target !== windowDragRegion)
            return;
        if (e.detail === 2) {
            sendIpc({ type: 'ToggleMaximizeWindow' });
        }
        else if (e.button === 0) {
            sendIpc({ type: 'DragWindow' });
        }
    }
    if (windowDragRegion) {
        windowDragRegion.addEventListener('mousedown', onDragStart);
    }
    // Navigation & Browser Actions
    newTabBtn.addEventListener('click', () => {
        if (urlInput)
            urlInput.blur();
        sendIpc({ type: 'NewTab', url: 'titan://newtab' });
    });
    backBtn.addEventListener('click', () => {
        sendIpc({ type: 'GoBack' });
    });
    forwardBtn.addEventListener('click', () => {
        sendIpc({ type: 'GoForward' });
    });
    reloadBtn.addEventListener('click', (e) => {
        e.currentTarget.blur();
        sendIpc({ type: 'Reload' });
    });
    homeBtn.addEventListener('click', (e) => {
        e.currentTarget.blur();
        sendIpc({ type: 'GoHome' });
    });
    urlForm.addEventListener('submit', (e) => {
        e.preventDefault();
        const query = urlInput.value.trim();
        if (query) {
            sendIpc({ type: 'Navigate', url: query });
            urlInput.blur();
        }
    });
    urlInput.addEventListener('focus', () => {
        urlInput.select();
    });
    urlInput.addEventListener('blur', () => {
        const activeTab = getActiveTab();
        if (activeTab && (!urlInput.value || urlInput.value.trim() === '')) {
            urlInput.value = activeTab.url || '';
        }
    });
    bookmarkToggleBtn.addEventListener('click', () => {
        const activeTab = getActiveTab();
        if (activeTab && activeTab.url) {
            sendIpc({
                type: 'ToggleBookmark',
                title: activeTab.title || activeTab.url,
                url: activeTab.url,
            });
        }
    });
    zoomInBtn.addEventListener('click', () => {
        state.zoom = Math.min(2.5, +(state.zoom + 0.1).toFixed(1));
        sendIpc({ type: 'SetZoom', zoom: state.zoom });
        zoomDisplay.textContent = `${Math.round(state.zoom * 100)}%`;
    });
    zoomOutBtn.addEventListener('click', () => {
        state.zoom = Math.max(0.5, +(state.zoom - 0.1).toFixed(1));
        sendIpc({ type: 'SetZoom', zoom: state.zoom });
        zoomDisplay.textContent = `${Math.round(state.zoom * 100)}%`;
    });
    // Global Keyboard Shortcuts
    window.addEventListener('keydown', (e) => {
        if (e.ctrlKey) {
            if (e.key === 't' || e.key === 'T') {
                e.preventDefault();
                sendIpc({ type: 'NewTab', url: 'titan://newtab' });
            }
            else if (e.key === 'w' || e.key === 'W') {
                e.preventDefault();
                const activeId = state.activeTabId ?? state.active_tab_id;
                if (activeId !== undefined && activeId !== null) {
                    sendIpc({ type: 'CloseTab', tab_id: activeId });
                }
            }
            else if (e.key === 'l' || e.key === 'L') {
                e.preventDefault();
                urlInput.focus();
                urlInput.select();
            }
            else if (e.key === 'r' || e.key === 'R') {
                e.preventDefault();
                sendIpc({ type: 'Reload' });
            }
            else if (e.key === ',' || e.key === '<') {
                e.preventDefault();
                sendIpc({ type: 'OpenSettings' });
            }
        }
    });
    // Exposed callbacks for Rust
    window.onBrowserState = function (newState) {
        state = { ...state, ...newState };
        if (newState.active_tab_id !== undefined) {
            state.activeTabId = newState.active_tab_id;
            state.active_tab_id = newState.active_tab_id;
        }
        if (state.settings) {
            applyTheme(state.settings.theme, state.settings.accent_color);
        }
        renderTabs();
        renderBookmarks();
        updateNav();
    };
    window.onTabUpdate = function (tabUpdate) {
        const tab = state.tabs.find((t) => t.id === tabUpdate.id);
        if (tab) {
            const titleChanged = tabUpdate.title && tabUpdate.title !== tab.title;
            const loadingChanged = tabUpdate.is_loading !== undefined && tabUpdate.is_loading !== tab.is_loading;
            Object.assign(tab, tabUpdate);
            if (titleChanged || loadingChanged) {
                const tabEl = tabsContainer.querySelector(`.tab-item[data-id="${tab.id}"]`);
                if (tabEl) {
                    const titleEl = tabEl.querySelector('.tab-title');
                    if (titleEl && tab.title) {
                        titleEl.textContent = tab.title;
                        titleEl.setAttribute('title', tab.title);
                    }
                    tabEl.classList.toggle('loading', !!tab.is_loading);
                }
            }
            const activeId = state.activeTabId ?? state.active_tab_id;
            if (tab.id === activeId) {
                updateNav();
            }
        }
    };
    // Notify backend UI is ready
    sendIpc({ type: 'UiReady' });
})();
