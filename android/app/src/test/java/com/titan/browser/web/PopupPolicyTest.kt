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

        assertTrue(PopupPolicy.shouldBlockNewWindow("https://publisher.example/article", false, settings))
        assertTrue(PopupPolicy.shouldBlockNewWindow("https://another-site.test/watch", false, settings))
        assertTrue(PopupPolicy.shouldBlockNewWindow(null, false, settings))
    }

    @Test
    fun allowsUserInitiatedWindowsForLoginAndPaymentFlows() {
        val settings = BrowserSettings()

        assertFalse(PopupPolicy.shouldBlockNewWindow("https://shop.example/", true, settings))
    }

    @Test
    fun allowsNewWindowsWhenPopupProtectionIsDisabled() {
        val settings = BrowserSettings(blockPopups = false)

        assertFalse(PopupPolicy.shouldBlockNewWindow("https://example.com/", false, settings))
    }

    @Test
    fun allowsNewWindowsWhenAdblockIsDisabled() {
        val settings = BrowserSettings(adblockEnabled = false)

        assertFalse(PopupPolicy.shouldBlockNewWindow("https://example.com/", false, settings))
    }

    @Test
    fun allowsWhitelistedSitesAndTheirSubdomains() {
        val settings = BrowserSettings(adblockWhitelistedDomains = listOf("trusted.example"))

        assertFalse(PopupPolicy.shouldBlockNewWindow("https://trusted.example/", false, settings))
        assertFalse(PopupPolicy.shouldBlockNewWindow("https://account.trusted.example/login", false, settings))
        assertTrue(PopupPolicy.shouldBlockNewWindow("https://untrusted.example/", false, settings))
    }
}
