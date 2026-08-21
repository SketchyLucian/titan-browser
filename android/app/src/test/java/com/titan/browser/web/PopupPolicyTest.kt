package com.titan.browser.web

import com.titan.browser.model.BrowserSettings
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

class PopupPolicyTest {

    @Test
    fun blocksNewWindowsByBehaviorInsteadOfDestination() {
        val settings = BrowserSettings(
            adblockFilterLists = emptyList(),
            adblockBlockedDomains = emptyList()
        )

        assertTrue(PopupPolicy.shouldBlockNewWindow("https://publisher.example/article", settings))
        assertTrue(PopupPolicy.shouldBlockNewWindow("https://another-site.test/watch", settings))
        assertTrue(PopupPolicy.shouldBlockNewWindow(null, settings))
    }

    @Test
    fun allowsNewWindowsWhenPopupProtectionIsDisabled() {
        val settings = BrowserSettings(blockPopups = false)

        assertFalse(PopupPolicy.shouldBlockNewWindow("https://example.com/", settings))
    }

    @Test
    fun allowsNewWindowsWhenAdblockIsDisabled() {
        val settings = BrowserSettings(adblockEnabled = false)

        assertFalse(PopupPolicy.shouldBlockNewWindow("https://example.com/", settings))
    }

    @Test
    fun allowsWhitelistedSitesAndTheirSubdomains() {
        val settings = BrowserSettings(adblockWhitelistedDomains = listOf("trusted.example"))

        assertFalse(PopupPolicy.shouldBlockNewWindow("https://trusted.example/", settings))
        assertFalse(PopupPolicy.shouldBlockNewWindow("https://account.trusted.example/login", settings))
        assertTrue(PopupPolicy.shouldBlockNewWindow("https://untrusted.example/", settings))
    }
}
