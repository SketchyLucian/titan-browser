package com.titan.browser.model

import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

class SessionPolicyTest {

    @Test
    fun restoresOnlyWebAndTitanNewTabUrls() {
        assertTrue(SessionPolicy.isRestorableUrl("https://example.test/path"))
        assertTrue(SessionPolicy.isRestorableUrl("http://localhost/page"))
        assertTrue(SessionPolicy.isRestorableUrl("titan://newtab"))
        assertFalse(SessionPolicy.isRestorableUrl("javascript:alert(1)"))
        assertFalse(SessionPolicy.isRestorableUrl("file:///data/data/secret"))
        assertFalse(SessionPolicy.isRestorableUrl("not a url"))
    }
}
