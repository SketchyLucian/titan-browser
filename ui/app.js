// Titan Browser Client Chrome Controller

(function () {
  let state = {
    tabs: [],
    activeTabId: 1,
    active_tab_id: 1,
    bookmarks: [],
    modules: [],
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
  const bookmarksList = document.getElementById('bookmarksList');
  const zoomInBtn = document.getElementById('zoomInBtn');
  const zoomOutBtn = document.getElementById('zoomOutBtn');
  const zoomDisplay = document.getElementById('zoomDisplay');
  const windowDragRegion = document.getElementById('windowDragRegion');
  const tabStrip = document.getElementById('tabStrip');
  const winMinBtn = document.getElementById('winMinBtn');
  const winMaxBtn = document.getElementById('winMaxBtn');
  const winCloseBtn = document.getElementById('winCloseBtn');
  const darkModeToggleBtn = document.getElementById('darkModeToggleBtn');

  // Helper to send IPC messages to Rust
  function sendIpc(message) {
    if (window.ipc && window.ipc.postMessage) {
      window.ipc.postMessage(JSON.stringify(message));
    } else {
      console.log('IPC Out:', message);
    }
  }

  function getActiveTab() {
    const activeId = state.activeTabId ?? state.active_tab_id;
    return state.tabs.find((t) => t.id === activeId);
  }

  // Render Tabs
  function renderTabs() {
    tabsContainer.innerHTML = '';
    const activeId = state.activeTabId ?? state.active_tab_id;

    state.tabs.forEach((tab) => {
      const tabEl = document.createElement('div');
      tabEl.className = `tab-item ${tab.id === activeId ? 'active' : ''} ${tab.is_loading ? 'loading' : ''}`;
      tabEl.setAttribute('data-id', tab.id);

      // Favicon / Special icons
      const isYt = tab.url && (tab.url.includes('youtube.com') || tab.url.includes('youtu.be'));
      const isSettings = tab.url && tab.url.startsWith('titan://');
      
      let iconHtml = `<svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"></circle><line x1="2" y1="12" x2="22" y2="12"></line><path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"></path></svg>`;
      if (isYt) {
        iconHtml = `<svg viewBox="0 0 24 24" width="14" height="14" fill="#ff4757"><path d="M23.498 6.186a3.016 3.016 0 0 0-2.122-2.136C19.505 3.545 12 3.545 12 3.545s-7.505 0-9.377.505A3.017 3.017 0 0 0 .502 6.186C0 8.07 0 12 0 12s0 3.93.502 5.814a3.016 3.016 0 0 0 2.122 2.136c1.871.505 9.376.505 9.376.505s7.505 0 9.377-.505a3.015 3.015 0 0 0 2.122-2.136C24 15.93 24 12 24 12s0-3.93-.502-5.814zM9.545 15.568V8.432L15.818 12l-6.273 3.568z"/></svg>`;
      } else if (isSettings) {
        iconHtml = `<svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="#38bdf8" stroke-width="2"><path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"/></svg>`;
      }

      tabEl.innerHTML = `
        <div class="tab-favicon">${iconHtml}</div>
        <div class="tab-spinner"></div>
        <div class="tab-title" title="${tab.title || tab.url || 'New Tab'}">${tab.title || 'New Tab'}</div>
        <button class="tab-close-btn" title="Close Tab (Ctrl+W)">
          <svg viewBox="0 0 24 24" width="12" height="12" stroke="currentColor" stroke-width="2.5" fill="none">
            <line x1="18" y1="6" x2="6" y2="18"></line>
            <line x1="6" y1="6" x2="18" y2="18"></line>
          </svg>
        </button>
      `;

      // Tab selection
      tabEl.addEventListener('click', (e) => {
        if (!e.target.closest('.tab-close-btn')) {
          state.activeTabId = tab.id;
          state.active_tab_id = tab.id;
          sendIpc({ type: 'SwitchTab', tab_id: tab.id });
          updateNav();
        }
      });

      // Tab close
      const closeBtn = tabEl.querySelector('.tab-close-btn');
      closeBtn.addEventListener('click', (e) => {
        e.stopPropagation();
        sendIpc({ type: 'CloseTab', tab_id: tab.id });
      });

      tabsContainer.appendChild(tabEl);
    });
  }

  // Render Bookmarks with Native Context Menu Support
  function renderBookmarks() {
    bookmarksList.innerHTML = '';
    state.bookmarks.forEach((bm) => {
      const isYt = bm.url && bm.url.includes('youtube.com');
      const bmEl = document.createElement('div');
      bmEl.className = `bookmark-item ${isYt ? 'yt-special' : ''}`;
      
      const iconSvg = isYt
        ? `<svg viewBox="0 0 24 24" fill="#ff4757"><path d="M10 15l5-3-5-3v6z"/><path d="M20 4H4c-1.1 0-2 .9-2 2v12c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V6c0-1.1-.9-2-2-2z"/></svg>`
        : `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/></svg>`;

      bmEl.innerHTML = `${iconSvg} <span>${bm.title}</span>`;
      bmEl.title = bm.url;

      // Left click -> navigate
      bmEl.addEventListener('click', () => {
        sendIpc({ type: 'Navigate', url: bm.url });
      });

      // Right click -> trigger native Windows context menu
      bmEl.addEventListener('contextmenu', (e) => {
        e.preventDefault();
        e.stopPropagation();
        sendIpc({ type: 'ShowBookmarkContextMenu', url: bm.url });
      });

      bookmarksList.appendChild(bmEl);
    });
  }

  // 1-Click Dark Mode Toggle
  darkModeToggleBtn.addEventListener('click', () => {
    const isDark = state.modules.some((m) => m.id === 'dark_reader' && m.enabled);
    sendIpc({
      type: 'ToggleModule',
      module_id: 'dark_reader',
      enabled: !isDark,
    });
  });

  // Update Navigation Bar State & URL Display
  function updateNav() {
    const activeTab = getActiveTab();
    if (!activeTab) return;

    backBtn.disabled = !activeTab.can_go_back;
    forwardBtn.disabled = !activeTab.can_go_forward;

    // Display the current URL in the Omnibar
    if (document.activeElement !== urlInput) {
      urlInput.value = activeTab.url || '';
    }

    // SSL Lock indication
    if (activeTab.url && (activeTab.url.startsWith('https://') || activeTab.url.startsWith('titan://'))) {
      sslBadge.className = 'ssl-indicator';
      sslBadge.title = 'Connection is secure';
    } else {
      sslBadge.className = 'ssl-indicator insecure';
      sslBadge.title = 'Insecure connection';
    }

    // Loading animation
    if (activeTab.is_loading) {
      reloadBtn.classList.add('loading');
    } else {
      reloadBtn.classList.remove('loading');
    }

    // Bookmark star state
    const isBookmarked = state.bookmarks.some((b) => b.url === activeTab.url);
    if (isBookmarked) {
      bookmarkToggleBtn.classList.add('bookmarked');
    } else {
      bookmarkToggleBtn.classList.remove('bookmarked');
    }

    // Dark Mode active button state
    const isDarkActive = state.modules.some((m) => m.id === 'dark_reader' && m.enabled);
    if (isDarkActive) {
      darkModeToggleBtn.classList.add('active');
      darkModeToggleBtn.title = 'Universal Dark Mode is ON (Click to turn off)';
    } else {
      darkModeToggleBtn.classList.remove('active');
      darkModeToggleBtn.title = 'Universal Dark Mode is OFF (Click to turn on)';
    }

    zoomDisplay.textContent = `${Math.round(state.zoom * 100)}%`;
  }

  // Window Controls Listeners
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

  // Window Dragging & Double-Click Maximize
  function onDragStart(e) {
    if (e.target.closest('button') || e.target.closest('input') || e.target.closest('.tab-item')) {
      return;
    }
    
    if (e.detail === 2) {
      sendIpc({ type: 'ToggleMaximizeWindow' });
    } else if (e.button === 0) {
      sendIpc({ type: 'DragWindow' });
    }
  }

  windowDragRegion.addEventListener('mousedown', onDragStart);
  tabStrip.addEventListener('mousedown', onDragStart);

  // Navigation & Browser Actions
  newTabBtn.addEventListener('click', () => {
    sendIpc({ type: 'NewTab', url: 'https://www.google.com' });
  });

  backBtn.addEventListener('click', () => {
    sendIpc({ type: 'GoBack' });
  });

  forwardBtn.addEventListener('click', () => {
    sendIpc({ type: 'GoForward' });
  });

  reloadBtn.addEventListener('click', () => {
    sendIpc({ type: 'Reload' });
  });

  homeBtn.addEventListener('click', () => {
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
        sendIpc({ type: 'NewTab', url: 'https://www.google.com' });
      } else if (e.key === 'w' || e.key === 'W') {
        e.preventDefault();
        const activeId = state.activeTabId ?? state.active_tab_id;
        if (activeId !== null) {
          sendIpc({ type: 'CloseTab', tab_id: activeId });
        }
      } else if (e.key === 'l' || e.key === 'L') {
        e.preventDefault();
        urlInput.focus();
        urlInput.select();
      } else if (e.key === 'r' || e.key === 'R') {
        e.preventDefault();
        sendIpc({ type: 'Reload' });
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
    renderTabs();
    renderBookmarks();
    updateNav();
  };

  window.onTabUpdate = function (tabUpdate) {
    const idx = state.tabs.findIndex((t) => t.id === tabUpdate.id);
    if (idx !== -1) {
      state.tabs[idx] = { ...state.tabs[idx], ...tabUpdate };
      renderTabs();
      updateNav();
    }
  };

  // Notify backend UI is ready
  sendIpc({ type: 'UiReady' });
})();
