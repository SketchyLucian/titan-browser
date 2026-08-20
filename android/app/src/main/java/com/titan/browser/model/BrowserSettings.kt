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
    val accentColorHex: String = "#4E7CF6",
    val adblockEnabled: Boolean = true,
    val blockVideoAds: Boolean = true,
    val cosmeticFiltering: Boolean = true,
    val blockPopups: Boolean = true,
    val aggressiveMode: Boolean = false,
    val stripTrackingParameters: Boolean = true,
    val autoUpdateEnabled: Boolean = true
)
