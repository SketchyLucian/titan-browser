// Titan Browser - Standalone Themes Controller (TypeScript)

(function () {
  function sendIpc(message: IpcOutMessage) {
    if (window.ipc?.postMessage) {
      window.ipc.postMessage(JSON.stringify(message));
    } else {
      console.log('Themes IPC Out:', message);
    }
  }

  function selectTheme(themeId: string) {
    document.querySelectorAll<HTMLElement>('.theme-card').forEach((card) => {
      card.classList.toggle('active', card.dataset.theme === themeId);
    });
    document.body.className = `theme-${themeId}`;
    sendIpc({ type: 'SetTheme', theme: themeId });
  }

  function selectAccent(color: string) {
    document.querySelectorAll<HTMLElement>('.accent-swatch').forEach((swatch) => {
      swatch.classList.toggle('active', swatch.dataset.color === color);
    });
    document.documentElement.style.setProperty('--accent-primary', color);
    document.documentElement.style.setProperty('--border-focus', color);
    sendIpc({ type: 'SetAccentColor', color });
  }

  function toggleDarkReader(enabled: boolean) {
    sendIpc({ type: 'ToggleModule', module_id: 'dark_reader', enabled });
  }

  function initSettings(state: SettingsInitState) {
    if (state.settings) {
      const theme = state.settings.theme || 'titan-dark';
      document.body.className = `theme-${theme}`;
      document.querySelectorAll<HTMLElement>('.theme-card').forEach((card) => {
        card.classList.toggle('active', card.dataset.theme === theme);
      });

      const accent = state.settings.accent_color || '#4e7cf6';
      document.documentElement.style.setProperty('--accent-primary', accent);
      document.documentElement.style.setProperty('--border-focus', accent);
      document.querySelectorAll<HTMLElement>('.accent-swatch').forEach((swatch) => {
        swatch.classList.toggle('active', swatch.dataset.color === accent);
      });
    }

    const darkReader = state.modules?.find((module) => module.id === 'dark_reader');
    const darkReaderToggle = document.getElementById('darkReaderToggle') as HTMLInputElement | null;
    if (darkReaderToggle && darkReader) {
      darkReaderToggle.checked = darkReader.enabled;
    }
  }

  function setupEventListeners() {
    document.querySelectorAll<HTMLElement>('[data-action="open-settings"]').forEach((element) => {
      element.addEventListener('click', () => sendIpc({ type: 'OpenSettings' }));
    });

    document.querySelectorAll<HTMLElement>('.theme-card[data-theme]').forEach((card) => {
      card.addEventListener('click', () => {
        if (card.dataset.theme) selectTheme(card.dataset.theme);
      });
    });

    document.querySelectorAll<HTMLElement>('.accent-swatch[data-color]').forEach((swatch) => {
      swatch.addEventListener('click', () => {
        if (swatch.dataset.color) selectAccent(swatch.dataset.color);
      });
    });

    const darkReaderToggle = document.getElementById('darkReaderToggle') as HTMLInputElement | null;
    darkReaderToggle?.addEventListener('change', () => toggleDarkReader(darkReaderToggle.checked));
  }

  window.initSettings = initSettings;

  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', setupEventListeners);
  } else {
    setupEventListeners();
  }
})();
