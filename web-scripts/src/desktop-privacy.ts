// Titan Browser - Desktop Privacy Page Script (TypeScript)
// @ts-nocheck

interface TitanDesktopPrivacyConfig {
  doNotTrack: boolean;
  globalPrivacyControl: boolean;
  blockWebRtc: boolean;
}

declare const __TITAN_DESKTOP_PRIVACY_CONFIG__: TitanDesktopPrivacyConfig;

(function() {
    const config = __TITAN_DESKTOP_PRIVACY_CONFIG__;
    try {
        const dnt = config.doNotTrack;
        const gpc = config.globalPrivacyControl;
        const blockWebrtc = config.blockWebRtc;

        if (dnt) {
            try {
                Object.defineProperty(navigator, 'doNotTrack', { get: () => '1', configurable: true });
                Object.defineProperty(window, 'doNotTrack', { get: () => '1', configurable: true });
            } catch(e) {}
        }

        if (gpc) {
            try {
                Object.defineProperty(navigator, 'globalPrivacyControl', { get: () => true, configurable: true });
            } catch(e) {}
        }

        if (blockWebrtc) {
            try {
                if (window.RTCPeerConnection) {
                    const origSetLocalDesc = window.RTCPeerConnection.prototype.setLocalDescription;
                    if (origSetLocalDesc) {
                        window.RTCPeerConnection.prototype.setLocalDescription = function(desc) {
                            if (desc && desc.sdp) {
                                desc.sdp = desc.sdp.replace(/a=candidate:.+typ host .+\r\n/g, '');
                            }
                            return origSetLocalDesc.call(this, desc);
                        };
                    }
                }
            } catch(e) {}
        }
    } catch(e) {}
})();
