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
import java.util.concurrent.Executors
import java.util.concurrent.Future

class TitanWebViewClient(
    private val context: Context,
    private val settingsProvider: () -> BrowserSettings,
    private val onPageStartedCallback: (url: String) -> Unit,
    private val onPageFinishedCallback: (url: String, canGoBack: Boolean, canGoForward: Boolean) -> Unit,
    private val onErrorCallback: (errorCode: Int, description: String, failingUrl: String) -> Unit
) : WebViewClient() {

    @Volatile
    private var currentPageUrl: String = ""

    @Volatile
    private var pageGeneration = 0L

    @Volatile
    private var injectionTask: Future<*>? = null

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

    companion object {
        private val injectionExecutor = Executors.newFixedThreadPool(2) { runnable ->
            Thread(runnable, "titan-adblock-script").apply {
                isDaemon = true
            }
        }

        private fun blockedResponse(requestType: String): WebResourceResponse {
            val isScriptLike = requestType == "script" || requestType == "subdocument"
            val statusCode = if (isScriptLike) 403 else 204
            val reasonPhrase = if (isScriptLike) "Forbidden" else "No Content"
            val mimeType = if (requestType == "script") "application/javascript" else "text/plain"

            return WebResourceResponse(
                mimeType,
                "UTF-8",
                statusCode,
                reasonPhrase,
                mapOf(
                    "Cache-Control" to "no-store",
                    "Content-Length" to "0",
                    "X-Titan-Adblock" to "1"
                ),
                ByteArrayInputStream(ByteArray(0))
            )
        }
    }

    override fun shouldInterceptRequest(
        view: WebView?,
        request: WebResourceRequest?
    ): WebResourceResponse? {
        val settings = settingsProvider()
        if (!settings.adblockEnabled || request == null) {
            return super.shouldInterceptRequest(view, request)
        }

        val uri = request.url ?: return super.shouldInterceptRequest(view, request)
        val host = uri.host ?: ""

        // Fast-path bypass for benchmarks and local hosts
        if (host.isEmpty() || host.contains("browserbench") || host.contains("speedometer") ||
            host.contains("localhost") || host.contains("127.0.0.1")
        ) {
            return super.shouldInterceptRequest(view, request)
        }

        val reqUrl = uri.toString()
        val pageUrl = request.requestHeaders["Referer"].orEmpty().ifBlank { currentPageUrl }
        val requestType = inferRequestType(reqUrl, request.isForMainFrame)
        if (AdblockManager.isBlockedUrl(reqUrl, settings, pageUrl, requestType)) {
            return blockedResponse(requestType)
        }

        return super.shouldInterceptRequest(view, request)
    }

    override fun onPageStarted(view: WebView?, url: String?, favicon: Bitmap?) {
        super.onPageStarted(view, url, favicon)
        if (url != null) {
            currentPageUrl = url
            onPageStartedCallback(url)
            val settings = settingsProvider()
            if (settings.adblockEnabled && view != null && !url.contains("browserbench") && !url.contains("speedometer")) {
                val preparedScript = AdblockManager.getPreparedInjectionScript(settings, url)
                if (preparedScript == null) {
                    scheduleAdblockInjection(view, url, settings)
                } else if (preparedScript.isNotEmpty()) {
                    view.evaluateJavascript(preparedScript, null)
                }
            }
        }
    }

    override fun onPageFinished(view: WebView?, url: String?) {
        super.onPageFinished(view, url)
        if (url != null && view != null) {
            currentPageUrl = url
            onPageFinishedCallback(url, view.canGoBack(), view.canGoForward())
        }
    }

    private fun scheduleAdblockInjection(
        view: WebView,
        url: String,
        settings: BrowserSettings
    ) {
        val generation = pageGeneration + 1
        pageGeneration = generation
        injectionTask?.cancel(true)
        injectionTask = injectionExecutor.submit {
            val script = AdblockManager.getInjectionScript(settings, url)
            if (script.isEmpty() || Thread.currentThread().isInterrupted) return@submit

            view.post {
                if (pageGeneration == generation && settingsProvider().adblockEnabled) {
                    view.evaluateJavascript(script, null)
                }
            }
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

    private fun inferRequestType(url: String, isMainFrame: Boolean): String {
        if (isMainFrame) return "document"

        val lower = url.lowercase()
        return when {
            lower.endsWith(".js") || lower.contains(".js?") -> "script"
            lower.endsWith(".css") || lower.contains(".css?") -> "stylesheet"
            lower.endsWith(".png") || lower.endsWith(".jpg") || lower.endsWith(".jpeg") ||
                lower.endsWith(".gif") || lower.endsWith(".webp") || lower.endsWith(".svg") ||
                lower.contains(".png?") || lower.contains(".jpg?") || lower.contains(".jpeg?") ||
                lower.contains(".gif?") || lower.contains(".webp?") || lower.contains(".svg?") -> "image"
            lower.endsWith(".mp4") || lower.endsWith(".webm") || lower.endsWith(".m3u8") ||
                lower.contains(".mp4?") || lower.contains(".webm?") || lower.contains(".m3u8?") -> "media"
            lower.endsWith(".woff") || lower.endsWith(".woff2") || lower.endsWith(".ttf") ||
                lower.contains(".woff?") || lower.contains(".woff2?") || lower.contains(".ttf?") -> "font"
            lower.contains("/xhr") || lower.contains("/api/") || lower.contains("log_event") -> "xhr"
            else -> "other"
        }
    }
}
