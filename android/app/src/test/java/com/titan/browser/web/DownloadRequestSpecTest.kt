package com.titan.browser.web

import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test

class DownloadRequestSpecTest {

    @Test
    fun acceptsOnlyHttpAndHttpsDownloads() {
        assertEquals("https://example.test/file.pdf", spec("https://example.test/file.pdf").validatedUrl())
        assertEquals("http://localhost/file", spec("http://localhost/file").validatedUrl())
        assertNull(spec("javascript:alert(1)").validatedUrl())
        assertNull(spec("file:///data/data/secrets").validatedUrl())
        assertNull(spec("not a url").validatedUrl())
    }

    @Test
    fun stripsLineBreaksFromDownloadHeaders() {
        assertEquals("safe  injected", DownloadHandler.safeHeader("safe\r\ninjected"))
        assertNull(DownloadHandler.safeHeader("  "))
    }

    private fun spec(url: String) = DownloadRequestSpec(
        url = url,
        userAgent = null,
        contentDisposition = null,
        mimeType = null,
        contentLength = 0,
        referringUrl = null,
        cookieHeader = null
    )
}
