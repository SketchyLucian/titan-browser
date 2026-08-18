package com.titan.browser.model

import kotlinx.serialization.Serializable

@Serializable
enum class SearchEngine(val displayName: String, val searchUrl: String, val iconName: String) {
    GOOGLE("Google", "https://www.google.com/search?q=%s", "google"),
    DUCKDUCKGO("DuckDuckGo", "https://duckduckgo.com/?q=%s", "duckduckgo"),
    BING("Bing", "https://www.bing.com/search?q=%s", "bing"),
    BRAVE("Brave", "https://search.brave.com/search?q=%s", "brave"),
    YOUTUBE("YouTube", "https://www.youtube.com/results?search_query=%s", "youtube"),
    GITHUB("GitHub", "https://github.com/search?q=%s", "github");

    fun buildQueryUrl(query: String): String {
        val encoded = java.net.URLEncoder.encode(query, "UTF-8")
        return searchUrl.replace("%s", encoded)
    }

    companion object {
        fun fromName(name: String): SearchEngine {
            return entries.firstOrNull { it.displayName.equals(name, ignoreCase = true) } ?: GOOGLE
        }
    }
}
