package com.titan.browser

import android.app.Application
import android.os.Handler
import android.os.Looper
import android.webkit.WebView
import com.titan.browser.web.AdblockManager
import com.titan.browser.web.PrivacyManager
import com.titan.browser.web.TitanWebViewFactory

class TitanApp : Application() {
    override fun onCreate() {
        super.onCreate()

        val adblockScript = assets.open("android-adblock.js").bufferedReader().use { it.readText() }
        AdblockManager.initializeInjectionScriptTemplate(adblockScript)
        val privacyScript = assets.open("android-privacy.js").bufferedReader().use { it.readText() }
        PrivacyManager.initializeInjectionScriptTemplate(privacyScript)

        try {
            android.os.Process.setThreadPriority(android.os.Process.THREAD_PRIORITY_URGENT_DISPLAY)
        } catch (_: Exception) {
        }

        // Enable WebView debugging in debug builds
        if (BuildConfig.DEBUG) {
            WebView.setWebContentsDebuggingEnabled(true)
        }

        // Pre-warm Chromium engine, V8 JIT runtime, and GPU pipelines
        // by initializing a background instance as soon as the main looper is idle.
        Handler(Looper.getMainLooper()).post {
            try {
                val warmupWebView = TitanWebViewFactory.createWebView(this)
                warmupWebView.loadUrl("about:blank")
                warmupWebView.destroy()
            } catch (_: Exception) {
                // Non-fatal if system webview provider is updating
            }
        }
    }
}
