// Titan Browser Client Chrome Controller (TypeScript)

(function () {
  let state: BrowserState = {
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
    extensions: [],
  };
  let renderedTabsKey = '';
  let renderedBookmarksKey = '';
  let renderedTheme = '';
  let renderedAccentColor = '';

  // DOM Elements
  const tabsContainer = document.getElementById('tabsContainer') as HTMLElement;
  const newTabBtn = document.getElementById('newTabBtn') as HTMLElement;
  const backBtn = document.getElementById('backBtn') as HTMLButtonElement;
  const forwardBtn = document.getElementById('forwardBtn') as HTMLButtonElement;
  const reloadBtn = document.getElementById('reloadBtn') as HTMLElement;
  const homeBtn = document.getElementById('homeBtn') as HTMLElement;
  const urlForm = document.getElementById('urlForm') as HTMLFormElement;
  const urlInput = document.getElementById('urlInput') as HTMLInputElement;
  const sslBadge = document.getElementById('sslBadge') as HTMLElement;
  const bookmarkToggleBtn = document.getElementById('bookmarkToggleBtn') as HTMLElement;
  const bookmarksBar = document.getElementById('bookmarksBar') as HTMLElement;
  const bookmarksList = document.getElementById('bookmarksList') as HTMLElement;
  const zoomInBtn = document.getElementById('zoomInBtn') as HTMLElement;
  const zoomOutBtn = document.getElementById('zoomOutBtn') as HTMLElement;
  const zoomDisplay = document.getElementById('zoomDisplay') as HTMLElement;
  const windowDragRegion = document.getElementById('windowDragRegion') as HTMLElement;
  const tabStrip = document.getElementById('tabStrip') as HTMLElement;
  const winMinBtn = document.getElementById('winMinBtn') as HTMLElement;
  const winMaxBtn = document.getElementById('winMaxBtn') as HTMLElement;
  const winCloseBtn = document.getElementById('winCloseBtn') as HTMLElement;
  const settingsBtn = document.getElementById('settingsBtn') as HTMLElement;
  const historyBtn = document.getElementById('historyBtn') as HTMLElement;
  const downloadsBtn = document.getElementById('downloadsBtn') as HTMLElement;
  const privateTabBtn = document.getElementById('privateTabBtn') as HTMLElement;
  const toolbarExtensionsList = document.getElementById('toolbarExtensionsList') as HTMLElement | null;
  const extensionsBtn = document.getElementById('extensionsBtn') as HTMLElement;
  const extensionsDropdown = document.getElementById('extensionsDropdown') as HTMLElement;
  const manageExtensionsBtn = document.getElementById('manageExtensionsBtn') as HTMLElement;
  const extensionsDropdownList = document.getElementById('extensionsDropdownList') as HTMLElement;
  const closeExtDropdownBtn = document.getElementById('closeExtDropdownBtn') as HTMLElement | null;
  const extensionDetailView = document.getElementById('extensionDetailView') as HTMLElement | null;
  const brandLogo = document.getElementById('brandLogo') as HTMLElement;
  const searchEngineBadge = document.getElementById('searchEngineBadge') as HTMLElement;

  // Helper to send IPC messages to Rust
  function sendIpc(message: IpcOutMessage) {
    if (window.ipc && window.ipc.postMessage) {
      window.ipc.postMessage(JSON.stringify(message));
    } else {
      console.log('IPC Out:', message);
    }
  }

  function getActiveTab(): Tab | undefined {
    const activeId = state.activeTabId ?? state.active_tab_id;
    return state.tabs.find((t) => t.id === activeId);
  }

  function getTabsRenderKey(): string {
    return state.tabs
      .map((tab) => `${tab.id}\u001f${tab.url}\u001f${tab.title}\u001f${tab.is_loading}\u001f${tab.is_private}`)
      .join('\u001e');
  }

  function getBookmarksRenderKey(show: boolean): string {
    const bookmarksKey = state.bookmarks
      .map((bookmark) => `${bookmark.url}\u001f${bookmark.title}`)
      .join('\u001e');
    return `${show}\u001d${bookmarksKey}`;
  }

  // Render Tabs
  function renderTabs() {
    if (!tabsContainer) return;

    const activeId = state.activeTabId ?? state.active_tab_id;
    const nextRenderKey = getTabsRenderKey();
    if (nextRenderKey === renderedTabsKey) {
      tabsContainer.querySelectorAll<HTMLElement>('.tab-item').forEach((tabEl) => {
        tabEl.classList.toggle('active', Number(tabEl.dataset.id) === activeId);
      });
      return;
    }

    renderedTabsKey = nextRenderKey;
    tabsContainer.innerHTML = '';

    state.tabs.forEach((tab) => {
      const tabEl = document.createElement('div');
      const isActive = tab.id === activeId;
      tabEl.className = `tab-item ${isActive ? 'active' : ''} ${tab.is_loading ? 'loading' : ''} ${tab.is_private ? 'private' : ''}`;
      tabEl.dataset.id = String(tab.id);

      // Favicon or Spinner
      const faviconEl = document.createElement('div');
      faviconEl.className = 'tab-favicon';
      if (tab.is_loading) {
        faviconEl.innerHTML = `<svg class="spinner" viewBox="0 0 50 50"><circle class="path" cx="25" cy="25" r="20" fill="none" stroke-width="5"></circle></svg>`;
      } else if (tab.is_private) {
        faviconEl.innerHTML = `<svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2"><rect x="5" y="10" width="14" height="11" rx="2"></rect><path d="M8 10V7a4 4 0 0 1 8 0v3"></path></svg>`;
      } else if (tab.url.includes('youtube.com')) {
        faviconEl.innerHTML = `<svg viewBox="0 0 24 24" width="14" height="14" fill="#ff0000"><path d="M23.498 6.186a3.016 3.016 0 0 0-2.122-2.136C19.505 3.545 12 3.545 12 3.545s-7.505 0-9.377.505A3.017 3.017 0 0 0 .502 6.186C0 8.07 0 12 0 12s0 3.93.502 5.814a3.016 3.016 0 0 0 2.122 2.136c1.871.505 9.376.505 9.376.505s7.505 0 9.377-.505a3.015 3.015 0 0 0 2.122-2.136C24 15.93 24 12 24 12s0-3.93-.502-5.814zM9.545 15.568V8.432L15.818 12l-6.273 3.568z"/></svg>`;
      } else if (tab.url.includes('github.com')) {
        faviconEl.innerHTML = `<svg viewBox="0 0 24 24" width="14" height="14" fill="currentColor"><path d="M12 0C5.37 0 0 5.37 0 12c0 5.31 3.435 9.795 8.205 11.385.6.105.825-.255.825-.57 0-.285-.015-1.23-.015-2.235-3.015.555-3.795-.735-4.035-1.41-.135-.345-.72-1.41-1.23-1.695-.42-.225-1.02-.78-.015-.795.945-.015 1.62.87 1.845 1.23 1.08 1.815 2.805 1.305 3.495.99.105-.78.42-1.305.765-1.605-2.67-.3-5.46-1.335-5.46-5.925 0-1.305.465-2.385 1.23-3.225-.12-.3-.54-1.53.12-3.18 0 0 1.005-.315 3.3 1.23.96-.27 1.98-.405 3-.405s2.04.135 3 .405c2.295-1.56 3.3-1.23 3.3-1.23.66 1.65.24 2.88.12 3.18.765.84 1.23 1.905 1.23 3.225 0 4.605-2.805 5.625-5.475 5.925.435.375.81 1.095.81 2.22 0 1.605-.015 2.895-.015 3.3 0 .315.225.69.825.57A12.02 12.02 0 0 0 24 12c0-6.63-5.37-12-12-12z"/></svg>`;
      } else if (tab.url.startsWith('titan://settings') || tab.url.startsWith('titan://themes') || tab.url.startsWith('titan://privacy') || tab.url.startsWith('titan://adblock') || tab.url.startsWith('titan://extensions')) {
        faviconEl.innerHTML = `<svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="3"></circle><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"></path></svg>`;
      } else {
        faviconEl.innerHTML = `<svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"></circle><line x1="2" y1="12" x2="22" y2="12"></line><path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"></path></svg>`;
      }

      // Title
      const titleEl = document.createElement('span');
      titleEl.className = 'tab-title';
      titleEl.textContent = tab.title || (tab.url ? new URL(tab.url).hostname : 'New Tab');
      titleEl.title = `${tab.is_private ? 'Private · ' : ''}${tab.title || tab.url}`;

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
        if (urlInput) urlInput.blur();
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
    if (!bookmarksBar || !bookmarksList) return;
    const show = state.settings.show_bookmarks_bar && state.bookmarks.length > 0;
    const nextRenderKey = getBookmarksRenderKey(show);
    if (nextRenderKey === renderedBookmarksKey) return;

    renderedBookmarksKey = nextRenderKey;
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

  // Render Extensions Dropdown
  function renderExtensionsDropdown() {
    if (!extensionsDropdownList) return;
    extensionsDropdownList.innerHTML = '';

    const exts = state.extensions || [];
    if (exts.length === 0) {
      const empty = document.createElement('div');
      empty.className = 'ext-drop-empty';
      empty.textContent = 'No extensions installed yet. Click "Manage" to install from Chrome Web Store or Edge Add-ons.';
      extensionsDropdownList.appendChild(empty);
      return;
    }

    exts.forEach((ext) => {
      const item = document.createElement('div');
      item.className = 'ext-drop-item';
      item.title = `${ext.name} (v${ext.version})\n${ext.description || ''}`;

      const icon = document.createElement('img');
      icon.className = 'ext-drop-icon';
      icon.src = ext.icon || 'data:image/svg+xml;utf8,<svg viewBox="0 0 24 24" fill="none" stroke="%234e7cf6" stroke-width="2" xmlns="http://www.w3.org/2000/svg"><path d="M20.5 11H19V7c0-1.1-.9-2-2-2h-4V3.5a2.5 2.5 0 0 0-5 0V5H4c-1.1 0-1.99.9-1.99 2v3.8H3.5c1.49 0 2.7 1.21 2.7 2.7s-1.21 2.7-2.7 2.7H2V20c0 1.1.9 2 2 2h3.8v-1.5c0-1.49 1.21-2.7 2.7-2.7 1.49 0 2.7 1.21 2.7 2.7V22H17c1.1 0 2-.9 2-2v-4h1.5a2.5 2.5 0 0 0 0-5z"/></svg>';

      const info = document.createElement('div');
      info.className = 'ext-drop-info';

      const name = document.createElement('div');
      name.className = 'ext-drop-name';
      name.textContent = ext.name;

      const ver = document.createElement('div');
      ver.className = 'ext-drop-ver';
      ver.textContent = `v${ext.version} · ${ext.enabled ? 'Enabled' : 'Disabled'}`;

      info.appendChild(name);
      info.appendChild(ver);

      item.appendChild(icon);
      item.appendChild(info);

      item.addEventListener('click', () => {
        if (extensionsDropdown) extensionsDropdown.style.display = 'none';
        if (ext.options_page) {
          sendIpc({ type: 'OpenExtensionOptions', id: ext.id });
        } else {
          sendIpc({ type: 'OpenExtensions' });
        }
      });

      extensionsDropdownList.appendChild(item);
    });
  }

  // Update Navigation Bar State
  function updateNav() {
    const activeTab = getActiveTab();
    if (!activeTab) return;

    if (document.activeElement !== urlInput) {
      const nextUrl = activeTab.url === 'about:blank' ? '' : activeTab.url;
      if (urlInput.value !== nextUrl) urlInput.value = nextUrl;
    }

    const backDisabled = !activeTab.can_go_back;
    const forwardDisabled = !activeTab.can_go_forward;
    if (backBtn.disabled !== backDisabled) backBtn.disabled = backDisabled;
    if (forwardBtn.disabled !== forwardDisabled) forwardBtn.disabled = forwardDisabled;

    // SSL Badge
    let sslClassName: string;
    let sslTitle: string;
    if (activeTab.url.startsWith('https://')) {
      sslClassName = 'ssl-badge secure';
      sslTitle = 'Secure Connection (HTTPS)';
    } else if (activeTab.url.startsWith('titan://')) {
      sslClassName = 'ssl-badge secure';
      sslTitle = 'Titan Internal Page';
    } else {
      sslClassName = 'ssl-badge warning';
      sslTitle = 'Not Secure';
    }
    if (sslBadge.className !== sslClassName) sslBadge.className = sslClassName;
    if (sslBadge.title !== sslTitle) sslBadge.title = sslTitle;

    // Bookmark Star State
    const isBookmarked = state.bookmarks.some((b) => b.url === activeTab.url);
    bookmarkToggleBtn.classList.toggle('active', isBookmarked);

    // Search Engine Badge
    if (searchEngineBadge) {
      const nextSearchEngine = state.settings.search_engine || 'Google';
      if (searchEngineBadge.textContent !== nextSearchEngine) {
        searchEngineBadge.textContent = nextSearchEngine;
      }
    }

    renderToolbarExtensions();
    if (isExtensionDropdownOpen) {
      renderExtensionsDropdownList();
    }
  }

  let isExtensionDropdownOpen = false;

  function getActiveDomain(): string {
    const activeTab = getActiveTab();
    if (!activeTab || !activeTab.url) return 'current website';
    try {
      const parsed = new URL(activeTab.url);
      return parsed.hostname || 'current website';
    } catch {
      return 'current website';
    }
  }

  function openExtensionsDropdown(focusedExtensionId?: string) {
    if (!extensionsDropdown) return;
    isExtensionDropdownOpen = true;
    extensionsDropdown.style.display = 'flex';
    sendIpc({ type: 'SetHeaderExpanded', expanded: true });

    const extensions = state.extensions || [];
    if (focusedExtensionId) {
      const ext = extensions.find((e) => e.id === focusedExtensionId);
      if (ext) {
        renderExtensionDetailView(ext);
        return;
      }
    }
    renderExtensionsDropdownList();
  }

  function closeExtensionsDropdown() {
    if (!extensionsDropdown || !isExtensionDropdownOpen) return;
    isExtensionDropdownOpen = false;
    extensionsDropdown.style.display = 'none';
    sendIpc({ type: 'SetHeaderExpanded', expanded: false });
  }

  function toggleExtensionsDropdown() {
    if (isExtensionDropdownOpen) {
      closeExtensionsDropdown();
    } else {
      openExtensionsDropdown();
    }
  }

  function renderExtensionsDropdownList() {
    if (!extensionsDropdownList || !extensionDetailView) return;
    extensionsDropdownList.style.display = 'flex';
    extensionDetailView.style.display = 'none';
    extensionsDropdownList.innerHTML = '';

    const extensions = state.extensions || [];
    if (extensions.length === 0) {
      const empty = document.createElement('div');
      empty.className = 'ext-drop-empty';
      empty.textContent = 'No extensions installed yet. Click Manage to discover Add-ons.';
      extensionsDropdownList.appendChild(empty);
      return;
    }

    extensions.forEach((ext) => {
      const item = document.createElement('div');
      item.className = 'ext-drop-item';
      item.title = `Open ${ext.name} popup`;

      const icon = document.createElement('img');
      icon.className = 'ext-drop-icon';
      icon.src =
        ext.icon ||
        'data:image/svg+xml;utf8,<svg viewBox="0 0 24 24" fill="none" stroke="%234e7cf6" stroke-width="2" xmlns="http://www.w3.org/2000/svg"><path d="M20.5 11H19V7c0-1.1-.9-2-2-2h-4V3.5a2.5 2.5 0 0 0-5 0V5H4c-1.1 0-1.99.9-1.99 2v3.8H3.5c1.49 0 2.7 1.21 2.7 2.7s-1.21 2.7-2.7 2.7H2V20c0 1.1.9 2 2 2h3.8v-1.5c0-1.49 1.21-2.7 2.7-2.7 1.49 0 2.7 1.21 2.7 2.7V22H17c1.1 0 2-.9 2-2v-4h1.5a2.5 2.5 0 0 0 0-5z"/></svg>';

      const info = document.createElement('div');
      info.className = 'ext-drop-info';

      const name = document.createElement('div');
      name.className = 'ext-drop-name';
      name.textContent = ext.name;

      const ver = document.createElement('div');
      ver.className = 'ext-drop-ver';
      ver.textContent = `v${ext.version} • ${ext.enabled ? 'Active' : 'Disabled'}`;

      info.appendChild(name);
      info.appendChild(ver);

      item.appendChild(icon);
      item.appendChild(info);

      item.addEventListener('click', (e) => {
        e.stopPropagation();
        renderExtensionDetailView(ext);
      });

      extensionsDropdownList.appendChild(item);
    });
  }

  function renderExtensionDetailView(ext: ExtensionInfo) {
    if (!extensionsDropdownList || !extensionDetailView) return;
    extensionsDropdownList.style.display = 'none';
    extensionDetailView.style.display = 'flex';
    extensionDetailView.innerHTML = '';

    const domain = getActiveDomain();
    const isUblock = ext.name.toLowerCase().includes('ublock') || ext.name.toLowerCase().includes('ubo');
    const isBitwarden = ext.name.toLowerCase().includes('bitwarden');

    // Header with back button
    const header = document.createElement('div');
    header.className = 'ext-detail-header';

    const backBtn = document.createElement('button');
    backBtn.className = 'ext-back-btn';
    backBtn.title = 'Back to all extensions';
    backBtn.innerHTML = '←';
    backBtn.addEventListener('click', (e) => {
      e.stopPropagation();
      renderExtensionsDropdownList();
    });

    const icon = document.createElement('img');
    icon.className = 'ext-drop-icon';
    icon.src =
      ext.icon ||
      'data:image/svg+xml;utf8,<svg viewBox="0 0 24 24" fill="none" stroke="%234e7cf6" stroke-width="2" xmlns="http://www.w3.org/2000/svg"><path d="M20.5 11H19V7c0-1.1-.9-2-2-2h-4V3.5a2.5 2.5 0 0 0-5 0V5H4c-1.1 0-1.99.9-1.99 2v3.8H3.5c1.49 0 2.7 1.21 2.7 2.7s-1.21 2.7-2.7 2.7H2V20c0 1.1.9 2 2 2h3.8v-1.5c0-1.49 1.21-2.7 2.7-2.7 1.49 0 2.7 1.21 2.7 2.7V22H17c1.1 0 2-.9 2-2v-4h1.5a2.5 2.5 0 0 0 0-5z"/></svg>';

    const titleGroup = document.createElement('div');
    titleGroup.className = 'ext-detail-title-group';

    const name = document.createElement('div');
    name.className = 'ext-detail-name';
    name.textContent = ext.name;

    const status = document.createElement('div');
    status.className = 'ext-detail-status';
    status.textContent = ext.enabled ? '● Active on this site' : '○ Disabled';

    titleGroup.appendChild(name);
    titleGroup.appendChild(status);

    header.appendChild(backBtn);
    header.appendChild(icon);
    header.appendChild(titleGroup);
    extensionDetailView.appendChild(header);

    if (isUblock) {
      // uBlock Origin interactive popup
      const powerCard = document.createElement('div');
      powerCard.className = 'ext-power-card';

      const powerBtn = document.createElement('button');
      powerBtn.className = `ext-power-btn ${ext.enabled ? 'active' : ''}`;
      powerBtn.title = 'Toggle protection for this site';
      powerBtn.innerHTML = `<svg viewBox="0 0 24 24" width="24" height="24" stroke="currentColor" stroke-width="2.5" fill="none"><path d="M18.36 6.64a9 9 0 1 1-12.73 0"></path><line x1="12" y1="2" x2="12" y2="12"></line></svg>`;
      powerBtn.addEventListener('click', (e) => {
        e.stopPropagation();
        ext.enabled = !ext.enabled;
        sendIpc({ type: 'ToggleExtension', id: ext.id, enabled: ext.enabled });
        renderExtensionDetailView(ext);
      });

      const label = document.createElement('div');
      label.className = 'ext-power-label';
      label.textContent = ext.enabled ? 'Shield Protection Enabled' : 'Protection Paused';

      const domainLabel = document.createElement('div');
      domainLabel.className = 'ext-power-domain';
      domainLabel.textContent = `for ${domain}`;

      const statsRow = document.createElement('div');
      statsRow.className = 'ext-stats-row';

      const blockedCount = (state.adblock_logs || []).length || 0;
      statsRow.innerHTML = `
        <div class="ext-stat-item">
          <span class="ext-stat-val">${blockedCount}</span>
          <span class="ext-stat-lbl">Blocked on page</span>
        </div>
        <div class="ext-stat-item">
          <span class="ext-stat-val">Active</span>
          <span class="ext-stat-lbl">Filter Rules</span>
        </div>
      `;

      powerCard.appendChild(powerBtn);
      powerCard.appendChild(label);
      powerCard.appendChild(domainLabel);
      powerCard.appendChild(statsRow);
      extensionDetailView.appendChild(powerCard);
    } else if (isBitwarden) {
      // Bitwarden interactive popup
      const passBox = document.createElement('div');
      passBox.className = 'ext-power-card';

      const domainTitle = document.createElement('div');
      domainTitle.className = 'ext-power-label';
      domainTitle.textContent = `Vault: ${domain}`;

      const genBox = document.createElement('div');
      genBox.className = 'ext-pass-gen-box';
      genBox.style.width = '100%';

      const generatePassword = () => {
        const chars = 'abcdefghjkmnpqrstuvwxyzABCDEFGHJKLMNPQRSTUVWXYZ23456789!@#$%^&*';
        let pass = '';
        for (let i = 0; i < 16; i++) {
          pass += chars.charAt(Math.floor(Math.random() * chars.length));
        }
        return pass;
      };

      const passText = document.createElement('span');
      passText.className = 'ext-pass-text';
      passText.textContent = generatePassword();

      const copyBtn = document.createElement('button');
      copyBtn.className = 'ext-pass-copy-btn';
      copyBtn.textContent = 'Copy';
      copyBtn.addEventListener('click', (e) => {
        e.stopPropagation();
        navigator.clipboard.writeText(passText.textContent || '');
        copyBtn.textContent = 'Copied!';
        setTimeout(() => { copyBtn.textContent = 'Copy'; }, 1500);
      });

      genBox.appendChild(passText);
      genBox.appendChild(copyBtn);

      passBox.appendChild(domainTitle);
      passBox.appendChild(genBox);
      extensionDetailView.appendChild(passBox);
    }

    // Quick action buttons
    const actions = document.createElement('div');
    actions.className = 'ext-quick-actions';

    if (ext.options_page) {
      const optBtn = document.createElement('button');
      optBtn.className = 'ext-quick-btn';
      optBtn.innerHTML = `⚙️ Open Dashboard / Options`;
      optBtn.addEventListener('click', (e) => {
        e.stopPropagation();
        closeExtensionsDropdown();
        sendIpc({ type: 'OpenExtensionOptions', id: ext.id });
      });
      actions.appendChild(optBtn);
    }

    const manageBtn = document.createElement('button');
    manageBtn.className = 'ext-quick-btn';
    manageBtn.innerHTML = `🧩 Manage in Settings`;
    manageBtn.addEventListener('click', (e) => {
      e.stopPropagation();
      closeExtensionsDropdown();
      sendIpc({ type: 'OpenExtensions' });
    });
    actions.appendChild(manageBtn);

    extensionDetailView.appendChild(actions);
  }

  function renderToolbarExtensions() {
    if (!toolbarExtensionsList) return;
    const extensions = (state.extensions || []).filter((ext) => ext.enabled);
    toolbarExtensionsList.innerHTML = '';
    extensions.forEach((ext) => {
      const btn = document.createElement('button');
      btn.className = 'toolbar-ext-btn';
      btn.title = `${ext.name} (Click to open popup)`;
      const iconImg = document.createElement('img');
      iconImg.className = 'toolbar-ext-icon';
      iconImg.src =
        ext.icon ||
        'data:image/svg+xml;utf8,<svg viewBox="0 0 24 24" fill="none" stroke="%234e7cf6" stroke-width="2" xmlns="http://www.w3.org/2000/svg"><path d="M20.5 11H19V7c0-1.1-.9-2-2-2h-4V3.5a2.5 2.5 0 0 0-5 0V5H4c-1.1 0-1.99.9-1.99 2v3.8H3.5c1.49 0 2.7 1.21 2.7 2.7s-1.21 2.7-2.7 2.7H2V20c0 1.1.9 2 2 2h3.8v-1.5c0-1.49 1.21-2.7 2.7-2.7 1.49 0 2.7 1.21 2.7 2.7V22H17c1.1 0 2-.9 2-2v-4h1.5a2.5 2.5 0 0 0 0-5z"/></svg>';
      btn.appendChild(iconImg);
      btn.addEventListener('click', (e) => {
        e.stopPropagation();
        openExtensionsDropdown(ext.id);
      });
      toolbarExtensionsList.appendChild(btn);
    });
  }

  // Apply Theme to Chrome UI
  function applyTheme(themeId: string, accentColor?: string) {
    if (themeId !== renderedTheme) {
      document.body.className = `theme-${themeId}`;
      renderedTheme = themeId;
    }
    if (accentColor && accentColor !== renderedAccentColor) {
      document.documentElement.style.setProperty('--accent-blue', accentColor);
      renderedAccentColor = accentColor;
    }
  }

  // Event Listeners
  brandLogo.addEventListener('click', () => {
    sendIpc({ type: 'OpenSettings' });
  });

  settingsBtn.addEventListener('click', () => {
    sendIpc({ type: 'OpenSettings' });
  });

  historyBtn.addEventListener('click', () => {
    sendIpc({ type: 'OpenHistory' });
  });

  downloadsBtn.addEventListener('click', () => {
    sendIpc({ type: 'OpenDownloads' });
  });

  privateTabBtn.addEventListener('click', () => {
    sendIpc({ type: 'NewPrivateTab' });
  });

  if (extensionsBtn) {
    extensionsBtn.addEventListener('click', (e) => {
      e.stopPropagation();
      toggleExtensionsDropdown();
    });
  }

  if (closeExtDropdownBtn) {
    closeExtDropdownBtn.addEventListener('click', (e) => {
      e.stopPropagation();
      closeExtensionsDropdown();
    });
  }

  if (manageExtensionsBtn) {
    manageExtensionsBtn.addEventListener('click', (e) => {
      e.stopPropagation();
      closeExtensionsDropdown();
      sendIpc({ type: 'OpenExtensions' });
    });
  }

  // Close dropdown on click outside
  window.addEventListener('click', (e) => {
    if (!isExtensionDropdownOpen) return;
    const target = e.target as HTMLElement | null;
    if (
      target &&
      !target.closest('#extensionsDropdown') &&
      !target.closest('#extensionsBtn') &&
      !target.closest('.toolbar-ext-btn')
    ) {
      closeExtensionsDropdown();
    }
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
  function onDragStart(e: MouseEvent) {
    if (e.target !== windowDragRegion) return;

    if (e.detail === 2) {
      sendIpc({ type: 'ToggleMaximizeWindow' });
    } else if (e.button === 0) {
      sendIpc({ type: 'DragWindow' });
    }
  }

  if (windowDragRegion) {
    windowDragRegion.addEventListener('mousedown', onDragStart);
  }

  // Navigation & Browser Actions
  newTabBtn.addEventListener('click', () => {
    if (urlInput) urlInput.blur();
    sendIpc({ type: 'NewTab', url: 'titan://newtab' });
  });

  backBtn.addEventListener('click', () => {
    sendIpc({ type: 'GoBack' });
  });

  forwardBtn.addEventListener('click', () => {
    sendIpc({ type: 'GoForward' });
  });

  reloadBtn.addEventListener('click', (e) => {
    (e.currentTarget as HTMLElement).blur();
    sendIpc({ type: 'Reload' });
  });

  homeBtn.addEventListener('click', (e) => {
    (e.currentTarget as HTMLElement).blur();
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
  window.addEventListener('keydown', (e: KeyboardEvent) => {
    if (e.altKey && !e.ctrlKey && !e.metaKey && !e.shiftKey) {
      if (e.key === 'ArrowLeft') {
        e.preventDefault();
        sendIpc({ type: 'GoBack' });
      } else if (e.key === 'ArrowRight') {
        e.preventDefault();
        sendIpc({ type: 'GoForward' });
      }
      return;
    }

    if (e.ctrlKey) {
      if (e.shiftKey && (e.key === 'n' || e.key === 'N')) {
        e.preventDefault();
        sendIpc({ type: 'NewPrivateTab' });
      } else if (e.key === 't' || e.key === 'T') {
        e.preventDefault();
        sendIpc({ type: 'NewTab', url: 'titan://newtab' });
      } else if (e.key === 'w' || e.key === 'W') {
        e.preventDefault();
        const activeId = state.activeTabId ?? state.active_tab_id;
        if (activeId !== undefined && activeId !== null) {
          sendIpc({ type: 'CloseTab', tab_id: activeId });
        }
      } else if (e.key === 'l' || e.key === 'L') {
        e.preventDefault();
        urlInput.focus();
        urlInput.select();
      } else if (e.key === 'r' || e.key === 'R') {
        e.preventDefault();
        sendIpc({ type: 'Reload' });
      } else if (e.key === 'h' || e.key === 'H') {
        e.preventDefault();
        sendIpc({ type: 'OpenHistory' });
      } else if (e.key === 'j' || e.key === 'J') {
        e.preventDefault();
        sendIpc({ type: 'OpenDownloads' });
      } else if (e.key === ',' || e.key === '<') {
        e.preventDefault();
        sendIpc({ type: 'OpenSettings' });
      }
    }
  });

  // Exposed callbacks for Rust
  window.onBrowserState = function (newState: Partial<BrowserState>) {
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
    renderExtensionsDropdown();
    updateNav();
  };

  window.onTabUpdate = function (tabUpdate: Partial<Tab> & { id: number }) {
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
      renderedTabsKey = getTabsRenderKey();

      const activeId = state.activeTabId ?? state.active_tab_id;
      if (tab.id === activeId) {
        updateNav();
      }
    }
  };

  // Notify backend UI is ready
  sendIpc({ type: 'UiReady' });
})();
