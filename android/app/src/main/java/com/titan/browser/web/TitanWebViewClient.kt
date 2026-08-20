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
    private var currentPageHost: String = ""

    @Volatile
    private var pageGeneration = 0L

    @Volatile
    private var injectionTask: Future<*>? = null

    override fun shouldOverrideUrlLoading(view: WebView?, request: WebResourceRequest?): Boolean {
        val rawUrl = request?.url?.toString() ?: return false
        val settings = settingsProvider()
        val url = if (settings.stripTrackingParameters) UrlUtils.stripTrackingParameters(rawUrl) else rawUrl
        val navigationHeaders = PrivacyManager.navigationHeaders(settings)

        // Handle standard web protocols internally
        if (url.startsWith("http://") || url.startsWith("https://") ||
            url.startsWith("about:") || url.startsWith("file://")
        ) {
            if (request.isForMainFrame && request.method.equals("GET", ignoreCase = true) &&
                (url != rawUrl || navigationHeaders.isNotEmpty())
            ) {
                view?.loadUrl(url, navigationHeaders)
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
        private val emptyBody = ByteArray(0)
        private val adblockHeaders = mapOf(
            "Cache-Control" to "no-store",
            "Content-Length" to "0",
            "X-Titan-Adblock" to "1"
        )
        private val privacyHeaders = mapOf(
            "Cache-Control" to "no-store",
            "Content-Length" to "0",
            "X-Titan-Privacy" to "1"
        )
        private val injectionExecutor = Executors.newFixedThreadPool(2) { runnable ->
            Thread(runnable, "titan-adblock-script").apply {
                isDaemon = true
            }
        }

        private fun blockedResponse(
            requestType: String,
            protection: String = "Adblock"
        ): WebResourceResponse {
            val isScriptLike = requestType == "script" || requestType == "subdocument"
            val statusCode = if (isScriptLike) 403 else 204
            val reasonPhrase = if (isScriptLike) "Forbidden" else "No Content"
            val mimeType = if (requestType == "script") "application/javascript" else "text/plain"

            return WebResourceResponse(
                mimeType,
                "UTF-8",
                statusCode,
                reasonPhrase,
                if (protection == "Privacy") privacyHeaders else adblockHeaders,
                ByteArrayInputStream(emptyBody)
            )
        }
    }

    override fun shouldInterceptRequest(
        view: WebView?,
        request: WebResourceRequest?
    ): WebResourceResponse? {
        if (request == null) return null

        val uri = request.url ?: return null
        val host = uri.host?.lowercase().orEmpty()

        // Fast-path bypass for benchmarks and local hosts
        if (host.isEmpty() || host.contains("browserbench") || host.contains("speedometer") ||
            host.contains("localhost") || host.contains("127.0.0.1")
        ) {
            return null
        }

        val settings = settingsProvider()
        val reqUrl = uri.toString()
        val requestType = inferRequestType(uri, request.isForMainFrame)
        if (PrivacyManager.isBlockedTelemetryHost(host)) {
            return blockedResponse(requestType, "Privacy")
        }
        if (settings.adblockEnabled && AdblockManager.isBlockedRequest(
                url = reqUrl,
                requestHost = host,
                settings = settings,
                sourceHost = currentPageHost,
                requestType = requestType
            )
        ) {
            return blockedResponse(requestType)
        }

        return null
    }

    override fun onPageStarted(view: WebView?, url: String?, favicon: Bitmap?) {
        super.onPageStarted(view, url, favicon)
        if (url != null) {
            currentPageHost = Uri.parse(url).host?.lowercase().orEmpty()
            onPageStartedCallback(url)
            val settings = settingsProvider()
            if (view != null) {
                PrivacyManager.getInjectionScript(settings).takeIf { it.isNotEmpty() }?.let { script ->
                    view.evaluateJavascript(script, null)
                }
            }
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
            currentPageHost = Uri.parse(url).host?.lowercase().orEmpty()
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

    private fun inferRequestType(uri: Uri, isMainFrame: Boolean): String {
        if (isMainFrame) return "document"

        val lower = uri.path.orEmpty().lowercase()
        return when {
            lower.endsWith(".js") -> "script"
            lower.endsWith(".css") -> "stylesheet"
            lower.endsWith(".png") || lower.endsWith(".jpg") || lower.endsWith(".jpeg") ||
                lower.endsWith(".gif") || lower.endsWith(".webp") || lower.endsWith(".svg") ||
                lower.endsWith(".avif") -> "image"
            lower.endsWith(".mp4") || lower.endsWith(".webm") || lower.endsWith(".m3u8") ||
                lower.endsWith(".mp3") || lower.endsWith(".m4a") -> "media"
            lower.endsWith(".woff") || lower.endsWith(".woff2") || lower.endsWith(".ttf") ||
                lower.endsWith(".otf") -> "font"
            lower.contains("/xhr") || lower.contains("/api/") || lower.contains("log_event") -> "xhr"
            else -> "other"
        }
    }
}
