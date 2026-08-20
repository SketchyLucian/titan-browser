package com.titan.browser.web

import android.annotation.SuppressLint
import android.content.Context
import android.view.View
import android.webkit.CookieManager
import android.webkit.WebSettings
import android.webkit.WebView
import androidx.webkit.WebSettingsCompat
import androidx.webkit.WebViewFeature
import com.titan.browser.model.BrowserSettings

object TitanWebViewFactory {

    private const val DESKTOP_USER_AGENT =
        "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36"

    @Suppress("DEPRECATION")
    @SuppressLint("SetJavaScriptEnabled")
    fun configureSettings(
        webView: WebView,
        browserSettings: BrowserSettings = BrowserSettings(),
        isDesktopMode: Boolean = false,
        isDarkTheme: Boolean = true
    ) {
        val settings = webView.settings

        // 1. High Performance JavaScript & Storage
        settings.javaScriptEnabled = browserSettings.javascriptEnabled
        settings.domStorageEnabled = browserSettings.domStorageEnabled
        settings.databaseEnabled = browserSettings.domStorageEnabled
        settings.cacheMode = WebSettings.LOAD_DEFAULT

        // 2. Hardware Acceleration & Direct Window Compositing
        // Disable offscreenPreRaster to prevent raster worker thread CPU contention during rapid DOM updates
        settings.offscreenPreRaster = false
        // Avoid View.LAYER_TYPE_HARDWARE which creates an extra offscreen buffer copy;
        // WebView is already accelerated directly by the window's hardware canvas.
        webView.setLayerType(View.LAYER_TYPE_NONE, null)
        webView.isNestedScrollingEnabled = false
        webView.keepScreenOn = true
        webView.setRendererPriorityPolicy(WebView.RENDERER_PRIORITY_IMPORTANT, true)

        // 3. Viewport, Rendering Pipeline & Latency Minimization
        settings.loadWithOverviewMode = true
        settings.useWideViewPort = true
        settings.builtInZoomControls = true
        settings.displayZoomControls = false
        settings.setSupportZoom(true)
        settings.allowFileAccess = false
        settings.allowContentAccess = false
        settings.mediaPlaybackRequiresUserGesture = false
        settings.mixedContentMode = WebSettings.MIXED_CONTENT_COMPATIBILITY_MODE
        settings.setGeolocationEnabled(false)
        settings.setNeedInitialFocus(false)
        settings.setSupportMultipleWindows(false)

        // 4. SafeBrowsing & Security Check Bypass for Unthrottled Frame Transitions
        if (WebViewFeature.isFeatureSupported(WebViewFeature.SAFE_BROWSING_ENABLE)) {
            WebSettingsCompat.setSafeBrowsingEnabled(settings, false)
        }

        // 5. Cookie Manager Setup
        CookieManager.getInstance().setAcceptCookie(browserSettings.cookiesEnabled)
        CookieManager.getInstance().setAcceptThirdPartyCookies(
            webView,
            browserSettings.cookiesEnabled && !browserSettings.blockThirdPartyCookies
        )

        // 6. Desktop vs Mobile User Agent
        if (isDesktopMode) {
            settings.userAgentString = DESKTOP_USER_AGENT
        } else if (browserSettings.reduceFingerprinting) {
            settings.userAgentString = WebSettings.getDefaultUserAgent(webView.context)
                .replace(Regex("\\(Linux; Android [^)]*\\)"), "(Linux; Android 10; K; wv)")
        } else {
            settings.userAgentString = null // Default mobile UA
        }

        // 7. Web Theme & Native Dark Mode Strategy
        // Avoid FORCE_DARK_AUTO which injects dynamic color filter matrices on every DOM node during rendering
        if (WebViewFeature.isFeatureSupported(WebViewFeature.FORCE_DARK)) {
            WebSettingsCompat.setForceDark(
                settings,
                WebSettingsCompat.FORCE_DARK_OFF
            )
        }
        if (WebViewFeature.isFeatureSupported(WebViewFeature.ALGORITHMIC_DARKENING)) {
            WebSettingsCompat.setAlgorithmicDarkeningAllowed(settings, false)
        }
    }

    fun createWebView(
        context: Context,
        browserSettings: BrowserSettings = BrowserSettings()
    ): WebView {
        return WebView(context).apply {
            configureSettings(this, browserSettings)
        }
    }
}
