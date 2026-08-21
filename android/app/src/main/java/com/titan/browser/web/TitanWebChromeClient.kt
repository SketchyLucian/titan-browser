package com.titan.browser.web

import android.graphics.Bitmap
import android.os.Message
import android.net.Uri
import android.view.View
import android.webkit.GeolocationPermissions
import android.webkit.JsPromptResult
import android.webkit.JsResult
import android.webkit.PermissionRequest
import android.webkit.ValueCallback
import android.webkit.WebChromeClient
import android.webkit.WebView
import com.titan.browser.model.BrowserSettings
import java.net.URI

class TitanWebChromeClient(
    private val onProgressUpdate: (progress: Int) -> Unit,
    private val onTitleUpdate: (title: String) -> Unit,
    private val onFaviconUpdate: (icon: Bitmap?) -> Unit,
    private val onShowFullscreen: (view: View, callback: CustomViewCallback) -> Unit,
    private val onHideFullscreen: () -> Unit,
    private val settingsProvider: () -> BrowserSettings,
    private val onCreatePopupTab: () -> WebView?,
    private val onShowFileChooserRequest: (
        ValueCallback<Array<Uri>>,
        FileChooserParams
    ) -> Boolean,
    private val onGeolocationPermissionRequest: (
        String,
        GeolocationPermissions.Callback
    ) -> Unit,
    private val onWebPermissionRequest: (PermissionRequest) -> Unit
) : WebChromeClient() {

    override fun onProgressChanged(view: WebView?, newProgress: Int) {
        super.onProgressChanged(view, newProgress)
        onProgressUpdate(newProgress)
    }

    override fun onReceivedTitle(view: WebView?, title: String?) {
        super.onReceivedTitle(view, title)
        if (!title.isNullOrBlank()) {
            onTitleUpdate(title)
        }
    }

    override fun onReceivedIcon(view: WebView?, icon: Bitmap?) {
        super.onReceivedIcon(view, icon)
        onFaviconUpdate(icon)
    }

    override fun onJsAlert(
        view: WebView?,
        url: String?,
        message: String?,
        result: JsResult?
    ): Boolean {
        if (isScamDialog(message)) {
            result?.cancel()
            return true
        }
        return super.onJsAlert(view, url, message, result)
    }

    override fun onJsConfirm(
        view: WebView?,
        url: String?,
        message: String?,
        result: JsResult?
    ): Boolean {
        if (isScamDialog(message)) {
            result?.cancel()
            return true
        }
        return super.onJsConfirm(view, url, message, result)
    }

    override fun onJsPrompt(
        view: WebView?,
        url: String?,
        message: String?,
        defaultValue: String?,
        result: JsPromptResult?
    ): Boolean {
        if (isScamDialog(message)) {
            result?.cancel()
            return true
        }
        return super.onJsPrompt(view, url, message, defaultValue, result)
    }

    override fun onShowCustomView(view: View?, callback: CustomViewCallback?) {
        if (view != null && callback != null) {
            onShowFullscreen(view, callback)
        }
    }

    override fun onHideCustomView() {
        onHideFullscreen()
    }

    override fun onCreateWindow(
        view: WebView?,
        isDialog: Boolean,
        isUserGesture: Boolean,
        resultMsg: Message?
    ): Boolean {
        if (PopupPolicy.shouldBlockNewWindow(view?.url, isUserGesture, settingsProvider())) {
            return false
        }

        val transport = resultMsg?.obj as? WebView.WebViewTransport ?: return false
        val popupWebView = onCreatePopupTab() ?: return false
        transport.webView = popupWebView
        resultMsg.sendToTarget()
        view?.targetBlankUrl()?.let { url ->
            val headers = PrivacyManager.navigationHeaders(settingsProvider())
            popupWebView.postDelayed({
                if (popupWebView.url.isNullOrBlank() || popupWebView.url == "about:blank") {
                    popupWebView.loadUrl(url, headers)
                }
            }, 100)
        }
        return true
    }

    override fun onShowFileChooser(
        webView: WebView?,
        filePathCallback: ValueCallback<Array<Uri>>?,
        fileChooserParams: FileChooserParams?
    ): Boolean {
        if (filePathCallback == null || fileChooserParams == null) return false
        return onShowFileChooserRequest(filePathCallback, fileChooserParams)
    }

    override fun onGeolocationPermissionsShowPrompt(
        origin: String?,
        callback: GeolocationPermissions.Callback?
    ) {
        if (origin == null || callback == null) return
        onGeolocationPermissionRequest(origin, callback)
    }

    override fun onPermissionRequest(request: PermissionRequest?) {
        request?.let(onWebPermissionRequest)
    }

    private fun isScamDialog(message: String?): Boolean {
        val text = message?.lowercase().orEmpty()
        return text.contains("not a robot") ||
            text.contains("verify you are human") ||
            text.contains("click allow") ||
            text.contains("press allow") ||
            text.contains("tap allow") ||
            text.contains("payment has increased") ||
            text.contains("hurry up") ||
            text.contains("get your money") ||
            text.contains("claim prize") ||
            text.contains("you won") ||
            text.contains("congratulations") ||
            text.contains("file overload") ||
            text.contains("delete files") ||
            text.contains("cleanup is advised") ||
            text.contains("swift cleanup") ||
            text.contains("virus detected") ||
            text.contains("device infected") ||
            text.contains("security warning")
    }

    private fun WebView.targetBlankUrl(): String? {
        val hit = hitTestResult ?: return null
        val isAnchor = hit.type == WebView.HitTestResult.SRC_ANCHOR_TYPE ||
            hit.type == WebView.HitTestResult.SRC_IMAGE_ANCHOR_TYPE
        if (!isAnchor) return null
        val rawTarget = hit.extra?.takeIf { it.isNotBlank() } ?: return null
        return runCatching {
            val resolved = URI(url.orEmpty()).resolve(rawTarget).toString()
            resolved.takeIf {
                it.startsWith("http://", ignoreCase = true) ||
                    it.startsWith("https://", ignoreCase = true)
            }
        }.getOrNull()
    }
}
