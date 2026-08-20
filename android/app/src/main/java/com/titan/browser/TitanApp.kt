package com.titan.browser

import android.app.Application
import androidx.webkit.WebViewCompat
import androidx.webkit.WebViewOutcomeReceiver
import androidx.webkit.WebViewStartUpConfig
import androidx.webkit.WebViewStartUpResult
import androidx.webkit.WebViewStartupException
import com.titan.browser.web.AdblockManager
import com.titan.browser.web.PrivacyManager
import java.util.concurrent.Executors

class TitanApp : Application() {
    private val webViewStartupExecutor = Executors.newSingleThreadExecutor { runnable ->
        Thread(runnable, "titan-webview-startup")
    }

    override fun onCreate() {
        super.onCreate()

        val adblockScript = assets.open("android-adblock.js").bufferedReader().use { it.readText() }
        AdblockManager.initializeInjectionScriptTemplate(adblockScript)
        val privacyScript = assets.open("android-privacy.js").bufferedReader().use { it.readText() }
        PrivacyManager.initializeInjectionScriptTemplate(privacyScript)

        val startupConfig = WebViewStartUpConfig.Builder(webViewStartupExecutor)
            .setShouldRunUiThreadStartUpTasks(false)
            .build()
        WebViewCompat.startUpWebView(
            this,
            startupConfig,
            object : WebViewOutcomeReceiver<WebViewStartUpResult, WebViewStartupException> {
                override fun onResult(result: WebViewStartUpResult) {
                    webViewStartupExecutor.shutdown()
                }

                override fun onError(error: WebViewStartupException) {
                    // WebView falls back to normal on-demand startup after a provider failure.
                    webViewStartupExecutor.shutdown()
                }
            }
        )
    }
}
