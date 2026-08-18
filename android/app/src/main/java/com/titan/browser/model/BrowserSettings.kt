package com.titan.browser.model

import kotlinx.serialization.Serializable

@Serializable
data class BrowserSettings(
    val searchEngine: String = "Google",
    val darkTheme: Boolean = true,
    val desktopSiteByDefault: Boolean = false,
    val javascriptEnabled: Boolean = true,
    val cookiesEnabled: Boolean = true,
    val domStorageEnabled: Boolean = true,
    val accentColorHex: String = "#4E7CF6"
)
