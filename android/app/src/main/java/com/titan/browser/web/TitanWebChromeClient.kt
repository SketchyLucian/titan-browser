package com.titan.browser.web

import android.graphics.Bitmap
import android.view.View
import android.webkit.GeolocationPermissions
import android.webkit.JsPromptResult
import android.webkit.JsResult
import android.webkit.PermissionRequest
import android.webkit.WebChromeClient
import android.webkit.WebView

class TitanWebChromeClient(
    private val onProgressUpdate: (progress: Int) -> Unit,
    private val onTitleUpdate: (title: String) -> Unit,
    private val onFaviconUpdate: (icon: Bitmap?) -> Unit,
    private val onShowFullscreen: (view: View, callback: CustomViewCallback) -> Unit,
    private val onHideFullscreen: () -> Unit
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

    override fun onGeolocationPermissionsShowPrompt(
        origin: String?,
        callback: GeolocationPermissions.Callback?
    ) {
        callback?.invoke(origin, false, false)
    }

    override fun onPermissionRequest(request: PermissionRequest?) {
        request?.deny()
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
}
