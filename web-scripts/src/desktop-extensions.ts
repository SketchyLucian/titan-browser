// Titan Browser - Desktop Extension Store Integration Script
// @ts-nocheck

interface TitanDesktopExtensionsConfig {
    installedIds: string[];
}

declare const __TITAN_DESKTOP_EXTENSIONS_CONFIG__: TitanDesktopExtensionsConfig;

(function() {
    try {
        const config = typeof __TITAN_DESKTOP_EXTENSIONS_CONFIG__ !== 'undefined'
            ? __TITAN_DESKTOP_EXTENSIONS_CONFIG__
            : { installedIds: [] };

        const installedIds = new Set((config.installedIds || []).map(id => (id || '').toLowerCase()));
        const host = (window.location.hostname || '').toLowerCase();

        const isChromeStore = host.includes('chromewebstore.google.com') || host.includes('chrome.google.com');
        const isEdgeStore = host.includes('microsoftedge.microsoft.com');

        if (!isChromeStore && !isEdgeStore) return;

        function extractExtensionId(): { id: string, source: 'chrome' | 'edge' } | null {
            const urlStr = window.location.href;

            if (isChromeStore) {
                // Match /detail/<name>/<32-id> or /detail/<32-id> or any 32-char ID on detail page
                const detailMatch = urlStr.match(/\/detail\/(?:[^\/]+\/)?([a-z]{32})(?:[\?\/#]|$)/i);
                if (detailMatch && detailMatch[1]) {
                    return { id: detailMatch[1].toLowerCase(), source: 'chrome' };
                }
                if (urlStr.includes('/detail/')) {
                    const fallback = urlStr.match(/([a-z]{32})/i);
                    if (fallback && fallback[1]) {
                        return { id: fallback[1].toLowerCase(), source: 'chrome' };
                    }
                }
            } else if (isEdgeStore) {
                // Match /addons/detail/<name>/<32-id> or /addons/detail/<32-id>
                const detailMatch = urlStr.match(/\/addons\/detail\/(?:[^\/]+\/)?([a-z]{32})(?:[\?\/#]|$)/i);
                if (detailMatch && detailMatch[1]) {
                    return { id: detailMatch[1].toLowerCase(), source: 'edge' };
                }
                if (urlStr.includes('/detail/')) {
                    const fallback = urlStr.match(/([a-z]{32})/i);
                    if (fallback && fallback[1]) {
                        return { id: fallback[1].toLowerCase(), source: 'edge' };
                    }
                }
            }

            return null;
        }

        function injectTitanInstallBanner() {
            const ext = extractExtensionId();
            if (!ext) return;

            const existingBanner = document.getElementById('titan-store-install-banner');
            if (existingBanner) {
                if (existingBanner.getAttribute('data-ext-id') === ext.id) return;
                existingBanner.remove();
            }

            const isInstalled = installedIds.has(ext.id);

            const banner = document.createElement('div');
            banner.id = 'titan-store-install-banner';
            banner.setAttribute('data-ext-id', ext.id);
            banner.style.cssText = `
                position: fixed;
                bottom: 28px;
                right: 28px;
                z-index: 2147483647;
                display: flex;
                align-items: center;
                gap: 16px;
                background: #171821;
                color: #f0f2f8;
                border: 1px solid #363a4d;
                border-radius: 14px;
                padding: 14px 20px;
                box-shadow: 0 12px 36px rgba(0, 0, 0, 0.55);
                font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
                font-size: 14px;
                animation: titanSlideUp 0.25s ease-out;
            `;

            const styleSheet = document.createElement('style');
            styleSheet.textContent = `
                @keyframes titanSlideUp {
                    from { transform: translateY(20px); opacity: 0; }
                    to { transform: translateY(0); opacity: 1; }
                }
                .titan-install-btn {
                    background: #4e7cf6;
                    color: #ffffff;
                    border: none;
                    border-radius: 8px;
                    padding: 9px 20px;
                    font-size: 13.5px;
                    font-weight: 700;
                    cursor: pointer;
                    display: flex;
                    align-items: center;
                    gap: 6px;
                    transition: all 0.15s ease;
                    white-space: nowrap;
                }
                .titan-install-btn:hover {
                    background: #3b6ae8;
                    box-shadow: 0 4px 14px rgba(78, 124, 246, 0.45);
                }
                .titan-install-btn:disabled {
                    background: #272a38;
                    color: #8d92a6;
                    cursor: default;
                    box-shadow: none;
                }
            `;
            banner.appendChild(styleSheet);

            const iconHtml = `<svg viewBox="0 0 24 24" width="22" height="22" stroke="#4e7cf6" stroke-width="2" fill="none"><path d="M20.5 11H19V7c0-1.1-.9-2-2-2h-4V3.5a2.5 2.5 0 0 0-5 0V5H4c-1.1 0-1.99.9-1.99 2v3.8H3.5c1.49 0 2.7 1.21 2.7 2.7s-1.21 2.7-2.7 2.7H2V20c0 1.1.9 2 2 2h3.8v-1.5c0-1.49 1.21-2.7 2.7-2.7 1.49 0 2.7 1.21 2.7 2.7V22H17c1.1 0 2-.9 2-2v-4h1.5a2.5 2.5 0 0 0 0-5z"/></svg>`;

            const textDiv = document.createElement('div');
            textDiv.innerHTML = `<div style="font-weight: 700; font-size: 14px; display: flex; align-items: center; gap: 8px;">${iconHtml} Titan Browser Extension</div><div style="font-size: 12px; color: #8d92a6; margin-top: 2px;">Directly install from ${ext.source === 'chrome' ? 'Chrome Web Store' : 'Edge Add-ons'}</div>`;
            banner.appendChild(textDiv);

            const actionBtn = document.createElement('button');
            actionBtn.className = 'titan-install-btn';
            if (isInstalled) {
                actionBtn.textContent = 'Installed in Titan';
                actionBtn.disabled = true;
            } else {
                actionBtn.textContent = 'Add to Titan';
                actionBtn.onclick = function() {
                    actionBtn.disabled = true;
                    actionBtn.textContent = 'Installing...';
                    if (window.ipc && window.ipc.postMessage) {
                        window.ipc.postMessage(JSON.stringify({
                            type: 'InstallExtension',
                            id_or_url: ext.id,
                            source: ext.source
                        }));
                    }
                    setTimeout(() => {
                        actionBtn.textContent = 'Installed in Titan';
                        installedIds.add(ext.id);
                    }, 3000);
                };
            }
            banner.appendChild(actionBtn);

            const closeBtn = document.createElement('button');
            closeBtn.innerHTML = '&times;';
            closeBtn.style.cssText = 'background: none; border: none; color: #8d92a6; font-size: 22px; cursor: pointer; padding: 0 6px; line-height: 1;';
            closeBtn.onclick = () => banner.remove();
            banner.appendChild(closeBtn);

            const targetContainer = document.body || document.documentElement;
            if (targetContainer) {
                targetContainer.appendChild(banner);
            }
        }

        // Run on load and on SPA navigation changes
        if (document.readyState === 'loading') {
            document.addEventListener('DOMContentLoaded', injectTitanInstallBanner, { once: true });
        } else {
            injectTitanInstallBanner();
        }

        // Periodic check to ensure banner stays visible across dynamic SPA page transitions
        setInterval(injectTitanInstallBanner, 1000);

        let lastUrl = window.location.href;
        const pageObserver = new MutationObserver(() => {
            if (window.location.href !== lastUrl) {
                lastUrl = window.location.href;
                setTimeout(injectTitanInstallBanner, 300);
            }
        });
        if (document.documentElement) {
            pageObserver.observe(document.documentElement, { childList: true, subtree: true });
        }
        window.addEventListener('popstate', () => setTimeout(injectTitanInstallBanner, 300));
    } catch(e) {}
})();
