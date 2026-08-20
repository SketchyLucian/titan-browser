// Titan Browser - Desktop Dynamic Adblock Page Script (TypeScript)
// @ts-nocheck

interface TitanDesktopDynamicAdblockConfig {
  css: string;
  scriptlet: string;
}

declare const __TITAN_DESKTOP_DYNAMIC_ADBLOCK_CONFIG__: TitanDesktopDynamicAdblockConfig;

(function() {
    const config = __TITAN_DESKTOP_DYNAMIC_ADBLOCK_CONFIG__;
    try {
        const css = config.css;
        if (css) {
            let style = document.getElementById('titan-dynamic-adblock-style');
            if (!style) {
                style = document.createElement('style');
                style.id = 'titan-dynamic-adblock-style';
                (document.head || document.documentElement).appendChild(style);
            }
            style.textContent = css + ' { display: none !important; visibility: hidden !important; height: 0 !important; max-height: 0 !important; width: 0 !important; opacity: 0 !important; pointer-events: none !important; }';
        }
        const scriptlet = config.scriptlet;
        if (scriptlet) {
            try {
                const fn = new Function(scriptlet);
                fn();
            } catch(e) {}
        }
    } catch(e) {}
})();
