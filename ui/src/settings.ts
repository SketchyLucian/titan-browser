// Titan Browser - Settings Controller (TypeScript)

(function () {
  let viewEntryAnimation: Animation | null = null;

  function sendIpc(message: IpcOutMessage) {
    if (window.ipc && window.ipc.postMessage) {
      window.ipc.postMessage(JSON.stringify(message));
    } else {
      console.log('Settings IPC Out:', message);
    }
  }

  function animateViewEntry(view: HTMLElement | null) {
    if (!view) return;

    viewEntryAnimation?.cancel();
    view.style.willChange = 'opacity, transform';
    const animation = view.animate(
      [
        { opacity: 0, transform: 'translate3d(0, 4px, 0)' },
        { opacity: 1, transform: 'translate3d(0, 0, 0)' },
      ],
      { duration: 180, easing: 'ease' }
    );
    viewEntryAnimation = animation;
    void animation.finished
      .catch(() => undefined)
      .finally(() => {
        if (viewEntryAnimation === animation) {
          view.style.removeProperty('will-change');
          viewEntryAnimation = null;
        }
      });
  }

  function switchView(tabName: string) {
    const isThemes = tabName === 'themes';
    const isPrivacy = tabName === 'privacy';
    const isAdblock = tabName === 'adblock';
    const isExtensions = tabName === 'extensions';
    const isGeneral = !isThemes && !isPrivacy && !isAdblock && !isExtensions;

    const viewGeneral = document.getElementById('viewGeneral');
    const viewThemes = document.getElementById('viewThemes');
    const viewPrivacy = document.getElementById('viewPrivacy');
    const viewAdblock = document.getElementById('viewAdblock');
    const viewExtensions = document.getElementById('viewExtensions');

    const tabBtnGeneral = document.getElementById('tabBtnGeneral');
    const tabBtnThemes = document.getElementById('tabBtnThemes');
    const tabBtnPrivacy = document.getElementById('tabBtnPrivacy');
    const tabBtnAdblock = document.getElementById('tabBtnAdblock');
    const tabBtnExtensions = document.getElementById('tabBtnExtensions');

    const headerTitle = document.getElementById('headerTitle');
    const headerSubtitle = document.getElementById('headerSubtitle');
    const headerIconGeneral = document.getElementById('headerIconGeneral');
    const headerIconThemes = document.getElementById('headerIconThemes');
    const headerIconPrivacy = document.getElementById('headerIconPrivacy');
    const headerIconAdblock = document.getElementById('headerIconAdblock');
    const headerIconExtensions = document.getElementById('headerIconExtensions');

    if (viewGeneral) viewGeneral.classList.toggle('active', isGeneral);
    if (viewThemes) viewThemes.classList.toggle('active', isThemes);
    if (viewPrivacy) viewPrivacy.classList.toggle('active', isPrivacy);
    if (viewAdblock) viewAdblock.classList.toggle('active', isAdblock);
    if (viewExtensions) viewExtensions.classList.toggle('active', isExtensions);

    animateViewEntry(
      (isThemes
        ? viewThemes
        : isPrivacy
          ? viewPrivacy
          : isAdblock
            ? viewAdblock
            : isExtensions
              ? viewExtensions
              : viewGeneral) as HTMLElement | null
    );

    if (tabBtnGeneral) tabBtnGeneral.classList.toggle('active', isGeneral);
    if (tabBtnThemes) tabBtnThemes.classList.toggle('active', isThemes);
    if (tabBtnPrivacy) tabBtnPrivacy.classList.toggle('active', isPrivacy);
    if (tabBtnAdblock) tabBtnAdblock.classList.toggle('active', isAdblock);
    if (tabBtnExtensions) tabBtnExtensions.classList.toggle('active', isExtensions);

    if (headerTitle) {
      if (isThemes) headerTitle.textContent = 'Themes & Appearance';
      else if (isPrivacy) headerTitle.textContent = 'Privacy & Security';
      else if (isAdblock) headerTitle.textContent = 'AdBlock & Shields';
      else if (isExtensions) headerTitle.textContent = 'Extensions & Add-ons';
      else headerTitle.textContent = 'Settings';
    }

    if (headerSubtitle) {
      if (isThemes) {
        headerSubtitle.textContent = 'Customize browser themes, accent highlights, and web page contrast';
      } else if (isPrivacy) {
        headerSubtitle.textContent = 'Tracker blocking, privacy signals, fingerprinting controls, and local data';
      } else if (isAdblock) {
        headerSubtitle.textContent = 'Shield controls, video ad auto-skip, popup defense, and custom domain filters';
      } else if (isExtensions) {
        headerSubtitle.textContent = 'Manage, install, and configure Chromium extensions and add-ons';
      } else {
        headerSubtitle.textContent = 'Manage browser preferences, search, and system settings';
      }
    }

    if (headerIconGeneral) headerIconGeneral.style.display = isGeneral ? 'block' : 'none';
    if (headerIconThemes) headerIconThemes.style.display = isThemes ? 'block' : 'none';
    if (headerIconPrivacy) headerIconPrivacy.style.display = isPrivacy ? 'block' : 'none';
    if (headerIconAdblock) headerIconAdblock.style.display = isAdblock ? 'block' : 'none';
    if (headerIconExtensions) headerIconExtensions.style.display = isExtensions ? 'block' : 'none';
  }

  function selectTheme(themeId: string) {
    document.querySelectorAll('.theme-card').forEach((c) => {
      c.classList.toggle('active', c.getAttribute('data-theme') === themeId);
    });
    document.body.className = `theme-${themeId}`;
    sendIpc({ type: 'SetTheme', theme: themeId });
  }

  function selectAccent(color: string) {
    document.querySelectorAll('.accent-swatch').forEach((s) => {
      s.classList.toggle('active', s.getAttribute('data-color') === color);
    });
    document.documentElement.style.setProperty('--accent-primary', color);
    document.documentElement.style.setProperty('--border-focus', color);
    sendIpc({ type: 'SetAccentColor', color: color });
  }

  function changeSearchEngine(engine: string) {
    sendIpc({ type: 'SetSearchEngine', engine: engine });
  }

  function toggleBookmarksBar(show: boolean) {
    sendIpc({ type: 'SetShowBookmarksBar', show: show });
  }

  function setAutoUpdate(enabled: boolean) {
    sendIpc({ type: 'SetAutoUpdate', enabled: enabled });
  }

  function checkForUpdates() {
    renderUpdateState({
      current_version: '0.4.3',
      latest_version: null,
      release_url: null,
      status: 'Checking',
      message: 'Checking for updates...',
    });
    sendIpc({ type: 'CheckForUpdates' });
  }

  function openUpdateDownload() {
    sendIpc({ type: 'OpenUpdateDownload' });
  }

  function renderUpdateState(updateState?: UpdateState) {
    const statusEl = document.getElementById('updateStatusText');
    const versionEl = document.getElementById('updateVersionText');
    const checkBtn = document.getElementById('updateCheckBtn') as HTMLButtonElement | null;
    const openBtn = document.getElementById('updateOpenBtn') as HTMLButtonElement | null;

    if (!updateState) return;

    if (versionEl) {
      const latest = updateState.latest_version ? ` • Latest: ${updateState.latest_version}` : '';
      versionEl.textContent = `Current version: ${updateState.current_version}${latest}`;
    }

    if (statusEl) {
      statusEl.textContent = updateState.message;
      statusEl.classList.toggle('available', updateState.status === 'UpdateAvailable');
      statusEl.classList.toggle('error', updateState.status === 'Error');
    }

    if (checkBtn) {
      checkBtn.disabled = updateState.status === 'Checking';
      checkBtn.textContent = updateState.status === 'Checking' ? 'Checking...' : 'Check Now';
    }

    if (openBtn) {
      openBtn.style.display = updateState.release_url ? 'inline-flex' : 'none';
      openBtn.textContent = updateState.status === 'UpdateAvailable' ? 'Get Update' : 'Release Notes';
    }
  }

  function toggleDarkReader(enabled: boolean) {
    sendIpc({
      type: 'ToggleModule',
      module_id: 'dark_reader',
      enabled: enabled,
    });
  }

  function setPrivacySetting(key: string, enabled: boolean) {
    sendIpc({
      type: 'SetPrivacySetting',
      key: key,
      enabled: enabled,
    });
  }

  function setAdblockSetting(key: string, enabled: boolean) {
    sendIpc({
      type: 'SetAdblockSetting',
      key: key,
      enabled: enabled,
    });
  }

  function clearBrowsingData() {
    sendIpc({
      type: 'ClearBrowsingData',
      cookies: true,
      cache: true,
      local_storage: true,
    });

    const statusEl = document.getElementById('clearDataStatus');
    if (statusEl) {
      statusEl.textContent = 'Browsing data, cache, and cookies cleared successfully!';
      statusEl.style.display = 'block';
      setTimeout(() => {
        statusEl.style.display = 'none';
      }, 3500);
    }
  }

  // Initialize from backend state
  window.initSettings = function (state: SettingsInitState) {
    if (state.active_section) {
      switchView(state.active_section);
    }

    if (state.settings) {
      const theme = state.settings.theme || 'titan-dark';
      document.body.className = `theme-${theme}`;
      document.querySelectorAll('.theme-card').forEach((c) => {
        c.classList.toggle('active', c.getAttribute('data-theme') === theme);
      });

      const accent = state.settings.accent_color || '#4e7cf6';
      document.documentElement.style.setProperty('--accent-primary', accent);
      document.documentElement.style.setProperty('--border-focus', accent);
      document.querySelectorAll('.accent-swatch').forEach((s) => {
        s.classList.toggle('active', s.getAttribute('data-color') === accent);
      });

      if (state.settings.search_engine) {
        const sel = document.getElementById('searchEngineSelect') as HTMLSelectElement | null;
        if (sel) sel.value = state.settings.search_engine;
      }

      const bmToggle = document.getElementById('showBookmarksToggle') as HTMLInputElement | null;
      if (bmToggle) bmToggle.checked = !!state.settings.show_bookmarks_bar;

      const autoUpdateToggle = document.getElementById('autoUpdateToggle') as HTMLInputElement | null;
      if (autoUpdateToggle) autoUpdateToggle.checked = state.settings.auto_update_enabled !== false;

      // Privacy Settings
      const dntToggle = document.getElementById('dntToggle') as HTMLInputElement | null;
      if (dntToggle) dntToggle.checked = state.settings.do_not_track !== false;

      const gpcToggle = document.getElementById('gpcToggle') as HTMLInputElement | null;
      if (gpcToggle) gpcToggle.checked = state.settings.global_privacy_control !== false;

      const stripParamsToggle = document.getElementById('stripParamsToggle') as HTMLInputElement | null;
      if (stripParamsToggle) stripParamsToggle.checked = state.settings.strip_tracking_parameters !== false;

      const webrtcToggle = document.getElementById('webrtcToggle') as HTMLInputElement | null;
      if (webrtcToggle) webrtcToggle.checked = state.settings.block_webrtc_leak !== false;

      const fingerprintToggle = document.getElementById('fingerprintToggle') as HTMLInputElement | null;
      if (fingerprintToggle) fingerprintToggle.checked = state.settings.block_fingerprinting !== false;

      const auditingToggle = document.getElementById('auditingToggle') as HTMLInputElement | null;
      if (auditingToggle) auditingToggle.checked = state.settings.block_hyperlink_auditing !== false;

      // AdBlock Settings
      const adblockToggle = document.getElementById('adblockToggle') as HTMLInputElement | null;
      if (adblockToggle) adblockToggle.checked = state.settings.adblock_enabled !== false;

      const blockVideoAdsToggle = document.getElementById('blockVideoAdsToggle') as HTMLInputElement | null;
      if (blockVideoAdsToggle) blockVideoAdsToggle.checked = state.settings.adblock_block_video_ads !== false;

      const cosmeticFilteringToggle = document.getElementById('cosmeticFilteringToggle') as HTMLInputElement | null;
      if (cosmeticFilteringToggle) cosmeticFilteringToggle.checked = state.settings.adblock_cosmetic_filtering !== false;

      const blockPopupsToggle = document.getElementById('blockPopupsToggle') as HTMLInputElement | null;
      if (blockPopupsToggle) blockPopupsToggle.checked = state.settings.adblock_block_popups !== false;

      const aggressiveAdblockToggle = document.getElementById('aggressiveAdblockToggle') as HTMLInputElement | null;
      if (aggressiveAdblockToggle) aggressiveAdblockToggle.checked = !!state.settings.adblock_aggressive_mode;
    }

    renderUpdateState(state.update_state);

    if (state.modules) {
      const darkMod = state.modules.find((m) => m.id === 'dark_reader');
      const drToggle = document.getElementById('darkReaderToggle') as HTMLInputElement | null;
      if (drToggle && darkMod) {
        drToggle.checked = !!darkMod.enabled;
      }
    }
  };

  // Expose the methods used by the native browser integration.
  (window as unknown as Record<string, unknown>).switchView = switchView;
  (window as unknown as Record<string, unknown>).selectTheme = selectTheme;
  (window as unknown as Record<string, unknown>).selectAccent = selectAccent;
  (window as unknown as Record<string, unknown>).changeSearchEngine = changeSearchEngine;
  (window as unknown as Record<string, unknown>).toggleBookmarksBar = toggleBookmarksBar;
  (window as unknown as Record<string, unknown>).setAutoUpdate = setAutoUpdate;
  (window as unknown as Record<string, unknown>).checkForUpdates = checkForUpdates;
  (window as unknown as Record<string, unknown>).openUpdateDownload = openUpdateDownload;
  (window as unknown as Record<string, unknown>).toggleDarkReader = toggleDarkReader;
  (window as unknown as Record<string, unknown>).setPrivacySetting = setPrivacySetting;
  (window as unknown as Record<string, unknown>).setAdblockSetting = setAdblockSetting;
  (window as unknown as Record<string, unknown>).clearBrowsingData = clearBrowsingData;

  // DOM Event Listeners
  document.addEventListener('DOMContentLoaded', () => {
    const tabBtnGeneral = document.getElementById('tabBtnGeneral');
    if (tabBtnGeneral) {
      tabBtnGeneral.addEventListener('click', () => switchView('general'));
    }

    const tabBtnThemes = document.getElementById('tabBtnThemes');
    if (tabBtnThemes) {
      tabBtnThemes.addEventListener('click', () => switchView('themes'));
    }

    const tabBtnPrivacy = document.getElementById('tabBtnPrivacy');
    if (tabBtnPrivacy) {
      tabBtnPrivacy.addEventListener('click', () => switchView('privacy'));
    }

    const tabBtnAdblock = document.getElementById('tabBtnAdblock');
    if (tabBtnAdblock) {
      tabBtnAdblock.addEventListener('click', () => switchView('adblock'));
    }

    document.querySelectorAll('.theme-card').forEach((card) => {
      card.addEventListener('click', () => {
        const theme = card.getAttribute('data-theme');
        if (theme) selectTheme(theme);
      });
    });

    document.querySelectorAll('.accent-swatch').forEach((swatch) => {
      swatch.addEventListener('click', () => {
        const color = swatch.getAttribute('data-color');
        if (color) selectAccent(color);
      });
    });

    const searchSelect = document.getElementById('searchEngineSelect') as HTMLSelectElement | null;
    if (searchSelect) {
      searchSelect.addEventListener('change', (e) => {
        const target = e.target as HTMLSelectElement;
        changeSearchEngine(target.value);
      });
    }

    const bmToggle = document.getElementById('showBookmarksToggle') as HTMLInputElement | null;
    if (bmToggle) {
      bmToggle.addEventListener('change', (e) => {
        const target = e.target as HTMLInputElement;
        toggleBookmarksBar(target.checked);
      });
    }

    const autoUpdateToggle = document.getElementById('autoUpdateToggle') as HTMLInputElement | null;
    autoUpdateToggle?.addEventListener('change', () => setAutoUpdate(autoUpdateToggle.checked));

    document.getElementById('updateCheckBtn')?.addEventListener('click', checkForUpdates);
    document.getElementById('updateOpenBtn')?.addEventListener('click', openUpdateDownload);
    document.getElementById('defaultBrowserBtn')?.addEventListener('click', () => {
      sendIpc({ type: 'OpenDefaultBrowserSettings' });
    });

    const drToggle = document.getElementById('darkReaderToggle') as HTMLInputElement | null;
    if (drToggle) {
      drToggle.addEventListener('change', (e) => {
        const target = e.target as HTMLInputElement;
        toggleDarkReader(target.checked);
      });
    }

    // Privacy Toggles
    const privacyKeys = [
      { id: 'dntToggle', key: 'do_not_track' },
      { id: 'gpcToggle', key: 'global_privacy_control' },
      { id: 'stripParamsToggle', key: 'strip_tracking_parameters' },
      { id: 'webrtcToggle', key: 'block_webrtc_leak' },
      { id: 'fingerprintToggle', key: 'block_fingerprinting' },
      { id: 'auditingToggle', key: 'block_hyperlink_auditing' },
    ];

    privacyKeys.forEach(({ id, key }) => {
      const toggle = document.getElementById(id) as HTMLInputElement | null;
      if (toggle) {
        toggle.addEventListener('change', (e) => {
          const target = e.target as HTMLInputElement;
          setPrivacySetting(key, target.checked);
        });
      }
    });

    // AdBlock Toggles
    const adblockKeys = [
      { id: 'adblockToggle', key: 'adblock_enabled' },
      { id: 'blockVideoAdsToggle', key: 'adblock_block_video_ads' },
      { id: 'cosmeticFilteringToggle', key: 'adblock_cosmetic_filtering' },
      { id: 'blockPopupsToggle', key: 'adblock_block_popups' },
      { id: 'aggressiveAdblockToggle', key: 'adblock_aggressive_mode' },
    ];

    adblockKeys.forEach(({ id, key }) => {
      const toggle = document.getElementById(id) as HTMLInputElement | null;
      if (toggle) {
        toggle.addEventListener('change', (e) => {
          const target = e.target as HTMLInputElement;
          setAdblockSetting(key, target.checked);
        });
      }
    });

    const clearBtn = document.getElementById('clearBrowsingDataBtn');
    if (clearBtn) {
      clearBtn.addEventListener('click', clearBrowsingData);
    }

    const bindEnter = (inputId: string, action: () => void) => {
      document.getElementById(inputId)?.addEventListener('keydown', (event) => {
        if ((event as KeyboardEvent).key === 'Enter') action();
      });
    };

    bindEnter('newBlockedDomainInput', addBlockedDomain);
    bindEnter('newWhitelistDomainInput', addWhitelistedDomain);
    bindEnter('newCustomRuleInput', addCustomRule);
    bindEnter('newAdblockWhitelistDomainInput', addAdblockWhitelist);

    document.getElementById('addBlockedDomainBtn')?.addEventListener('click', addBlockedDomain);
    document.getElementById('addWhitelistedDomainBtn')?.addEventListener('click', addWhitelistedDomain);
    document.getElementById('addCustomRuleBtn')?.addEventListener('click', addCustomRule);
    document.getElementById('addAdblockWhitelistBtn')?.addEventListener('click', addAdblockWhitelist);
    document.getElementById('resetPrivacyRulesBtn')?.addEventListener('click', resetPrivacyRules);
    document.getElementById('clearAdblockLogsBtn')?.addEventListener('click', clearAdblockLogs);

    document.getElementById('blockedRulesSearch')?.addEventListener('input', (event) => {
      filterBlockedRules((event.target as HTMLInputElement).value);
    });

    document.addEventListener('click', (event) => {
      const target = (event.target as Element | null)?.closest<HTMLElement>('[data-settings-action]');
      if (!target) return;

      const value = decodeURIComponent(target.dataset.value || '');
      switch (target.dataset.settingsAction) {
        case 'remove-blocked-domain':
          removeBlockedDomain(value);
          break;
        case 'remove-whitelisted-domain':
          removeWhitelistedDomain(value);
          break;
        case 'remove-custom-rule':
          removeCustomRule(target.dataset.value || '');
          break;
        case 'remove-adblock-whitelist':
          removeAdblockWhitelist(value);
          break;
      }
    });

    document.addEventListener('change', (event) => {
      const target = (event.target as Element | null)?.closest<HTMLInputElement>(
        'input[data-settings-action="toggle-filter-list"]'
      );
      if (target?.dataset.value) {
        toggleFilterList(decodeURIComponent(target.dataset.value), target.checked);
      }
    });
  });

  // Privacy Blocklist & Whitelist Management
  let currentBlockedDomains: string[] = [];
  let mandatoryBlockedDomains: string[] = [];
  let currentWhitelistedDomains: string[] = [];
  let currentBlockedLogs: BlockedRequestLog[] = [];
  let currentRuleFilter: string = '';


  function cleanDomainInput(val: string): string {
    const candidate = val.trim();
    if (!candidate) return '';
    try {
      const parsed = new URL(/^https?:\/\//i.test(candidate) ? candidate : `https://${candidate}`);
      const host = parsed.hostname.toLowerCase().replace(/\.$/, '');
      return /^[a-z0-9.-]+$/.test(host) ? host : '';
    } catch (_) {
      return '';
    }
  }

  function escapeHtml(value: unknown): string {
    return String(value)
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;')
      .replace(/"/g, '&quot;')
      .replace(/'/g, '&#39;');
  }

  function getDomainCategory(d: string): { label: string; color: string } {
    if (d.includes('aria') || d.includes('telemetry') || d.includes('watson')) {
      return { label: 'Telemetry', color: '#f43f5e' };
    }
    if (d.includes('analytics') || d.includes('clarity') || d.includes('hotjar') || d.includes('datadog')) {
      return { label: 'Analytics', color: '#f59e0b' };
    }
    if (d.includes('sentry') || d.includes('bugsnag') || d.includes('crashlytics') || d.includes('loggly')) {
      return { label: 'Crash Logger', color: '#8b5cf6' };
    }
    if (d.includes('doubleclick') || d.includes('criteo') || d.includes('outbrain') || d.includes('taboola')) {
      return { label: 'Ad Tracker', color: '#06b6d4' };
    }
    return { label: 'Custom Rule', color: '#10b981' };
  }

  function renderBlockedDomainsList() {
    const listEl = document.getElementById('blockedDomainsList');
    const countBadge = document.getElementById('blockedRulesCount');
    if (!listEl) return;

    const filtered = currentBlockedDomains.filter((d) =>
      d.toLowerCase().includes(currentRuleFilter.toLowerCase())
    );

    if (countBadge) {
      countBadge.textContent = `${currentBlockedDomains.length} rules active`;
    }

    if (filtered.length === 0) {
      listEl.innerHTML = `
        <div class="rules-empty-state">
          ${currentRuleFilter ? 'No matching domains found.' : 'No blocked domains configured.'}
        </div>
      `;
      return;
    }

    listEl.innerHTML = filtered
      .map((domain) => {
        const cat = getDomainCategory(domain);
        const isMandatory = mandatoryBlockedDomains.includes(domain);
        return `
          <div class="rule-item">
            <div class="rule-item-left">
              <span class="rule-cat-badge" style="background: ${cat.color}22; color: ${cat.color}; border: 1px solid ${cat.color}44;">
                ${cat.label}
              </span>
              <span class="rule-domain-text">${escapeHtml(domain)}</span>
            </div>
            ${isMandatory ? '<span class="rules-badge">Built-in</span>' : `
              <button class="rule-delete-btn" data-settings-action="remove-blocked-domain" data-value="${encodeURIComponent(domain)}" title="Remove rule">
                <svg viewBox="0 0 24 24" width="14" height="14" stroke="currentColor" stroke-width="2" fill="none">
                  <line x1="18" y1="6" x2="6" y2="18"></line>
                  <line x1="6" y1="6" x2="18" y2="18"></line>
                </svg>
              </button>
            `}
          </div>
        `;
      })
      .join('');
  }

  function renderWhitelistedDomainsList() {
    const listEl = document.getElementById('whitelistDomainsList');
    const countBadge = document.getElementById('whitelistRulesCount');
    if (!listEl) return;

    if (countBadge) {
      countBadge.textContent = `${currentWhitelistedDomains.length} exceptions`;
    }

    if (currentWhitelistedDomains.length === 0) {
      listEl.innerHTML = `
        <div class="rules-empty-state">
          No custom exceptions are configured. Built-in telemetry firewall rules always apply.
        </div>
      `;
      return;
    }

    listEl.innerHTML = currentWhitelistedDomains
      .map((domain) => {
        return `
          <div class="rule-item">
            <div class="rule-item-left">
              <span class="rule-cat-badge" style="background: rgba(16, 185, 129, 0.15); color: #10b981; border: 1px solid rgba(16, 185, 129, 0.3);">
                Allowed
              </span>
              <span class="rule-domain-text">${escapeHtml(domain)}</span>
            </div>
            <button class="rule-delete-btn" data-settings-action="remove-whitelisted-domain" data-value="${encodeURIComponent(domain)}" title="Remove whitelist exception">
              <svg viewBox="0 0 24 24" width="14" height="14" stroke="currentColor" stroke-width="2" fill="none">
                <line x1="18" y1="6" x2="6" y2="18"></line>
                <line x1="6" y1="6" x2="18" y2="18"></line>
              </svg>
            </button>
          </div>
        `;
      })
      .join('');
  }

  function renderBlockedActivityLogs() {
    const logListEl = document.getElementById('blockedActivityLogList');
    const countBadge = document.getElementById('blockedActivityCountBadge');
    if (!logListEl) return;

    if (countBadge) {
      countBadge.textContent = `${currentBlockedLogs.length} stopped`;
    }

    if (currentBlockedLogs.length === 0) {
      logListEl.innerHTML = `
        <div class="rules-empty-state">
          No blocked tracker requests are recorded for this session.
        </div>
      `;
      return;
    }

    logListEl.innerHTML = currentBlockedLogs
      .slice(0, 30)
      .map((log) => {
        const safeType = escapeHtml(log.req_type.toUpperCase());
        const safeDomain = escapeHtml(log.domain);
        const safeTimestamp = escapeHtml(log.timestamp);
        const safeUrl = escapeHtml(log.url);
        return `
          <div class="log-item">
            <div class="log-item-header">
              <span class="log-badge-type">${safeType}</span>
              <span class="log-domain-match">${safeDomain}</span>
              <span class="log-timestamp">${safeTimestamp}</span>
            </div>
            <div class="log-url-text" title="${safeUrl}">${safeUrl}</div>
          </div>
        `;
      })
      .join('');
  }

  // AdBlock Engine Lists, Custom Rules, Stats, Whitelist & Logs

  let currentFilterLists: FilterListConfig[] = [];
  let currentCustomRules: string[] = [];
  let currentAdblockStats: AdblockStats | null = null;
  let currentAdblockWhitelistedDomains: string[] = [];
  let currentAdblockLogs: BlockedRequestLog[] = [];

  function formatBytes(bytes: number): string {
    if (!bytes || bytes === 0) return '0 KB';
    const k = 1024;
    const sizes = ['Bytes', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
  }

  function renderAdblockStats() {
    const statAds = document.getElementById('statAdsBlocked');
    const statBw = document.getElementById('statBandwidthSaved');
    const statCosm = document.getElementById('statCosmeticHidden');
    const statRules = document.getElementById('statTotalRules');

    const totalBlocked = currentAdblockStats ? currentAdblockStats.blocked_requests_count : currentAdblockLogs.length;
    const bwSaved = currentAdblockStats && currentAdblockStats.estimated_bandwidth_saved_bytes > 0
      ? currentAdblockStats.estimated_bandwidth_saved_bytes
      : currentAdblockLogs.length * 45 * 1024;
    const cosmeticCount = currentAdblockStats ? currentAdblockStats.cosmetic_elements_hidden_count : 0;
    const totalRulesCount = currentAdblockStats && currentAdblockStats.total_rules > 0
      ? currentAdblockStats.total_rules
      : currentFilterLists.reduce((acc, l) => acc + (l.enabled ? l.count : 0), 0);

    if (statAds) statAds.textContent = totalBlocked.toLocaleString();
    if (statBw) statBw.textContent = formatBytes(bwSaved);
    if (statCosm) statCosm.textContent = cosmeticCount.toLocaleString();
    if (statRules) statRules.textContent = totalRulesCount > 0 ? totalRulesCount.toLocaleString() : '120,000+';
  }

  function renderFilterLists() {
    const container = document.getElementById('filterListsContainer');
    if (!container) return;

    if (currentFilterLists.length === 0) {
      container.innerHTML = `<div class="rules-empty-state">Loading filter subscriptions...</div>`;
      return;
    }

    container.innerHTML = currentFilterLists
      .map((list) => {
        const safeName = escapeHtml(list.name);
        const safeDescription = escapeHtml(list.description);
        return `
          <div class="filter-list-item">
            <div class="filter-list-info">
              <div class="filter-list-name-row">
                <span class="filter-list-name">${safeName}</span>
                <span class="filter-list-count">${list.count.toLocaleString()} rules</span>
              </div>
              <div class="filter-list-desc">${safeDescription}</div>
            </div>
            <label class="switch" style="flex-shrink: 0;">
              <input type="checkbox" ${list.enabled ? 'checked' : ''} data-settings-action="toggle-filter-list" data-value="${encodeURIComponent(list.id)}" />
              <span class="slider"></span>
            </label>
          </div>
        `;
      })
      .join('');
  }

  function renderCustomRules() {
    const listEl = document.getElementById('customRulesList');
    const countBadge = document.getElementById('customRulesCount');
    if (!listEl) return;

    if (countBadge) {
      countBadge.textContent = `${currentCustomRules.length} custom rules`;
    }

    if (currentCustomRules.length === 0) {
      listEl.innerHTML = `
        <div class="rules-empty-state">
          No custom rules added yet. Enter standard Adblock Plus or uBlock Origin filter rules above.
        </div>
      `;
      return;
    }

    listEl.innerHTML = currentCustomRules
      .map((rule) => {
        const isCosmetic = rule.includes('##') || rule.includes('#@#') || rule.includes('#?#');
        const badgeLabel = isCosmetic ? 'Cosmetic CSS' : 'Network Rule';
        const badgeColor = isCosmetic ? '#f59e0b' : '#3b82f6';
        const safeRule = encodeURIComponent(rule);
        const escapedRule = escapeHtml(rule);
        return `
          <div class="rule-item">
            <div class="rule-item-left">
              <span class="rule-cat-badge" style="background: ${badgeColor}22; color: ${badgeColor}; border: 1px solid ${badgeColor}44;">
                ${badgeLabel}
              </span>
              <span class="rule-domain-text" title="${escapedRule}">${escapedRule}</span>
            </div>
            <button class="rule-delete-btn" data-settings-action="remove-custom-rule" data-value="${safeRule}" title="Remove rule">
              <svg viewBox="0 0 24 24" width="14" height="14" stroke="currentColor" stroke-width="2" fill="none">
                <line x1="18" y1="6" x2="6" y2="18"></line>
                <line x1="6" y1="6" x2="18" y2="18"></line>
              </svg>
            </button>
          </div>
        `;
      })
      .join('');
  }

  function renderAdblockWhitelistedDomainsList() {
    const listEl = document.getElementById('adblockWhitelistDomainsList');
    const countBadge = document.getElementById('adblockWhitelistRulesCount');
    if (!listEl) return;

    if (countBadge) {
      countBadge.textContent = `${currentAdblockWhitelistedDomains.length} websites`;
    }

    if (currentAdblockWhitelistedDomains.length === 0) {
      listEl.innerHTML = `
        <div class="rules-empty-state">
          No websites whitelisted. AdBlock is active across all visited websites.
        </div>
      `;
      return;
    }

    listEl.innerHTML = currentAdblockWhitelistedDomains
      .map((domain) => {
        return `
          <div class="rule-item">
            <div class="rule-item-left">
              <span class="rule-cat-badge" style="background: rgba(16, 185, 129, 0.15); color: #10b981; border: 1px solid rgba(16, 185, 129, 0.3);">
                Allowed
              </span>
              <span class="rule-domain-text">${escapeHtml(domain)}</span>
            </div>
            <button class="rule-delete-btn" data-settings-action="remove-adblock-whitelist" data-value="${encodeURIComponent(domain)}" title="Remove whitelist exception">
              <svg viewBox="0 0 24 24" width="14" height="14" stroke="currentColor" stroke-width="2" fill="none">
                <line x1="18" y1="6" x2="6" y2="18"></line>
                <line x1="6" y1="6" x2="18" y2="18"></line>
              </svg>
            </button>
          </div>
        `;
      })
      .join('');
  }

  function renderAdblockActivityLogs() {
    const logListEl = document.getElementById('adblockActivityLogList');
    const countBadge = document.getElementById('adblockActivityCountBadge');
    if (!logListEl) return;

    if (countBadge) {
      countBadge.textContent = `${currentAdblockLogs.length} ads blocked`;
    }

    if (currentAdblockLogs.length === 0) {
      logListEl.innerHTML = `
        <div class="rules-empty-state">
          No ads intercepted in this session.
        </div>
      `;
      return;
    }

    logListEl.innerHTML = currentAdblockLogs
      .slice(0, 40)
      .map((log) => {
        const safeType = escapeHtml(log.req_type.toUpperCase());
        const safeDomain = escapeHtml(log.domain);
        const safeTimestamp = escapeHtml(log.timestamp);
        const safeUrl = escapeHtml(log.url);
        return `
          <div class="log-item">
            <div class="log-item-header">
              <span class="log-badge-type" style="background: rgba(78, 124, 246, 0.2); color: #60a5fa; border-color: rgba(78, 124, 246, 0.35);">
                ${safeType}
              </span>
              <span class="log-domain-match">${safeDomain}</span>
              <span class="log-timestamp">${safeTimestamp}</span>
            </div>
            <div class="log-url-text" title="${safeUrl}">${safeUrl}</div>
          </div>
        `;
      })
      .join('');
  }

  // Privacy Rule Actions
  function addBlockedDomain() {
    const input = document.getElementById('newBlockedDomainInput') as HTMLInputElement | null;
    if (!input) return;
    const domain = cleanDomainInput(input.value);
    if (!domain) return;
    sendIpc({ type: 'AddBlockedDomain', domain });
    input.value = '';
    if (!currentBlockedDomains.includes(domain)) {
      currentBlockedDomains.unshift(domain);
      renderBlockedDomainsList();
    }
  }

  function removeBlockedDomain(domain: string) {
    if (mandatoryBlockedDomains.includes(domain)) return;
    sendIpc({ type: 'RemoveBlockedDomain', domain });
    currentBlockedDomains = currentBlockedDomains.filter((d) => d !== domain);
    renderBlockedDomainsList();
  }

  function addWhitelistedDomain() {
    const input = document.getElementById('newWhitelistDomainInput') as HTMLInputElement | null;
    if (!input) return;
    const domain = cleanDomainInput(input.value);
    if (!domain) return;
    sendIpc({ type: 'AddWhitelistedDomain', domain });
    input.value = '';
    if (!currentWhitelistedDomains.includes(domain)) {
      currentWhitelistedDomains.unshift(domain);
      renderWhitelistedDomainsList();
    }
  }

  function removeWhitelistedDomain(domain: string) {
    sendIpc({ type: 'RemoveWhitelistedDomain', domain });
    currentWhitelistedDomains = currentWhitelistedDomains.filter((d) => d !== domain);
    renderWhitelistedDomainsList();
  }

  function resetPrivacyRules() {
    if (confirm('Reset all privacy blocklist rules and whitelist exceptions to Titan defaults?')) {
      sendIpc({ type: 'ResetPrivacyRules' });
    }
  }

  function filterBlockedRules(query: string) {
    currentRuleFilter = query;
    renderBlockedDomainsList();
  }

  // AdBlock / uBlock Actions
  function toggleFilterList(listId: string, enabled: boolean) {
    sendIpc({ type: 'ToggleFilterList', list_id: listId, enabled });
    const list = currentFilterLists.find((l) => l.id === listId);
    if (list) {
      list.enabled = enabled;
      renderAdblockStats();
    }
  }

  function addCustomRule() {
    const input = document.getElementById('newCustomRuleInput') as HTMLInputElement | null;
    if (!input) return;
    const rule = input.value.trim();
    if (!rule) return;
    sendIpc({ type: 'AddCustomFilterRule', rule });
    input.value = '';
    if (!currentCustomRules.includes(rule)) {
      currentCustomRules.unshift(rule);
      renderCustomRules();
      renderAdblockStats();
    }
  }

  function removeCustomRule(encodedRule: string) {
    const rule = decodeURIComponent(encodedRule);
    sendIpc({ type: 'RemoveCustomFilterRule', rule });
    currentCustomRules = currentCustomRules.filter((r) => r !== rule);
    renderCustomRules();
    renderAdblockStats();
  }

  function addAdblockWhitelist() {
    const input = document.getElementById('newAdblockWhitelistDomainInput') as HTMLInputElement | null;
    if (!input) return;
    const domain = cleanDomainInput(input.value);
    if (!domain) return;
    sendIpc({ type: 'AddAdblockWhitelist', domain });
    input.value = '';
    if (!currentAdblockWhitelistedDomains.includes(domain)) {
      currentAdblockWhitelistedDomains.unshift(domain);
      renderAdblockWhitelistedDomainsList();
    }
  }

  function removeAdblockWhitelist(domain: string) {
    sendIpc({ type: 'RemoveAdblockWhitelist', domain });
    currentAdblockWhitelistedDomains = currentAdblockWhitelistedDomains.filter((d) => d !== domain);
    renderAdblockWhitelistedDomainsList();
  }

  function clearAdblockLogs() {
    currentAdblockLogs = [];
    renderAdblockActivityLogs();
    renderAdblockStats();
    sendIpc({ type: 'ClearAdblockLogs' });
  }

  // ==========================================================================
  // Extensions Management Logic
  // ==========================================================================
  let currentExtensions: ExtensionInfo[] = [];

  function renderExtensionsList(searchFilter = '') {
    const grid = document.getElementById('extensionsGrid');
    const countBadge = document.getElementById('extensionsCountBadge');
    if (!grid) return;

    const filter = searchFilter.toLowerCase().trim();
    const filtered = currentExtensions.filter(
      (ext) =>
        !filter ||
        ext.name.toLowerCase().includes(filter) ||
        ext.id.toLowerCase().includes(filter) ||
        ext.description.toLowerCase().includes(filter)
    );

    if (countBadge) {
      countBadge.textContent = `${currentExtensions.length} extension${currentExtensions.length === 1 ? '' : 's'}`;
    }

    grid.innerHTML = '';

    if (filtered.length === 0) {
      const empty = document.createElement('div');
      empty.style.gridColumn = '1 / -1';
      empty.style.textAlign = 'center';
      empty.style.padding = '36px 16px';
      empty.style.color = 'var(--text-muted)';
      empty.style.fontSize = '13px';
      empty.textContent =
        currentExtensions.length === 0
          ? 'No extensions installed yet. Use the box above or visit the Chrome Web Store / Edge Add-ons to install.'
          : 'No extensions match your search.';
      grid.appendChild(empty);
      return;
    }

    filtered.forEach((ext) => {
      const card = document.createElement('div');
      card.className = 'ext-card';

      const header = document.createElement('div');
      header.className = 'ext-card-header';

      const icon = document.createElement('img');
      icon.className = 'ext-card-icon';
      icon.src =
        ext.icon ||
        'data:image/svg+xml;utf8,<svg viewBox="0 0 24 24" fill="none" stroke="%234e7cf6" stroke-width="2" xmlns="http://www.w3.org/2000/svg"><path d="M20.5 11H19V7c0-1.1-.9-2-2-2h-4V3.5a2.5 2.5 0 0 0-5 0V5H4c-1.1 0-1.99.9-1.99 2v3.8H3.5c1.49 0 2.7 1.21 2.7 2.7s-1.21 2.7-2.7 2.7H2V20c0 1.1.9 2 2 2h3.8v-1.5c0-1.49 1.21-2.7 2.7-2.7 1.49 0 2.7 1.21 2.7 2.7V22H17c1.1 0 2-.9 2-2v-4h1.5a2.5 2.5 0 0 0 0-5z"/></svg>';

      const titleGroup = document.createElement('div');
      titleGroup.className = 'ext-card-title-group';

      const name = document.createElement('div');
      name.className = 'ext-card-name';
      name.textContent = ext.name;
      name.title = ext.name;

      const meta = document.createElement('div');
      meta.className = 'ext-card-meta';

      const version = document.createElement('span');
      version.textContent = `v${ext.version}`;

      const badge = document.createElement('span');
      const srcLower = (ext.source || '').toLowerCase();
      badge.className = `ext-badge ${srcLower === 'chrome' ? 'chrome' : srcLower === 'edge' ? 'edge' : 'unpacked'}`;
      badge.textContent = srcLower === 'chrome' ? 'Chrome Store' : srcLower === 'edge' ? 'Edge Add-ons' : 'Unpacked';

      meta.appendChild(version);
      meta.appendChild(badge);

      titleGroup.appendChild(name);
      titleGroup.appendChild(meta);

      header.appendChild(icon);
      header.appendChild(titleGroup);

      const desc = document.createElement('div');
      desc.className = 'ext-card-desc';
      desc.textContent = ext.description || 'No description provided.';
      desc.title = ext.description || '';

      const actions = document.createElement('div');
      actions.className = 'ext-card-actions';

      const btnGroup = document.createElement('div');
      btnGroup.className = 'ext-btn-group';
      if (ext.popup_page) {
        const popupBtn = document.createElement('button');
        popupBtn.className = 'ext-action-btn';
        popupBtn.innerHTML = `<svg viewBox="0 0 24 24" width="12" height="12" stroke="currentColor" stroke-width="2" fill="none"><rect x="3" y="3" width="18" height="18" rx="2" ry="2"></rect><line x1="9" y1="3" x2="9" y2="21"></line></svg> Popup`;
        popupBtn.title = 'Open extension popup interface';
        popupBtn.addEventListener('click', () => {
          sendIpc({ type: 'OpenExtensionPopup', id: ext.id });
        });
        btnGroup.appendChild(popupBtn);
      }

      if (ext.options_page) {
        const optionsBtn = document.createElement('button');
        optionsBtn.className = 'ext-action-btn';
        optionsBtn.innerHTML = `<svg viewBox="0 0 24 24" width="12" height="12" stroke="currentColor" stroke-width="2" fill="none"><circle cx="12" cy="12" r="3"></circle><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"></path></svg> Options`;
        optionsBtn.addEventListener('click', () => {
          sendIpc({ type: 'OpenExtensionOptions', id: ext.id });
        });
        btnGroup.appendChild(optionsBtn);
      }

      const removeBtn = document.createElement('button');
      removeBtn.className = 'ext-action-btn danger';
      removeBtn.innerHTML = `<svg viewBox="0 0 24 24" width="12" height="12" stroke="currentColor" stroke-width="2" fill="none"><polyline points="3 6 5 6 21 6"></polyline><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path></svg> Remove`;
      removeBtn.addEventListener('click', () => {
        if (confirm(`Are you sure you want to remove "${ext.name}"?`)) {
          sendIpc({ type: 'UninstallExtension', id: ext.id });
          currentExtensions = currentExtensions.filter((e) => e.id !== ext.id);
          renderExtensionsList(searchFilter);
        }
      });
      btnGroup.appendChild(removeBtn);

      const toggleLabel = document.createElement('label');
      toggleLabel.className = 'switch';
      const toggleInput = document.createElement('input');
      toggleInput.type = 'checkbox';
      toggleInput.checked = ext.enabled;
      toggleInput.addEventListener('change', () => {
        ext.enabled = toggleInput.checked;
        sendIpc({ type: 'ToggleExtension', id: ext.id, enabled: ext.enabled });
      });
      const slider = document.createElement('span');
      slider.className = 'slider';

      toggleLabel.appendChild(toggleInput);
      toggleLabel.appendChild(slider);

      actions.appendChild(btnGroup);
      actions.appendChild(toggleLabel);

      card.appendChild(header);
      card.appendChild(desc);
      card.appendChild(actions);

      grid.appendChild(card);
    });
  }

  function installExtensionFromInput() {
    const input = document.getElementById('extensionInstallInput') as HTMLInputElement | null;
    const btn = document.getElementById('extensionInstallBtn') as HTMLButtonElement | null;
    if (!input) return;

    const val = input.value.trim();
    if (!val) return;

    if (btn) {
      btn.disabled = true;
      btn.textContent = 'Installing...';
    }

    sendIpc({ type: 'InstallExtension', id_or_url: val });
    input.value = '';

    setTimeout(() => {
      if (btn) {
        btn.disabled = false;
        btn.innerHTML = `<svg viewBox="0 0 24 24" width="14" height="14" stroke="currentColor" stroke-width="2" fill="none"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path><polyline points="7 10 12 15 17 10"></polyline><line x1="12" y1="15" x2="12" y2="3"></line></svg> Install Extension`;
      }
    }, 2500);
  }

  function loadUnpackedExtensionPrompt() {
    const folderPath = prompt('Enter the absolute path to the unpacked extension directory:\n(Contains manifest.json)');
    if (folderPath && folderPath.trim()) {
      sendIpc({ type: 'LoadUnpackedExtension', path: folderPath.trim() });
    }
  }

  // Update window.initSettings to populate blocklists, filter subscriptions, custom rules, stats & logs, extensions
  const origInitSettings = window.initSettings;
  window.initSettings = function (state: SettingsInitState) {
    if (origInitSettings) origInitSettings(state);

    if (state.settings) {
      if (Array.isArray(state.settings.blocked_domains)) {
        currentBlockedDomains = [...state.settings.blocked_domains];
        renderBlockedDomainsList();
      }
      if (Array.isArray(state.settings.whitelisted_domains)) {
        currentWhitelistedDomains = [...state.settings.whitelisted_domains];
        renderWhitelistedDomainsList();
      }
      if (Array.isArray(state.settings.adblock_whitelisted_domains)) {
        currentAdblockWhitelistedDomains = [...state.settings.adblock_whitelisted_domains];
        renderAdblockWhitelistedDomainsList();
      }
      if (Array.isArray(state.settings.adblock_custom_rules)) {
        currentCustomRules = [...state.settings.adblock_custom_rules];
        renderCustomRules();
      }
    }

    if (Array.isArray(state.adblock_filter_lists)) {
      currentFilterLists = [...state.adblock_filter_lists];
      renderFilterLists();
    }

    if (Array.isArray(state.mandatory_blocked_domains)) {
      mandatoryBlockedDomains = [...state.mandatory_blocked_domains];
      renderBlockedDomainsList();
    }

    if (Array.isArray(state.adblock_custom_rules)) {
      currentCustomRules = [...state.adblock_custom_rules];
      renderCustomRules();
    }

    if (state.adblock_stats) {
      currentAdblockStats = { ...state.adblock_stats };
      renderAdblockStats();
    }

    if (Array.isArray(state.blocked_logs)) {
      currentBlockedLogs = [...state.blocked_logs];
      renderBlockedActivityLogs();
    }

    if (Array.isArray(state.adblock_logs)) {
      currentAdblockLogs = [...state.adblock_logs];
      renderAdblockActivityLogs();
      renderAdblockStats();
    }

    if (Array.isArray(state.extensions)) {
      currentExtensions = [...state.extensions];
      renderExtensionsList();
    }
  };

  // Wire event listeners on load
  document.addEventListener('DOMContentLoaded', () => {
    const tabBtnExt = document.getElementById('tabBtnExtensions');
    if (tabBtnExt) {
      tabBtnExt.addEventListener('click', () => switchView('extensions'));
    }

    const installBtn = document.getElementById('extensionInstallBtn');
    if (installBtn) {
      installBtn.addEventListener('click', installExtensionFromInput);
    }

    const installInput = document.getElementById('extensionInstallInput');
    if (installInput) {
      installInput.addEventListener('keydown', (e) => {
        if (e.key === 'Enter') {
          e.preventDefault();
          installExtensionFromInput();
        }
      });
    }

    const searchInput = document.getElementById('extensionSearchInput') as HTMLInputElement | null;
    if (searchInput) {
      searchInput.addEventListener('input', () => {
        renderExtensionsList(searchInput.value);
      });
    }

    const chromeBtn = document.getElementById('openChromeStoreBtn');
    if (chromeBtn) {
      chromeBtn.addEventListener('click', () => {
        sendIpc({ type: 'NewTab', url: 'https://chromewebstore.google.com' });
      });
    }

    const edgeBtn = document.getElementById('openEdgeStoreBtn');
    if (edgeBtn) {
      edgeBtn.addEventListener('click', () => {
        sendIpc({ type: 'NewTab', url: 'https://microsoftedge.microsoft.com/addons' });
      });
    }

    const loadUnpackedBtn = document.getElementById('loadUnpackedExtensionBtn');
    if (loadUnpackedBtn) {
      loadUnpackedBtn.addEventListener('click', loadUnpackedExtensionPrompt);
    }
  });

  function initializeFromEmbeddedState() {
    const stateElement = document.getElementById('titan-settings-state');
    if (!stateElement?.textContent || !window.initSettings) return;

    try {
      window.initSettings(JSON.parse(stateElement.textContent) as SettingsInitState);
    } catch (error) {
      console.error('Failed to read the initial settings state:', error);
    }
  }

  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', initializeFromEmbeddedState);
  } else {
    initializeFromEmbeddedState();
  }

  (window as unknown as Record<string, unknown>).addBlockedDomain = addBlockedDomain;
  (window as unknown as Record<string, unknown>).removeBlockedDomain = removeBlockedDomain;
  (window as unknown as Record<string, unknown>).addWhitelistedDomain = addWhitelistedDomain;
  (window as unknown as Record<string, unknown>).removeWhitelistedDomain = removeWhitelistedDomain;
  (window as unknown as Record<string, unknown>).resetPrivacyRules = resetPrivacyRules;
  (window as unknown as Record<string, unknown>).filterBlockedRules = filterBlockedRules;

  (window as unknown as Record<string, unknown>).toggleFilterList = toggleFilterList;
  (window as unknown as Record<string, unknown>).addCustomRule = addCustomRule;
  (window as unknown as Record<string, unknown>).removeCustomRule = removeCustomRule;
  (window as unknown as Record<string, unknown>).addAdblockWhitelist = addAdblockWhitelist;
  (window as unknown as Record<string, unknown>).removeAdblockWhitelist = removeAdblockWhitelist;
  (window as unknown as Record<string, unknown>).clearAdblockLogs = clearAdblockLogs;
  (window as unknown as Record<string, unknown>).installExtensionFromInput = installExtensionFromInput;
  (window as unknown as Record<string, unknown>).loadUnpackedExtensionPrompt = loadUnpackedExtensionPrompt;
})();
