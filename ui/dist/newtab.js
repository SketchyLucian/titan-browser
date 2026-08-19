"use strict";
// Titan Browser - New Tab Page Controller (TypeScript)
(function () {
    function sendIpc(message) {
        if (window.ipc && window.ipc.postMessage) {
            window.ipc.postMessage(JSON.stringify(message));
        }
        else {
            console.log('NewTab IPC Out:', message);
        }
    }
    function navigate(url) {
        if (!url)
            return;
        sendIpc({ type: 'Navigate', url: url });
    }
    function handleSearch(e) {
        e.preventDefault();
        const input = document.getElementById('searchInput');
        if (!input)
            return;
        const query = input.value.trim();
        if (query) {
            navigate(query);
        }
    }
    function initNewTab(state) {
        if (state.theme) {
            document.body.className = `theme-${state.theme}`;
        }
        if (state.accent_color) {
            document.documentElement.style.setProperty('--accent-primary', state.accent_color);
            document.documentElement.style.setProperty('--border-focus', state.accent_color);
        }
        if (state.search_engine) {
            const input = document.getElementById('searchInput');
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
        const clickableElements = document.querySelectorAll('[data-url]');
        clickableElements.forEach((el) => {
            el.addEventListener('click', (e) => {
                e.preventDefault();
                const url = el.getAttribute('data-url');
                if (url) {
                    navigate(url);
                }
            });
        });
    }
    // Global methods
    window.navigate = navigate;
    window.handleSearch = handleSearch;
    window.initNewTab = initNewTab;
    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', setupEventListeners);
    }
    else {
        setupEventListeners();
    }
})();
