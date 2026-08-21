package com.titan.browser.model

import android.graphics.Bitmap
import android.webkit.WebView

data class Tab(
    val id: String = java.util.UUID.randomUUID().toString(),
    val url: String = "titan://newtab",
    val title: String = "New Tab",

    val favicon: Bitmap? = null,
    val isLoading: Boolean = false,
    val progress: Int = 0,
    val canGoBack: Boolean = false,
    val canGoForward: Boolean = false,
    val isDesktopMode: Boolean = false,
    val isPrivate: Boolean = false,
    val webView: WebView? = null
)
