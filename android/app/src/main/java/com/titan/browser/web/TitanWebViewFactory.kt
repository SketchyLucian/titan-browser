package com.titan.browser.web

import android.annotation.SuppressLint
import android.content.Context
import android.view.View
import android.webkit.CookieManager
import android.webkit.WebSettings
import android.webkit.WebView
import androidx.webkit.WebSettingsCompat
import androidx.webkit.WebViewFeature
import com.titan.browser.BuildConfig
import com.titan.browser.model.BrowserSettings
import java.util.concurrent.atomic.AtomicBoolean

object TitanWebViewFactory {

    private const val DESKTOP_USER_AGENT =
        "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36"
    private val androidUserAgentSection = Regex("\\(Linux; Android [^)]*\\)")
    private val debuggingConfigured = AtomicBoolean(false)

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

        // 2. Hardware acceleration and direct window compositing
        settings.offscreenPreRaster = false
        webView.setLayerType(View.LAYER_TYPE_NONE, null)
        webView.isNestedScrollingEnabled = false
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
        settings.setSupportMultipleWindows(true)

        // 4. Keep provider-backed Safe Browsing enabled.
        if (WebViewFeature.isFeatureSupported(WebViewFeature.SAFE_BROWSING_ENABLE)) {
            WebSettingsCompat.setSafeBrowsingEnabled(settings, true)
        }

        // 5. Cookie Manager Setup
        val cookieManager = CookieManager.getInstance()
        cookieManager.setAcceptCookie(browserSettings.cookiesEnabled)
        cookieManager.setAcceptThirdPartyCookies(
            webView,
            browserSettings.cookiesEnabled && !browserSettings.blockThirdPartyCookies
        )

        // 6. Desktop vs Mobile User Agent
        if (isDesktopMode) {
            settings.userAgentString = DESKTOP_USER_AGENT
        } else if (browserSettings.reduceFingerprinting) {
            settings.userAgentString = WebSettings.getDefaultUserAgent(webView.context)
                .replace(androidUserAgentSection, "(Linux; Android 10; K; wv)")
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
            if (BuildConfig.DEBUG && debuggingConfigured.compareAndSet(false, true)) {
                WebView.setWebContentsDebuggingEnabled(true)
            }
            configureSettings(this, browserSettings)
        }
    }
}
