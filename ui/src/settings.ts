// Titan Browser - Settings Controller (TypeScript)

(function () {
  function sendIpc(message: IpcOutMessage) {
    if (window.ipc && window.ipc.postMessage) {
      window.ipc.postMessage(JSON.stringify(message));
    } else {
      console.log('Settings IPC Out:', message);
    }
  }

  function switchView(tabName: string) {
    const isThemes = tabName === 'themes';
    const isPrivacy = tabName === 'privacy';
    const isGeneral = !isThemes && !isPrivacy;

    const viewGeneral = document.getElementById('viewGeneral');
    const viewThemes = document.getElementById('viewThemes');
    const viewPrivacy = document.getElementById('viewPrivacy');

    const tabBtnGeneral = document.getElementById('tabBtnGeneral');
    const tabBtnThemes = document.getElementById('tabBtnThemes');
    const tabBtnPrivacy = document.getElementById('tabBtnPrivacy');

    const headerTitle = document.getElementById('headerTitle');
    const headerSubtitle = document.getElementById('headerSubtitle');
    const headerIconGeneral = document.getElementById('headerIconGeneral');
    const headerIconThemes = document.getElementById('headerIconThemes');
    const headerIconPrivacy = document.getElementById('headerIconPrivacy');

    if (viewGeneral) viewGeneral.classList.toggle('active', isGeneral);
    if (viewThemes) viewThemes.classList.toggle('active', isThemes);
    if (viewPrivacy) viewPrivacy.classList.toggle('active', isPrivacy);

    if (tabBtnGeneral) tabBtnGeneral.classList.toggle('active', isGeneral);
    if (tabBtnThemes) tabBtnThemes.classList.toggle('active', isThemes);
    if (tabBtnPrivacy) tabBtnPrivacy.classList.toggle('active', isPrivacy);

    if (headerTitle) {
      if (isThemes) headerTitle.textContent = 'Themes & Appearance';
      else if (isPrivacy) headerTitle.textContent = 'Privacy & Security';
      else headerTitle.textContent = 'Settings';
    }

    if (headerSubtitle) {
      if (isThemes) {
        headerSubtitle.textContent = 'Customize browser themes, accent highlights, and web page contrast';
      } else if (isPrivacy) {
        headerSubtitle.textContent = 'Zero-telemetry controls, anti-fingerprinting shields, and tracking protection';
      } else {
        headerSubtitle.textContent = 'Manage browser preferences, search, and system settings';
      }
    }

    if (headerIconGeneral) headerIconGeneral.style.display = isGeneral ? 'block' : 'none';
    if (headerIconThemes) headerIconThemes.style.display = isThemes ? 'block' : 'none';
    if (headerIconPrivacy) headerIconPrivacy.style.display = isPrivacy ? 'block' : 'none';
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

      const telemetryToggle = document.getElementById('telemetryToggle') as HTMLInputElement | null;
      if (telemetryToggle) telemetryToggle.checked = state.settings.telemetry_disabled !== false;
    }

    if (state.modules) {
      const darkMod = state.modules.find((m) => m.id === 'dark_reader');
      const drToggle = document.getElementById('darkReaderToggle') as HTMLInputElement | null;
      if (drToggle && darkMod) {
        drToggle.checked = !!darkMod.enabled;
      }
    }
  };

  // Expose global methods for inline HTML onclick attributes
  (window as unknown as Record<string, unknown>).switchView = switchView;
  (window as unknown as Record<string, unknown>).selectTheme = selectTheme;
  (window as unknown as Record<string, unknown>).selectAccent = selectAccent;
  (window as unknown as Record<string, unknown>).changeSearchEngine = changeSearchEngine;
  (window as unknown as Record<string, unknown>).toggleBookmarksBar = toggleBookmarksBar;
  (window as unknown as Record<string, unknown>).toggleDarkReader = toggleDarkReader;
  (window as unknown as Record<string, unknown>).setPrivacySetting = setPrivacySetting;
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
      { id: 'telemetryToggle', key: 'telemetry_disabled' },
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

    const clearBtn = document.getElementById('clearBrowsingDataBtn');
    if (clearBtn) {
      clearBtn.addEventListener('click', clearBrowsingData);
    }
  });

  // Blocklist and Whitelist Management
  let currentBlockedDomains: string[] = [];
  let currentWhitelistedDomains: string[] = [];
  let currentBlockedLogs: BlockedRequestLog[] = [];
  let currentRuleFilter: string = '';

  function cleanDomainInput(val: string): string {
    return val
      .trim()
      .toLowerCase()
      .replace(/^https?:\/\//, '')
      .replace(/\/.*$/, '');
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
        return `
          <div class="rule-item">
            <div class="rule-item-left">
              <span class="rule-cat-badge" style="background: ${cat.color}22; color: ${cat.color}; border: 1px solid ${cat.color}44;">
                ${cat.label}
              </span>
              <span class="rule-domain-text">${domain}</span>
            </div>
            <button class="rule-delete-btn" onclick="removeBlockedDomain('${domain}')" title="Remove rule">
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
          No whitelist exceptions configured. All blocklist rules apply universally.
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
              <span class="rule-domain-text">${domain}</span>
            </div>
            <button class="rule-delete-btn" onclick="removeWhitelistedDomain('${domain}')" title="Remove whitelist exception">
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
          No tracking or telemetry requests detected in this session. Browsing is completely clean.
        </div>
      `;
      return;
    }

    logListEl.innerHTML = currentBlockedLogs
      .slice(0, 30)
      .map((log) => {
        return `
          <div class="log-item">
            <div class="log-item-header">
              <span class="log-badge-type">${log.req_type.toUpperCase()}</span>
              <span class="log-domain-match">${log.domain}</span>
              <span class="log-timestamp">${log.timestamp}</span>
            </div>
            <div class="log-url-text" title="${log.url}">${log.url}</div>
          </div>
        `;
      })
      .join('');
  }

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

  // Update window.initSettings to populate blocklist, whitelist and activity logs
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
    }

    if (Array.isArray(state.blocked_logs)) {
      currentBlockedLogs = [...state.blocked_logs];
      renderBlockedActivityLogs();
    }
  };

  (window as unknown as Record<string, unknown>).addBlockedDomain = addBlockedDomain;
  (window as unknown as Record<string, unknown>).removeBlockedDomain = removeBlockedDomain;
  (window as unknown as Record<string, unknown>).addWhitelistedDomain = addWhitelistedDomain;
  (window as unknown as Record<string, unknown>).removeWhitelistedDomain = removeWhitelistedDomain;
  (window as unknown as Record<string, unknown>).resetPrivacyRules = resetPrivacyRules;
  (window as unknown as Record<string, unknown>).filterBlockedRules = filterBlockedRules;
})();

