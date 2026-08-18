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
        const viewGeneral = document.getElementById('viewGeneral');
        const viewThemes = document.getElementById('viewThemes');
        const tabBtnGeneral = document.getElementById('tabBtnGeneral');
        const tabBtnThemes = document.getElementById('tabBtnThemes');
        const headerTitle = document.getElementById('headerTitle');
        const headerSubtitle = document.getElementById('headerSubtitle');
        const headerIconGeneral = document.getElementById('headerIconGeneral');
        const headerIconThemes = document.getElementById('headerIconThemes');
        if (viewGeneral)
            viewGeneral.classList.toggle('active', !isThemes);
        if (viewThemes)
            viewThemes.classList.toggle('active', isThemes);
        if (tabBtnGeneral)
            tabBtnGeneral.classList.toggle('active', !isThemes);
        if (tabBtnThemes)
            tabBtnThemes.classList.toggle('active', isThemes);
        if (headerTitle) {
            headerTitle.textContent = isThemes ? 'Themes & Appearance' : 'Settings';
        }
        if (headerSubtitle) {
            headerSubtitle.textContent = isThemes
                ? 'Customize browser themes, accent highlights, and web page contrast'
                : 'Manage browser preferences, search, and system settings';
        }
        if (headerIconGeneral)
            headerIconGeneral.style.display = isThemes ? 'none' : 'block';
        if (headerIconThemes)
            headerIconThemes.style.display = isThemes ? 'block' : 'none';
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
    function toggleDarkReader(enabled) {
        sendIpc({
            type: 'ToggleModule',
            module_id: 'dark_reader',
            enabled: enabled,
        });
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
        }
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
    window.toggleDarkReader = toggleDarkReader;
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
    });
})();
