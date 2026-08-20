package com.titan.browser.web

import android.annotation.SuppressLint
import android.content.Context
import android.view.View
import android.webkit.CookieManager
import android.webkit.WebSettings
import android.webkit.WebView
import androidx.webkit.WebSettingsCompat
import androidx.webkit.WebViewFeature

object TitanWebViewFactory {

    private const val DESKTOP_USER_AGENT =
        "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36"

    @Suppress("DEPRECATION")
    @SuppressLint("SetJavaScriptEnabled")
    fun configureSettings(webView: WebView, isDesktopMode: Boolean = false, isDarkTheme: Boolean = true) {
        val settings = webView.settings

        // 1. High Performance JavaScript & Storage
        settings.javaScriptEnabled = true
        settings.domStorageEnabled = true
        settings.databaseEnabled = true
        settings.cacheMode = WebSettings.LOAD_DEFAULT

        // 2. Hardware Acceleration & Multi-Core Pre-Rasterization
        settings.offscreenPreRaster = true
        // Avoid View.LAYER_TYPE_HARDWARE which creates an extra offscreen buffer copy;
        // WebView is already accelerated directly by the window's hardware canvas.
        webView.setLayerType(View.LAYER_TYPE_NONE, null)
        webView.isNestedScrollingEnabled = true
        webView.keepScreenOn = true

        // 3. Viewport & Rendering Pipeline
        settings.loadWithOverviewMode = true
        settings.useWideViewPort = true
        settings.builtInZoomControls = true
        settings.displayZoomControls = false
        settings.setSupportZoom(true)
        settings.allowFileAccess = false
        settings.allowContentAccess = true
        settings.mediaPlaybackRequiresUserGesture = false
        settings.mixedContentMode = WebSettings.MIXED_CONTENT_COMPATIBILITY_MODE

        // 4. Cookie Manager Setup
        CookieManager.getInstance().setAcceptCookie(true)
        CookieManager.getInstance().setAcceptThirdPartyCookies(webView, true)

        // 5. Desktop vs Mobile User Agent
        if (isDesktopMode) {
            settings.userAgentString = DESKTOP_USER_AGENT
        } else {
            settings.userAgentString = null // Default mobile UA
        }

        // 6. Web Theme & Native Dark Mode Strategy
        if (WebViewFeature.isFeatureSupported(WebViewFeature.FORCE_DARK_STRATEGY)) {
            WebSettingsCompat.setForceDarkStrategy(
                settings,
                WebSettingsCompat.DARK_STRATEGY_PREFER_WEB_THEME_OVER_USER_AGENT_DARKENING
            )
        }
        if (WebViewFeature.isFeatureSupported(WebViewFeature.FORCE_DARK)) {
            WebSettingsCompat.setForceDark(
                settings,
                if (isDarkTheme) WebSettingsCompat.FORCE_DARK_AUTO else WebSettingsCompat.FORCE_DARK_OFF
            )
        }
        if (WebViewFeature.isFeatureSupported(WebViewFeature.ALGORITHMIC_DARKENING)) {
            WebSettingsCompat.setAlgorithmicDarkeningAllowed(settings, false)
        }
    }

    fun createWebView(context: Context): WebView {
        return WebView(context).apply {
            configureSettings(this)
        }
    }
}
