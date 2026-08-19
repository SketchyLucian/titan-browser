package com.titan.browser.web

import android.content.Context
import android.content.Intent
import android.graphics.Bitmap
import android.net.Uri
import android.webkit.WebResourceError
import android.webkit.WebResourceRequest
import android.webkit.WebResourceResponse
import android.webkit.WebView
import android.webkit.WebViewClient
import com.titan.browser.model.BrowserSettings
import java.io.ByteArrayInputStream

class TitanWebViewClient(
    private val context: Context,
    private val settingsProvider: () -> BrowserSettings,
    private val onPageStartedCallback: (url: String) -> Unit,
    private val onPageFinishedCallback: (url: String, canGoBack: Boolean, canGoForward: Boolean) -> Unit,
    private val onErrorCallback: (errorCode: Int, description: String, failingUrl: String) -> Unit
) : WebViewClient() {

    override fun shouldOverrideUrlLoading(view: WebView?, request: WebResourceRequest?): Boolean {
        val rawUrl = request?.url?.toString() ?: return false
        val settings = settingsProvider()
        val url = if (settings.stripTrackingParameters) UrlUtils.stripTrackingParameters(rawUrl) else rawUrl

        // Handle standard web protocols internally
        if (url.startsWith("http://") || url.startsWith("https://") ||
            url.startsWith("about:") || url.startsWith("file://")
        ) {
            if (url != rawUrl) {
                view?.loadUrl(url)
                return true
            }
            return false
        }

        // Handle external schemes: mailto, tel, sms, intent, market, etc.
        return try {
            val intent = Intent(Intent.ACTION_VIEW, Uri.parse(url)).apply {
                flags = Intent.FLAG_ACTIVITY_NEW_TASK
            }
            context.startActivity(intent)
            true
        } catch (_: Exception) {
            true // Intercepted even if no handler app exists to avoid crash
        }
    }

    override fun shouldInterceptRequest(
        view: WebView?,
        request: WebResourceRequest?
    ): WebResourceResponse? {
        val settings = settingsProvider()
        if (settings.adblockEnabled) {
            val reqUrl = request?.url?.toString()
            if (reqUrl != null && AdblockManager.isBlockedUrl(reqUrl, settings.aggressiveMode)) {
                // Drop the ad / tracker request by returning an empty response
                return WebResourceResponse(
                    "text/plain",
                    "UTF-8",
                    ByteArrayInputStream(ByteArray(0))
                )
            }
        }
        return super.shouldInterceptRequest(view, request)
    }

    override fun onPageStarted(view: WebView?, url: String?, favicon: Bitmap?) {
        super.onPageStarted(view, url, favicon)
        if (url != null) {
            onPageStartedCallback(url)
            val settings = settingsProvider()
            if (settings.adblockEnabled && view != null) {
                val script = AdblockManager.getInjectionScript(settings)
                if (script.isNotEmpty()) {
                    view.evaluateJavascript(script, null)
                }
            }
        }
    }

    override fun onPageFinished(view: WebView?, url: String?) {
        super.onPageFinished(view, url)
        if (url != null && view != null) {
            val settings = settingsProvider()
            if (settings.adblockEnabled) {
                val script = AdblockManager.getInjectionScript(settings)
                if (script.isNotEmpty()) {
                    view.evaluateJavascript(script, null)
                }
            }
            onPageFinishedCallback(url, view.canGoBack(), view.canGoForward())
        }
    }

    override fun onReceivedError(
        view: WebView?,
        request: WebResourceRequest?,
        error: WebResourceError?
    ) {
        super.onReceivedError(view, request, error)
        if (request?.isForMainFrame == true && error != null) {
            onErrorCallback(
                error.errorCode,
                error.description.toString(),
                request.url.toString()
            )
        }
    }
}

