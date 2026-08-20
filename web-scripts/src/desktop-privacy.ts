// Titan Browser - Desktop Privacy Protection (TypeScript)
// @ts-nocheck

interface TitanDesktopPrivacyConfig {
  doNotTrack: boolean;
  globalPrivacyControl: boolean;
  blockWebRtc: boolean;
  blockFingerprinting: boolean;
  blockHyperlinkAuditing: boolean;
  telemetryDisabled: boolean;
  mandatoryDomains: string[];
  blockedDomains: string[];
  whitelistedDomains: string[];
}

declare const __TITAN_DESKTOP_PRIVACY_CONFIG__: TitanDesktopPrivacyConfig;

(function () {
  const incomingConfig = __TITAN_DESKTOP_PRIVACY_CONFIG__;
  const privacyWindow = window as typeof window & {
    __titanPrivacyState?: { config: TitanDesktopPrivacyConfig };
  };

  if (privacyWindow.__titanPrivacyState) {
    privacyWindow.__titanPrivacyState.config = incomingConfig;
    return;
  }

  const state = { config: incomingConfig };
  privacyWindow.__titanPrivacyState = state;

  const matchesDomain = (host: string, domain: string): boolean =>
    host === domain || host.endsWith(`.${domain}`);

  const parseBlockedUrl = (value: string | URL): URL | null => {
    if (!state.config.telemetryDisabled) return null;
    try {
      const parsed = new URL(String(value), window.location.href);
      const host = parsed.hostname.toLowerCase().replace(/\.$/, '');
      if (state.config.mandatoryDomains.some((domain) => matchesDomain(host, domain))) return parsed;
      if (state.config.whitelistedDomains.some((domain) => matchesDomain(host, domain))) return null;
      return state.config.blockedDomains.some((domain) => matchesDomain(host, domain)) ? parsed : null;
    } catch (_) {
      return null;
    }
  };

  const reportBlocked = (parsed: URL, requestType: string): void => {
    try {
      window.ipc?.postMessage(JSON.stringify({
        type: 'ReportBlockedRequest',
        domain: parsed.hostname,
        url: `${parsed.origin}${parsed.pathname}`.slice(0, 300),
        req_type: requestType
      }));
    } catch (_) {
      // Blocking does not depend on activity reporting.
    }
  };

  const shouldBlock = (value: string | URL, requestType: string): boolean => {
    const parsed = parseBlockedUrl(value);
    if (!parsed) return false;
    reportBlocked(parsed, requestType);
    return true;
  };

  const defineDynamicProperty = (target: object, property: PropertyKey, getter: () => unknown): void => {
    try {
      Object.defineProperty(target, property, { configurable: true, get: getter });
    } catch (_) {
      // WebView2 can expose non-configurable properties on some versions.
    }
  };

  defineDynamicProperty(navigator, 'doNotTrack', () => state.config.doNotTrack ? '1' : null);
  defineDynamicProperty(window, 'doNotTrack', () => state.config.doNotTrack ? '1' : null);
  defineDynamicProperty(navigator, 'globalPrivacyControl', () => state.config.globalPrivacyControl);

  const originalSendBeacon = navigator.sendBeacon?.bind(navigator);
  if (originalSendBeacon) {
    navigator.sendBeacon = (url, data): boolean => {
      if (shouldBlock(url, 'beacon')) return false;
      return originalSendBeacon(url, data);
    };
  }

  const originalFetch = window.fetch.bind(window);
  window.fetch = async (input, init) => {
    const url = input instanceof Request ? input.url : input;
    if (shouldBlock(url, 'fetch')) throw new TypeError('Request blocked by Titan privacy protection');
    return originalFetch(input, init);
  };

  const xhrUrls = new WeakMap();
  const originalXhrOpen = XMLHttpRequest.prototype.open;
  XMLHttpRequest.prototype.open = function (method, url, ...args) {
    xhrUrls.set(this, String(url));
    return originalXhrOpen.call(this, method, url, ...args);
  };
  const originalXhrSend = XMLHttpRequest.prototype.send;
  XMLHttpRequest.prototype.send = function (...args) {
    const url = xhrUrls.get(this);
    if (url && shouldBlock(url, 'xhr')) {
      this.abort();
      return;
    }
    return originalXhrSend.apply(this, args);
  };

  const guardUrlProperty = (prototype, property: string, requestType: string, replacement: string): void => {
    const descriptor = Object.getOwnPropertyDescriptor(prototype, property);
    if (!descriptor?.set) return;
    try {
      Object.defineProperty(prototype, property, {
        configurable: true,
        get: descriptor.get,
        set(value: string) {
          descriptor.set.call(this, shouldBlock(value, requestType) ? replacement : value);
        }
      });
    } catch (_) {
      // The WebView2 socket rules still block the built-in telemetry list.
    }
  };

  guardUrlProperty(HTMLScriptElement.prototype, 'src', 'script', 'data:text/javascript,');
  guardUrlProperty(HTMLImageElement.prototype, 'src', 'image', 'data:image/gif;base64,R0lGODlhAQABAIAAAAAAAP///ywAAAAAAQABAAACAUwAOw==');
  guardUrlProperty(HTMLIFrameElement.prototype, 'src', 'subdocument', 'about:blank');

  const OriginalWebSocket = window.WebSocket;
  window.WebSocket = class TitanPrivacyWebSocket extends OriginalWebSocket {
    constructor(url, protocols) {
      if (shouldBlock(url, 'websocket')) {
        throw new DOMException('Connection blocked by Titan privacy protection', 'SecurityError');
      }
      super(url, protocols);
    }
  };

  if (window.EventSource) {
    const OriginalEventSource = window.EventSource;
    window.EventSource = class TitanPrivacyEventSource extends OriginalEventSource {
      constructor(url, options) {
        if (shouldBlock(url, 'eventsource')) {
          throw new DOMException('Connection blocked by Titan privacy protection', 'SecurityError');
        }
        super(url, options);
      }
    };
  }

  const OriginalPeerConnection = window.RTCPeerConnection;
  if (OriginalPeerConnection) {
    window.RTCPeerConnection = class TitanPrivacyPeerConnection extends OriginalPeerConnection {
      constructor(configuration) {
        if (state.config.blockWebRtc) {
          throw new DOMException('WebRTC is disabled by Titan privacy protection', 'NotAllowedError');
        }
        super(configuration);
      }
    };
  }

  const originalHardwareConcurrency = navigator.hardwareConcurrency;
  defineDynamicProperty(navigator, 'hardwareConcurrency', () =>
    state.config.blockFingerprinting ? 4 : originalHardwareConcurrency
  );
  const navigatorWithMemory = navigator as Navigator & { deviceMemory?: number };
  const originalDeviceMemory = navigatorWithMemory.deviceMemory;
  defineDynamicProperty(navigator, 'deviceMemory', () =>
    state.config.blockFingerprinting ? 4 : originalDeviceMemory
  );

  const removePingAttributes = (root: ParentNode): void => {
    if (!state.config.blockHyperlinkAuditing) return;
    root.querySelectorAll<HTMLAnchorElement>('a[ping]').forEach((anchor) => anchor.removeAttribute('ping'));
  };
  const observer = new MutationObserver((records) => {
    for (const record of records) {
      if (state.config.blockHyperlinkAuditing) {
        if (record.target instanceof HTMLAnchorElement && record.attributeName === 'ping') {
          record.target.removeAttribute('ping');
        }
        record.addedNodes.forEach((node) => {
          if (node instanceof HTMLAnchorElement) node.removeAttribute('ping');
          if (node instanceof Element) removePingAttributes(node);
        });
      }

      record.addedNodes.forEach((node) => {
        if (!(node instanceof Element)) return;
        node.querySelectorAll<HTMLElement>('script[src], img[src], iframe[src]').forEach((element) => {
          const source = element.getAttribute('src');
          if (source && shouldBlock(source, element.tagName.toLowerCase())) element.remove();
        });
      });
    }
  });

  const startDocumentProtection = (): void => {
    removePingAttributes(document);
    if (document.documentElement) {
      observer.observe(document.documentElement, {
        attributes: true,
        attributeFilter: ['ping'],
        childList: true,
        subtree: true
      });
    }
  };
  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', startDocumentProtection, { once: true });
  } else {
    startDocumentProtection();
  }
})();
