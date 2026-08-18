package com.titan.browser.web

import android.graphics.Bitmap
import android.view.View
import android.webkit.GeolocationPermissions
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
        // Allow geolocation if granted by app permissions
        callback?.invoke(origin, true, false)
    }
}
