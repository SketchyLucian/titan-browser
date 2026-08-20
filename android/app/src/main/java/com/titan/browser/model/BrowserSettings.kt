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
    val adblockBlockedDomains: List<String> = defaultAdblockDomains(),
    val adblockWhitelistedDomains: List<String> = emptyList(),
    val adblockFilterLists: List<String> = defaultAdblockFilterLists(),
    val adblockCustomRules: List<String> = emptyList(),
    val stripTrackingParameters: Boolean = true,
    val autoUpdateEnabled: Boolean = true
)

fun defaultAdblockFilterLists(): List<String> = listOf(
    "easylist",
    "easyprivacy",
    "ublock_filters",
    "ublock_badware",
    "ublock_privacy",
    "ublock_quick_fixes",
    "turtlecute_test"
)

fun defaultAdblockDomains(): List<String> = listOf(
    "doubleclick.net",
    "googleadservices.com",
    "googlesyndication.com",
    "adservice.google.com",
    "pagead2.googlesyndication.com",
    "adnxs.com",
    "advertising.com",
    "rubiconproject.com",
    "pubmatic.com",
    "criteo.com",
    "outbrain.com",
    "taboola.com",
    "popads.net",
    "popcash.net",
    "propellerads.com",
    "adcash.com",
    "bidswitch.net",
    "casalemedia.com",
    "openx.net",
    "smartadserver.com",
    "zedo.com",
    "amazon-adsystem.com",
    "adroll.com",
    "media.net",
    "moatads.com",
    "quantserve.com",
    "scorecardresearch.com",
    "adform.net",
    "ads-twitter.com",
    "revcontent.com",
    "mgid.com",
    "inmobi.com",
    "flashtalking.com",
    "exponential.com",
    "adcolony.com",
    "unityads.unity3d.com",
    "applovin.com",
    "vungle.com",
    "ironsrc.com",
    "chartboost.com",
    "adservice.com",
    "adserver.com"
)
