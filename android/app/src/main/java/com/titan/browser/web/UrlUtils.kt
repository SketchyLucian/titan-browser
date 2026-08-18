package com.titan.browser.web

import android.util.Patterns
import com.titan.browser.model.SearchEngine
import java.net.URI

object UrlUtils {

    /**
     * Normalizes an input string into either a direct URL or a search engine query URL.
     * Supports prefixes:
     * - @yt or yt: -> YouTube search
     * - @gh or gh: -> GitHub search
     * - @ddg or ddg: -> DuckDuckGo search
     */
    fun normalizeOrSearch(rawInput: String, defaultEngine: SearchEngine = SearchEngine.GOOGLE): String {
        val input = rawInput.trim()
        if (input.isEmpty()) return "https://www.google.com"

        // Search engine shortcuts
        if (input.startsWith("@yt ") || input.startsWith("yt:")) {
            val query = input.substringAfter(" ").trim()
            return SearchEngine.YOUTUBE.buildQueryUrl(query)
        }
        if (input.startsWith("@gh ") || input.startsWith("gh:")) {
            val query = input.substringAfter(" ").trim()
            return SearchEngine.GITHUB.buildQueryUrl(query)
        }
        if (input.startsWith("@ddg ") || input.startsWith("ddg:")) {
            val query = input.substringAfter(" ").trim()
            return SearchEngine.DUCKDUCKGO.buildQueryUrl(query)
        }

        // Already has scheme
        if (input.startsWith("http://") || input.startsWith("https://") ||
            input.startsWith("file://") || input.startsWith("about:") || input.startsWith("titan://")
        ) {
            return input
        }

        // Localhost / IP address / Port checks
        if (input.startsWith("localhost") || input.startsWith("127.0.0.1") || input.matches(Regex("^\\d{1,3}\\.\\d{1,3}\\.\\d{1,3}\\.\\d{1,3}(:\\d+)?.*"))) {
            return "http://$input"
        }

        // Check if input looks like a valid domain or web URL (e.g. youtube.com, rust-lang.org/learn)
        val hasWhitespace = input.contains(" ")
        val isDomainLike = Patterns.WEB_URL.matcher(input).matches() ||
                (input.contains(".") && !hasWhitespace && !input.startsWith(".") && !input.endsWith("."))

        if (!hasWhitespace && isDomainLike) {
            return "https://$input"
        }

        // Otherwise fallback to search engine query
        return defaultEngine.buildQueryUrl(input)
    }

    fun getDomain(url: String): String {
        return try {
            val uri = URI(url)
            uri.host ?: url
        } catch (_: Exception) {
            url
        }
    }

    fun isSecure(url: String): Boolean {
        return url.startsWith("https://")
    }
}
