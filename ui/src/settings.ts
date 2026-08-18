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
    document.querySelectorAll('.nav-item').forEach((item) => {
      item.classList.toggle('active', item.getAttribute('data-tab') === tabName);
    });

    document.querySelectorAll('.tab-pane').forEach((pane) => {
      pane.classList.toggle('active', pane.id === `${tabName}Tab`);
    });
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

  function toggleDarkReader(enabled: boolean) {
    sendIpc({
      type: 'ToggleModule',
      module_id: 'dark_reader',
      enabled: enabled,
    });
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
    }

    if (state.modules) {
      const darkMod = state.modules.find((m) => m.id === 'dark_reader');
      const drToggle = document.getElementById('darkReaderToggle') as HTMLInputElement | null;
      if (drToggle && darkMod) {
        drToggle.checked = !!darkMod.enabled;
      }
    }
  };

  // Event Listeners
  document.addEventListener('DOMContentLoaded', () => {
    document.querySelectorAll('.nav-item').forEach((item) => {
      item.addEventListener('click', () => {
        const tab = item.getAttribute('data-tab');
        if (tab) switchView(tab);
      });
    });

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
        sendIpc({ type: 'SetSearchEngine', engine: target.value });
      });
    }

    const bmToggle = document.getElementById('showBookmarksToggle') as HTMLInputElement | null;
    if (bmToggle) {
      bmToggle.addEventListener('change', (e) => {
        const target = e.target as HTMLInputElement;
        sendIpc({ type: 'SetShowBookmarksBar', show: target.checked });
      });
    }

    const drToggle = document.getElementById('darkReaderToggle') as HTMLInputElement | null;
    if (drToggle) {
      drToggle.addEventListener('change', (e) => {
        const target = e.target as HTMLInputElement;
        toggleDarkReader(target.checked);
      });
    }
  });

  // Global methods for inline callers
  window.switchView = switchView;
  window.selectTheme = selectTheme;
  window.selectAccent = selectAccent;
  window.toggleDarkReader = toggleDarkReader;
})();
