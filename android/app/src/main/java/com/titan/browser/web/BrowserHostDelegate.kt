package com.titan.browser.web

import android.net.Uri
import android.webkit.GeolocationPermissions
import android.webkit.PermissionRequest
import android.webkit.ValueCallback
import android.webkit.WebChromeClient

interface BrowserHostDelegate {
    fun onShowBrowserMessage(message: String)

    fun onOpenDownloads()

    fun onOpenDefaultBrowserSettings()

    fun onDownloadRequested(request: DownloadRequestSpec)

    fun onShowFileChooser(
        callback: ValueCallback<Array<Uri>>,
        params: WebChromeClient.FileChooserParams
    ): Boolean

    fun onGeolocationPermissionRequest(
        origin: String,
        callback: GeolocationPermissions.Callback
    )

    fun onWebPermissionRequest(request: PermissionRequest)
}
