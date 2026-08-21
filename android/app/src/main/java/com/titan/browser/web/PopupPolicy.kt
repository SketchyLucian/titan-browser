package com.titan.browser.web

import com.titan.browser.model.BrowserSettings
import java.net.URI

internal object PopupPolicy {

    fun shouldBlockNewWindow(
        sourceUrl: String?,
        settings: BrowserSettings
    ): Boolean {
        if (!settings.adblockEnabled || !settings.blockPopups) return false

        val sourceHost = runCatching {
            URI(sourceUrl.orEmpty()).host?.lowercase().orEmpty()
        }.getOrDefault("")

        return !AdblockManager.isWhitelistedHost(
            sourceHost,
            settings.adblockWhitelistedDomains
        )
    }
}
