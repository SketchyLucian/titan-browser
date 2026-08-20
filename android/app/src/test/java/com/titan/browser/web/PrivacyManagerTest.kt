package com.titan.browser.web

import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

class PrivacyManagerTest {
    @Test
    fun blocksKnownTelemetryHostsAndSubdomains() {
        assertTrue(PrivacyManager.isBlockedTelemetryHost("google-analytics.com"))
        assertTrue(PrivacyManager.isBlockedTelemetryHost("www.google-analytics.com"))
        assertTrue(PrivacyManager.isBlockedTelemetryHost("browser-intake-datadoghq.com"))
    }

    @Test
    fun doesNotBlockLookalikeOrNormalHosts() {
        assertFalse(PrivacyManager.isBlockedTelemetryHost("google-analytics.com.example.org"))
        assertFalse(PrivacyManager.isBlockedTelemetryHost("example.org"))
        assertFalse(PrivacyManager.isBlockedTelemetryHost(null))
    }
}
