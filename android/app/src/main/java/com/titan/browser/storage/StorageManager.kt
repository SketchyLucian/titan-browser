package com.titan.browser.storage

import android.content.Context
import android.content.SharedPreferences
import com.titan.browser.model.Bookmark
import com.titan.browser.model.BrowserSettings
import kotlinx.serialization.encodeToString
import kotlinx.serialization.json.Json

class StorageManager(context: Context) {
    private val prefs: SharedPreferences =
        context.getSharedPreferences("titan_browser_prefs", Context.MODE_PRIVATE)

    private val json = Json {
        ignoreUnknownKeys = true
        encodeDefaults = true
    }

    companion object {
        private const val KEY_BOOKMARKS = "titan_bookmarks"
        private const val KEY_SETTINGS = "titan_settings"
        private const val KEY_ADBLOCK_FILTER_PREFIX = "adblock_filter_"

        val DEFAULT_BOOKMARKS = listOf(
            Bookmark("YouTube", "https://www.youtube.com"),
            Bookmark("Google", "https://www.google.com"),
            Bookmark("Rust Docs", "https://doc.rust-lang.org"),
            Bookmark("GitHub", "https://github.com"),
            Bookmark("Reddit", "https://reddit.com"),
            Bookmark("Wikipedia", "https://en.wikipedia.org")
        )
    }

    fun loadBookmarks(): List<Bookmark> {
        val raw = prefs.getString(KEY_BOOKMARKS, null)
        return if (raw != null) {
            try {
                json.decodeFromString<List<Bookmark>>(raw)
            } catch (_: Exception) {
                DEFAULT_BOOKMARKS
            }
        } else {
            saveBookmarks(DEFAULT_BOOKMARKS)
            DEFAULT_BOOKMARKS
        }
    }

    fun saveBookmarks(bookmarks: List<Bookmark>) {
        try {
            val raw = json.encodeToString(bookmarks)
            prefs.edit().putString(KEY_BOOKMARKS, raw).apply()
        } catch (_: Exception) {
        }
    }

    fun addBookmark(bookmark: Bookmark) {
        val current = loadBookmarks().toMutableList()
        // Replace existing URL or add
        val index = current.indexOfFirst { it.url == bookmark.url }
        if (index >= 0) {
            current[index] = bookmark
        } else {
            current.add(0, bookmark)
        }
        saveBookmarks(current)
    }

    fun removeBookmark(url: String) {
        val current = loadBookmarks().filterNot { it.url == url }
        saveBookmarks(current)
    }

    fun isBookmarked(url: String): Boolean {
        return loadBookmarks().any { it.url == url }
    }

    fun loadSettings(): BrowserSettings {
        val raw = prefs.getString(KEY_SETTINGS, null)
        return if (raw != null) {
            try {
                val decoded = json.decodeFromString<BrowserSettings>(raw)
                    .withDefaultAdblockLists()
                    .withPrivacyDefaults()
                saveSettings(decoded)
                decoded
            } catch (_: Exception) {
                BrowserSettings().withPrivacyDefaults()
            }
        } else {
            BrowserSettings().withPrivacyDefaults()
        }
    }

    fun saveSettings(settings: BrowserSettings) {
        try {
            val raw = json.encodeToString(settings)
            prefs.edit().putString(KEY_SETTINGS, raw).apply()
        } catch (_: Exception) {
        }
    }

    fun loadAdblockFilterLists(ids: Collection<String>): Map<String, String> =
        ids.mapNotNull { id ->
            prefs.getString(KEY_ADBLOCK_FILTER_PREFIX + id, null)?.let { id to it }
        }.toMap()

    fun saveAdblockFilterList(id: String, content: String) {
        prefs.edit().putString(KEY_ADBLOCK_FILTER_PREFIX + id, content).apply()
    }

    private fun BrowserSettings.withDefaultAdblockLists(): BrowserSettings {
        val merged = adblockFilterLists.toMutableList()
        if (!merged.contains("turtlecute_test")) merged.add("turtlecute_test")
        return copy(adblockFilterLists = merged)
    }

    private fun BrowserSettings.withPrivacyDefaults(): BrowserSettings =
        if (privacyMigrationVersion >= 1) {
            this
        } else {
            copy(
                autoUpdateEnabled = false,
                autoUpdateFilterLists = false,
                privacyMigrationVersion = 1
            )
        }
}
