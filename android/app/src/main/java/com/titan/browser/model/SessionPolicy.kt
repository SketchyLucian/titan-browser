package com.titan.browser.model

import java.net.URI

internal object SessionPolicy {
    fun isRestorableUrl(url: String): Boolean {
        if (url == "titan://newtab" || url == "about:blank") return true
        return runCatching {
            val parsed = URI(url)
            (parsed.scheme.equals("http", ignoreCase = true) ||
                parsed.scheme.equals("https", ignoreCase = true)) &&
                !parsed.host.isNullOrBlank()
        }.getOrDefault(false)
    }
}
