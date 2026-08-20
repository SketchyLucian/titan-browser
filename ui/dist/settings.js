"use strict";
// Titan Browser - Settings Controller (TypeScript)
(function () {
    function sendIpc(message) {
        if (window.ipc && window.ipc.postMessage) {
            window.ipc.postMessage(JSON.stringify(message));
        }
        else {
            console.log('Settings IPC Out:', message);
        }
    }
    function switchView(tabName) {
        const isThemes = tabName === 'themes';
        const isPrivacy = tabName === 'privacy';
        const isAdblock = tabName === 'adblock';
        const isGeneral = !isThemes && !isPrivacy && !isAdblock;
        const viewGeneral = document.getElementById('viewGeneral');
        const viewThemes = document.getElementById('viewThemes');
        const viewPrivacy = document.getElementById('viewPrivacy');
        const viewAdblock = document.getElementById('viewAdblock');
        const tabBtnGeneral = document.getElementById('tabBtnGeneral');
        const tabBtnThemes = document.getElementById('tabBtnThemes');
        const tabBtnPrivacy = document.getElementById('tabBtnPrivacy');
        const tabBtnAdblock = document.getElementById('tabBtnAdblock');
        const headerTitle = document.getElementById('headerTitle');
        const headerSubtitle = document.getElementById('headerSubtitle');
        const headerIconGeneral = document.getElementById('headerIconGeneral');
        const headerIconThemes = document.getElementById('headerIconThemes');
        const headerIconPrivacy = document.getElementById('headerIconPrivacy');
        const headerIconAdblock = document.getElementById('headerIconAdblock');
        if (viewGeneral)
            viewGeneral.classList.toggle('active', isGeneral);
        if (viewThemes)
            viewThemes.classList.toggle('active', isThemes);
        if (viewPrivacy)
            viewPrivacy.classList.toggle('active', isPrivacy);
        if (viewAdblock)
            viewAdblock.classList.toggle('active', isAdblock);
        if (tabBtnGeneral)
            tabBtnGeneral.classList.toggle('active', isGeneral);
        if (tabBtnThemes)
            tabBtnThemes.classList.toggle('active', isThemes);
        if (tabBtnPrivacy)
            tabBtnPrivacy.classList.toggle('active', isPrivacy);
        if (tabBtnAdblock)
            tabBtnAdblock.classList.toggle('active', isAdblock);
        if (headerTitle) {
            if (isThemes)
                headerTitle.textContent = 'Themes & Appearance';
            else if (isPrivacy)
                headerTitle.textContent = 'Privacy & Security';
            else if (isAdblock)
                headerTitle.textContent = 'AdBlock & Shields';
            else
                headerTitle.textContent = 'Settings';
        }
        if (headerSubtitle) {
            if (isThemes) {
                headerSubtitle.textContent = 'Customize browser themes, accent highlights, and web page contrast';
            }
            else if (isPrivacy) {
                headerSubtitle.textContent = 'Zero-telemetry controls, anti-fingerprinting shields, and tracking protection';
            }
            else if (isAdblock) {
                headerSubtitle.textContent = 'Shield controls, video ad auto-skip, popup defense, and custom domain filters';
            }
            else {
                headerSubtitle.textContent = 'Manage browser preferences, search, and system settings';
            }
        }
        if (headerIconGeneral)
            headerIconGeneral.style.display = isGeneral ? 'block' : 'none';
        if (headerIconThemes)
            headerIconThemes.style.display = isThemes ? 'block' : 'none';
        if (headerIconPrivacy)
            headerIconPrivacy.style.display = isPrivacy ? 'block' : 'none';
        if (headerIconAdblock)
            headerIconAdblock.style.display = isAdblock ? 'block' : 'none';
    }
    function selectTheme(themeId) {
        document.querySelectorAll('.theme-card').forEach((c) => {
            c.classList.toggle('active', c.getAttribute('data-theme') === themeId);
        });
        document.body.className = `theme-${themeId}`;
        sendIpc({ type: 'SetTheme', theme: themeId });
    }
    function selectAccent(color) {
        document.querySelectorAll('.accent-swatch').forEach((s) => {
            s.classList.toggle('active', s.getAttribute('data-color') === color);
        });
        document.documentElement.style.setProperty('--accent-primary', color);
        document.documentElement.style.setProperty('--border-focus', color);
        sendIpc({ type: 'SetAccentColor', color: color });
    }
    function changeSearchEngine(engine) {
        sendIpc({ type: 'SetSearchEngine', engine: engine });
    }
    function toggleBookmarksBar(show) {
        sendIpc({ type: 'SetShowBookmarksBar', show: show });
    }
    function setAutoUpdate(enabled) {
        sendIpc({ type: 'SetAutoUpdate', enabled: enabled });
    }
    function checkForUpdates() {
        renderUpdateState({
            current_version: '0.4.1',
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
    function renderUpdateState(updateState) {
        const statusEl = document.getElementById('updateStatusText');
        const versionEl = document.getElementById('updateVersionText');
        const checkBtn = document.getElementById('updateCheckBtn');
        const openBtn = document.getElementById('updateOpenBtn');
        if (!updateState)
            return;
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
    function toggleDarkReader(enabled) {
        sendIpc({
            type: 'ToggleModule',
            module_id: 'dark_reader',
            enabled: enabled,
        });
    }
    function setPrivacySetting(key, enabled) {
        sendIpc({
            type: 'SetPrivacySetting',
            key: key,
            enabled: enabled,
        });
    }
    function setAdblockSetting(key, enabled) {
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
    window.initSettings = function (state) {
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
                const sel = document.getElementById('searchEngineSelect');
                if (sel)
                    sel.value = state.settings.search_engine;
            }
            const bmToggle = document.getElementById('showBookmarksToggle');
            if (bmToggle)
                bmToggle.checked = !!state.settings.show_bookmarks_bar;
            const autoUpdateToggle = document.getElementById('autoUpdateToggle');
            if (autoUpdateToggle)
                autoUpdateToggle.checked = state.settings.auto_update_enabled !== false;
            // Privacy Settings
            const dntToggle = document.getElementById('dntToggle');
            if (dntToggle)
                dntToggle.checked = state.settings.do_not_track !== false;
            const gpcToggle = document.getElementById('gpcToggle');
            if (gpcToggle)
                gpcToggle.checked = state.settings.global_privacy_control !== false;
            const stripParamsToggle = document.getElementById('stripParamsToggle');
            if (stripParamsToggle)
                stripParamsToggle.checked = state.settings.strip_tracking_parameters !== false;
            const webrtcToggle = document.getElementById('webrtcToggle');
            if (webrtcToggle)
                webrtcToggle.checked = state.settings.block_webrtc_leak !== false;
            const fingerprintToggle = document.getElementById('fingerprintToggle');
            if (fingerprintToggle)
                fingerprintToggle.checked = state.settings.block_fingerprinting !== false;
            const auditingToggle = document.getElementById('auditingToggle');
            if (auditingToggle)
                auditingToggle.checked = state.settings.block_hyperlink_auditing !== false;
            const telemetryToggle = document.getElementById('telemetryToggle');
            if (telemetryToggle)
                telemetryToggle.checked = state.settings.telemetry_disabled !== false;
            // AdBlock Settings
            const adblockToggle = document.getElementById('adblockToggle');
            if (adblockToggle)
                adblockToggle.checked = state.settings.adblock_enabled !== false;
            const blockVideoAdsToggle = document.getElementById('blockVideoAdsToggle');
            if (blockVideoAdsToggle)
                blockVideoAdsToggle.checked = state.settings.adblock_block_video_ads !== false;
            const cosmeticFilteringToggle = document.getElementById('cosmeticFilteringToggle');
            if (cosmeticFilteringToggle)
                cosmeticFilteringToggle.checked = state.settings.adblock_cosmetic_filtering !== false;
            const blockPopupsToggle = document.getElementById('blockPopupsToggle');
            if (blockPopupsToggle)
                blockPopupsToggle.checked = state.settings.adblock_block_popups !== false;
            const aggressiveAdblockToggle = document.getElementById('aggressiveAdblockToggle');
            if (aggressiveAdblockToggle)
                aggressiveAdblockToggle.checked = !!state.settings.adblock_aggressive_mode;
        }
        renderUpdateState(state.update_state);
        if (state.modules) {
            const darkMod = state.modules.find((m) => m.id === 'dark_reader');
            const drToggle = document.getElementById('darkReaderToggle');
            if (drToggle && darkMod) {
                drToggle.checked = !!darkMod.enabled;
            }
        }
    };
    // Expose global methods for inline HTML onclick attributes
    window.switchView = switchView;
    window.selectTheme = selectTheme;
    window.selectAccent = selectAccent;
    window.changeSearchEngine = changeSearchEngine;
    window.toggleBookmarksBar = toggleBookmarksBar;
    window.setAutoUpdate = setAutoUpdate;
    window.checkForUpdates = checkForUpdates;
    window.openUpdateDownload = openUpdateDownload;
    window.toggleDarkReader = toggleDarkReader;
    window.setPrivacySetting = setPrivacySetting;
    window.setAdblockSetting = setAdblockSetting;
    window.clearBrowsingData = clearBrowsingData;
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
                if (theme)
                    selectTheme(theme);
            });
        });
        document.querySelectorAll('.accent-swatch').forEach((swatch) => {
            swatch.addEventListener('click', () => {
                const color = swatch.getAttribute('data-color');
                if (color)
                    selectAccent(color);
            });
        });
        const searchSelect = document.getElementById('searchEngineSelect');
        if (searchSelect) {
            searchSelect.addEventListener('change', (e) => {
                const target = e.target;
                changeSearchEngine(target.value);
            });
        }
        const bmToggle = document.getElementById('showBookmarksToggle');
        if (bmToggle) {
            bmToggle.addEventListener('change', (e) => {
                const target = e.target;
                toggleBookmarksBar(target.checked);
            });
        }
        const drToggle = document.getElementById('darkReaderToggle');
        if (drToggle) {
            drToggle.addEventListener('change', (e) => {
                const target = e.target;
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
            const toggle = document.getElementById(id);
            if (toggle) {
                toggle.addEventListener('change', (e) => {
                    const target = e.target;
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
            const toggle = document.getElementById(id);
            if (toggle) {
                toggle.addEventListener('change', (e) => {
                    const target = e.target;
                    setAdblockSetting(key, target.checked);
                });
            }
        });
        const clearBtn = document.getElementById('clearBrowsingDataBtn');
        if (clearBtn) {
            clearBtn.addEventListener('click', clearBrowsingData);
        }
    });
    // Privacy Blocklist & Whitelist Management
    let currentBlockedDomains = [];
    let currentWhitelistedDomains = [];
    let currentBlockedLogs = [];
    let currentRuleFilter = '';
    function cleanDomainInput(val) {
        return val
            .trim()
            .toLowerCase()
            .replace(/^https?:\/\//, '')
            .replace(/\/.*$/, '');
    }
    function getDomainCategory(d) {
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
        if (!listEl)
            return;
        const filtered = currentBlockedDomains.filter((d) => d.toLowerCase().includes(currentRuleFilter.toLowerCase()));
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
        if (!listEl)
            return;
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
        if (!logListEl)
            return;
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
    // AdBlock Engine Lists, Custom Rules, Stats, Whitelist & Logs
    let currentFilterLists = [];
    let currentCustomRules = [];
    let currentAdblockStats = null;
    let currentAdblockWhitelistedDomains = [];
    let currentAdblockLogs = [];
    function formatBytes(bytes) {
        if (!bytes || bytes === 0)
            return '0 KB';
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
        if (statAds)
            statAds.textContent = totalBlocked.toLocaleString();
        if (statBw)
            statBw.textContent = formatBytes(bwSaved);
        if (statCosm)
            statCosm.textContent = cosmeticCount.toLocaleString();
        if (statRules)
            statRules.textContent = totalRulesCount > 0 ? totalRulesCount.toLocaleString() : '120,000+';
    }
    function renderFilterLists() {
        const container = document.getElementById('filterListsContainer');
        if (!container)
            return;
        if (currentFilterLists.length === 0) {
            container.innerHTML = `<div class="rules-empty-state">Loading filter subscriptions...</div>`;
            return;
        }
        container.innerHTML = currentFilterLists
            .map((list) => {
            return `
          <div class="filter-list-item">
            <div class="filter-list-info">
              <div class="filter-list-name-row">
                <span class="filter-list-name">${list.name}</span>
                <span class="filter-list-count">${list.count.toLocaleString()} rules</span>
              </div>
              <div class="filter-list-desc">${list.description}</div>
            </div>
            <label class="switch" style="flex-shrink: 0;">
              <input type="checkbox" ${list.enabled ? 'checked' : ''} onchange="toggleFilterList('${list.id}', this.checked)" />
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
        if (!listEl)
            return;
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
            return `
          <div class="rule-item">
            <div class="rule-item-left">
              <span class="rule-cat-badge" style="background: ${badgeColor}22; color: ${badgeColor}; border: 1px solid ${badgeColor}44;">
                ${badgeLabel}
              </span>
              <span class="rule-domain-text" title="${rule}">${rule}</span>
            </div>
            <button class="rule-delete-btn" onclick="removeCustomRule('${safeRule}')" title="Remove rule">
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
        if (!listEl)
            return;
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
              <span class="rule-domain-text">${domain}</span>
            </div>
            <button class="rule-delete-btn" onclick="removeAdblockWhitelist('${domain}')" title="Remove whitelist exception">
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
        if (!logListEl)
            return;
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
            return `
          <div class="log-item">
            <div class="log-item-header">
              <span class="log-badge-type" style="background: rgba(78, 124, 246, 0.2); color: #60a5fa; border-color: rgba(78, 124, 246, 0.35);">
                ${log.req_type.toUpperCase()}
              </span>
              <span class="log-domain-match">${log.domain}</span>
              <span class="log-timestamp">${log.timestamp}</span>
            </div>
            <div class="log-url-text" title="${log.url}">${log.url}</div>
          </div>
        `;
        })
            .join('');
    }
    // Privacy Rule Actions
    function addBlockedDomain() {
        const input = document.getElementById('newBlockedDomainInput');
        if (!input)
            return;
        const domain = cleanDomainInput(input.value);
        if (!domain)
            return;
        sendIpc({ type: 'AddBlockedDomain', domain });
        input.value = '';
        if (!currentBlockedDomains.includes(domain)) {
            currentBlockedDomains.unshift(domain);
            renderBlockedDomainsList();
        }
    }
    function removeBlockedDomain(domain) {
        sendIpc({ type: 'RemoveBlockedDomain', domain });
        currentBlockedDomains = currentBlockedDomains.filter((d) => d !== domain);
        renderBlockedDomainsList();
    }
    function addWhitelistedDomain() {
        const input = document.getElementById('newWhitelistDomainInput');
        if (!input)
            return;
        const domain = cleanDomainInput(input.value);
        if (!domain)
            return;
        sendIpc({ type: 'AddWhitelistedDomain', domain });
        input.value = '';
        if (!currentWhitelistedDomains.includes(domain)) {
            currentWhitelistedDomains.unshift(domain);
            renderWhitelistedDomainsList();
        }
    }
    function removeWhitelistedDomain(domain) {
        sendIpc({ type: 'RemoveWhitelistedDomain', domain });
        currentWhitelistedDomains = currentWhitelistedDomains.filter((d) => d !== domain);
        renderWhitelistedDomainsList();
    }
    function resetPrivacyRules() {
        if (confirm('Reset all privacy blocklist rules and whitelist exceptions to Titan defaults?')) {
            sendIpc({ type: 'ResetPrivacyRules' });
        }
    }
    function filterBlockedRules(query) {
        currentRuleFilter = query;
        renderBlockedDomainsList();
    }
    // AdBlock / uBlock Actions
    function toggleFilterList(listId, enabled) {
        sendIpc({ type: 'ToggleFilterList', list_id: listId, enabled });
        const list = currentFilterLists.find((l) => l.id === listId);
        if (list) {
            list.enabled = enabled;
            renderAdblockStats();
        }
    }
    function addCustomRule() {
        const input = document.getElementById('newCustomRuleInput');
        if (!input)
            return;
        const rule = input.value.trim();
        if (!rule)
            return;
        sendIpc({ type: 'AddCustomFilterRule', rule });
        input.value = '';
        if (!currentCustomRules.includes(rule)) {
            currentCustomRules.unshift(rule);
            renderCustomRules();
            renderAdblockStats();
        }
    }
    function removeCustomRule(encodedRule) {
        const rule = decodeURIComponent(encodedRule);
        sendIpc({ type: 'RemoveCustomFilterRule', rule });
        currentCustomRules = currentCustomRules.filter((r) => r !== rule);
        renderCustomRules();
        renderAdblockStats();
    }
    function addAdblockWhitelist() {
        const input = document.getElementById('newAdblockWhitelistDomainInput');
        if (!input)
            return;
        const domain = cleanDomainInput(input.value);
        if (!domain)
            return;
        sendIpc({ type: 'AddAdblockWhitelist', domain });
        input.value = '';
        if (!currentAdblockWhitelistedDomains.includes(domain)) {
            currentAdblockWhitelistedDomains.unshift(domain);
            renderAdblockWhitelistedDomainsList();
        }
    }
    function removeAdblockWhitelist(domain) {
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
    // Update window.initSettings to populate blocklists, filter subscriptions, custom rules, stats & logs
    const origInitSettings = window.initSettings;
    window.initSettings = function (state) {
        if (origInitSettings)
            origInitSettings(state);
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
    };
    window.addBlockedDomain = addBlockedDomain;
    window.removeBlockedDomain = removeBlockedDomain;
    window.addWhitelistedDomain = addWhitelistedDomain;
    window.removeWhitelistedDomain = removeWhitelistedDomain;
    window.resetPrivacyRules = resetPrivacyRules;
    window.filterBlockedRules = filterBlockedRules;
    window.toggleFilterList = toggleFilterList;
    window.addCustomRule = addCustomRule;
    window.removeCustomRule = removeCustomRule;
    window.addAdblockWhitelist = addAdblockWhitelist;
    window.removeAdblockWhitelist = removeAdblockWhitelist;
    window.clearAdblockLogs = clearAdblockLogs;
})();
