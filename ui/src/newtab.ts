// Titan Browser - New Tab Page Controller (TypeScript)

interface NewTabInitState {
  theme?: string;
  accent_color?: string;
  search_engine?: string;
  bookmarks?: Array<{ title: string; url: string }>;
}

(function () {
  function sendIpc(message: IpcOutMessage) {
    if (window.ipc && window.ipc.postMessage) {
      window.ipc.postMessage(JSON.stringify(message));
    } else {
      console.log('NewTab IPC Out:', message);
    }
  }

  function navigate(url: string) {
    if (!url) return;
    sendIpc({ type: 'Navigate', url: url });
  }

  function handleSearch(e: Event) {
    e.preventDefault();
    const input = document.getElementById('searchInput') as HTMLInputElement | null;
    if (!input) return;
    const query = input.value.trim();
    if (query) {
      navigate(query);
    }
  }

  function initNewTab(state: NewTabInitState) {
    if (state.theme) {
      document.body.className = `theme-${state.theme}`;
    }

    if (state.accent_color) {
      document.documentElement.style.setProperty('--accent-primary', state.accent_color);
      document.documentElement.style.setProperty('--border-focus', state.accent_color);
    }

    if (state.search_engine) {
      const input = document.getElementById('searchInput') as HTMLInputElement | null;
      if (input) {
        input.placeholder = `Search with ${state.search_engine} or enter URL...`;
      }
    }
  }

  function setupEventListeners() {
    const searchForm = document.getElementById('searchForm');
    if (searchForm) {
      searchForm.addEventListener('submit', handleSearch);
    }

    const clickableElements = document.querySelectorAll<HTMLElement>('[data-url]');
    clickableElements.forEach((el) => {
      el.addEventListener('click', (e) => {
        e.preventDefault();
        const url = el.getAttribute('data-url');
        if (url) {
          navigate(url);
        }
      });
    });

    const input = document.getElementById('searchInput') as HTMLInputElement | null;
    if (input) {
      input.focus();
    }
  }

  // Global methods
  (window as unknown as Record<string, unknown>).navigate = navigate;
  (window as unknown as Record<string, unknown>).handleSearch = handleSearch;
  (window as unknown as Record<string, unknown>).initNewTab = initNewTab;

  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', setupEventListeners);
  } else {
    setupEventListeners();
  }
})();
