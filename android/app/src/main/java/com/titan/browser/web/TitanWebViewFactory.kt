package com.titan.browser.web

import android.annotation.SuppressLint
import android.content.Context
import android.webkit.CookieManager
import android.webkit.WebSettings
import android.webkit.WebView
import androidx.webkit.WebSettingsCompat
import androidx.webkit.WebViewFeature

object TitanWebViewFactory {

    private const val DESKTOP_USER_AGENT =
        "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36"

    @SuppressLint("SetJavaScriptEnabled")
    fun configureSettings(webView: WebView, isDesktopMode: Boolean = false, isDarkTheme: Boolean = true) {
        val settings = webView.settings

        settings.javaScriptEnabled = true
        settings.domStorageEnabled = true
        settings.databaseEnabled = true
        settings.loadWithOverviewMode = true
        settings.useWideViewPort = true
        settings.builtInZoomControls = true
        settings.displayZoomControls = false
        settings.setSupportZoom(true)
        settings.allowFileAccess = true
        settings.allowContentAccess = true
        settings.mediaPlaybackRequiresUserGesture = false
        settings.mixedContentMode = WebSettings.MIXED_CONTENT_COMPATIBILITY_MODE

        // Cookie manager setup
        CookieManager.getInstance().setAcceptCookie(true)
        CookieManager.getInstance().setAcceptThirdPartyCookies(webView, true)

        // Desktop vs Mobile User Agent
        if (isDesktopMode) {
            settings.userAgentString = DESKTOP_USER_AGENT
        } else {
            settings.userAgentString = null // Default mobile UA
        }

        // Dark mode support via AndroidX Webkit
        if (isDarkTheme) {
            webView.setBackgroundColor(android.graphics.Color.parseColor("#0F1015"))
            if (WebViewFeature.isFeatureSupported(WebViewFeature.ALGORITHMIC_DARKENING)) {
                WebSettingsCompat.setAlgorithmicDarkeningAllowed(settings, true)
            } else if (WebViewFeature.isFeatureSupported(WebViewFeature.FORCE_DARK)) {
                @Suppress("DEPRECATION")
                WebSettingsCompat.setForceDark(settings, WebSettingsCompat.FORCE_DARK_ON)
            }
        } else {
            webView.setBackgroundColor(android.graphics.Color.WHITE)
        }
    }

    fun createWebView(context: Context): WebView {
        return WebView(context).apply {
            setBackgroundColor(android.graphics.Color.parseColor("#0F1015"))
            configureSettings(this)
        }
    }
}
