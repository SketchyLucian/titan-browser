interface TitanAndroidPrivacyConfig {
  doNotTrack: boolean;
  globalPrivacyControl: boolean;
  blockWebRtc: boolean;
  reduceFingerprinting: boolean;
  blockHyperlinkAuditing: boolean;
  blockedDomains: string[];
}

declare const __TITAN_ANDROID_PRIVACY_CONFIG__: TitanAndroidPrivacyConfig;

interface TitanPrivacyState {
  config: TitanAndroidPrivacyConfig;
}

interface TitanPrivacyWindow extends Window {
  __titanPrivacyState?: TitanPrivacyState;
}

(function () {
  const privacyWindow = window as TitanPrivacyWindow;
  const incomingConfig = __TITAN_ANDROID_PRIVACY_CONFIG__;
  if (privacyWindow.__titanPrivacyState) {
    privacyWindow.__titanPrivacyState.config = incomingConfig;
    return;
  }

  const state: TitanPrivacyState = { config: incomingConfig };
  privacyWindow.__titanPrivacyState = state;

  const matchesDomain = (host: string, domain: string): boolean =>
    host === domain || host.endsWith(`.${domain}`);

  const isBlockedUrl = (value: string | URL): boolean => {
    try {
      const host = new URL(String(value), window.location.href).hostname.toLowerCase().replace(/\.$/, '');
      return state.config.blockedDomains.some((domain) => matchesDomain(host, domain));
    } catch (_) {
      return false;
    }
  };

  const defineDynamicProperty = (
    target: object,
    property: PropertyKey,
    getter: () => unknown
  ): void => {
    try {
      Object.defineProperty(target, property, { configurable: true, get: getter });
    } catch (_) {
      // Some WebView builds expose non-configurable properties.
    }
  };

  defineDynamicProperty(navigator, 'doNotTrack', () => state.config.doNotTrack ? '1' : null);
  defineDynamicProperty(window, 'doNotTrack', () => state.config.doNotTrack ? '1' : null);
  defineDynamicProperty(navigator, 'globalPrivacyControl', () => state.config.globalPrivacyControl);

  const originalSendBeacon = navigator.sendBeacon?.bind(navigator);
  if (originalSendBeacon) {
    navigator.sendBeacon = (url: string | URL, data?: BodyInit | null): boolean => {
      if (isBlockedUrl(url)) return false;
      return originalSendBeacon(url, data);
    };
  }

  const originalFetch = window.fetch.bind(window);
  window.fetch = (async (input: RequestInfo | URL, init?: RequestInit): Promise<Response> => {
    const url = input instanceof Request ? input.url : input;
    if (isBlockedUrl(url)) throw new TypeError('Request blocked by Titan privacy protection');
    return originalFetch(input, init);
  }) as typeof window.fetch;

  const xhrUrls = new WeakMap<XMLHttpRequest, string>();
  const originalXhrOpen = XMLHttpRequest.prototype.open;
  XMLHttpRequest.prototype.open = function (
    method: string,
    url: string | URL,
    async?: boolean,
    username?: string | null,
    password?: string | null
  ): void {
    xhrUrls.set(this, String(url));
    originalXhrOpen.call(this, method, url, async ?? true, username, password);
  };
  const originalXhrSend = XMLHttpRequest.prototype.send;
  XMLHttpRequest.prototype.send = function (body?: Document | XMLHttpRequestBodyInit | null): void {
    const url = xhrUrls.get(this);
    if (url && isBlockedUrl(url)) {
      this.abort();
      return;
    }
    originalXhrSend.call(this, body);
  };

  const guardUrlProperty = (prototype: object, property: string, replacement: string): void => {
    const descriptor = Object.getOwnPropertyDescriptor(prototype, property);
    if (!descriptor?.set) return;
    try {
      Object.defineProperty(prototype, property, {
        configurable: true,
        get: descriptor.get,
        set(value: string) {
          descriptor.set?.call(this, isBlockedUrl(value) ? replacement : value);
        }
      });
    } catch (_) {
      // Native request interception remains active if a property cannot be wrapped.
    }
  };

  guardUrlProperty(HTMLScriptElement.prototype, 'src', 'data:text/javascript,');
  guardUrlProperty(HTMLImageElement.prototype, 'src', 'data:image/gif;base64,R0lGODlhAQABAIAAAAAAAP///ywAAAAAAQABAAACAUwAOw==');
  guardUrlProperty(HTMLIFrameElement.prototype, 'src', 'about:blank');

  const OriginalWebSocket = window.WebSocket;
  window.WebSocket = class TitanPrivacyWebSocket extends OriginalWebSocket {
    constructor(url: string | URL, protocols?: string | string[]) {
      if (isBlockedUrl(url)) throw new DOMException('Connection blocked by Titan privacy protection', 'SecurityError');
      super(url, protocols);
    }
  } as typeof WebSocket;

  if (window.EventSource) {
    const OriginalEventSource = window.EventSource;
    window.EventSource = class TitanPrivacyEventSource extends OriginalEventSource {
      constructor(url: string | URL, eventSourceInitDict?: EventSourceInit) {
        if (isBlockedUrl(url)) throw new DOMException('Connection blocked by Titan privacy protection', 'SecurityError');
        super(url, eventSourceInitDict);
      }
    } as typeof EventSource;
  }

  const OriginalPeerConnection = window.RTCPeerConnection;
  if (OriginalPeerConnection) {
    window.RTCPeerConnection = class TitanPrivacyPeerConnection extends OriginalPeerConnection {
      constructor(configuration?: RTCConfiguration) {
        if (state.config.blockWebRtc) {
          throw new DOMException('WebRTC is disabled by Titan privacy protection', 'NotAllowedError');
        }
        super(configuration);
      }
    } as typeof RTCPeerConnection;
  }

  const originalHardwareConcurrency = navigator.hardwareConcurrency;
  defineDynamicProperty(navigator, 'hardwareConcurrency', () =>
    state.config.reduceFingerprinting ? 4 : originalHardwareConcurrency
  );
  const navigatorWithMemory = navigator as Navigator & { deviceMemory?: number };
  const originalDeviceMemory = navigatorWithMemory.deviceMemory;
  defineDynamicProperty(navigator, 'deviceMemory', () =>
    state.config.reduceFingerprinting ? 4 : originalDeviceMemory
  );

  const removePingAttributes = (root: ParentNode): void => {
    if (!state.config.blockHyperlinkAuditing) return;
    root.querySelectorAll<HTMLAnchorElement>('a[ping]').forEach((anchor) => anchor.removeAttribute('ping'));
  };
  const observer = new MutationObserver((records) => {
    if (!state.config.blockHyperlinkAuditing) return;
    for (const record of records) {
      if (record.target instanceof HTMLAnchorElement && record.attributeName === 'ping') {
        record.target.removeAttribute('ping');
      }
      record.addedNodes.forEach((node) => {
        if (node instanceof HTMLAnchorElement) node.removeAttribute('ping');
        if (node instanceof Element) removePingAttributes(node);
      });
    }
  });

  const startAuditingProtection = (): void => {
    removePingAttributes(document);
    if (document.documentElement) {
      observer.observe(document.documentElement, { attributes: true, attributeFilter: ['ping'], childList: true, subtree: true });
    }
  };
  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', startAuditingProtection, { once: true });
  } else {
    startAuditingProtection();
  }
})();
