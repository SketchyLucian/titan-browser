package com.titan.browser.storage

import android.content.Context
import android.content.SharedPreferences
import android.util.Log
import androidx.core.content.edit
import com.titan.browser.model.Bookmark
import com.titan.browser.model.BrowserSession
import com.titan.browser.model.BrowserSettings
import com.titan.browser.model.HistoryEntry
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
        private const val KEY_HISTORY = "titan_history"
        private const val KEY_SESSION = "titan_session"
        private const val KEY_ADBLOCK_FILTER_PREFIX = "adblock_filter_"
        private const val TAG = "TitanStorage"
        private const val MAX_HISTORY_ENTRIES = 2_000

        val DEFAULT_BOOKMARKS = listOf(
            Bookmark("YouTube", "https://www.youtube.com"),
            Bookmark("Google", "https://www.google.com"),
            Bookmark("Rust Docs", "https://doc.rust-lang.org"),
            Bookmark("GitHub", "https://github.com"),
            Bookmark("Reddit", "https://reddit.com"),
            Bookmark("Wikipedia", "https://en.wikipedia.org")
        )

        internal fun updateHistory(
            history: List<HistoryEntry>,
            title: String,
            url: String,
            nowMs: Long
        ): List<HistoryEntry> {
            val previous = history.firstOrNull { it.url == url }
            val entry = HistoryEntry(
                title = title.ifBlank { url },
                url = url,
                lastVisitedMs = nowMs,
                visitCount = previous?.visitCount?.plus(1) ?: 1
            )
            return (listOf(entry) + history.filterNot { it.url == url })
                .take(MAX_HISTORY_ENTRIES)
        }
    }

    fun loadBookmarks(): List<Bookmark> {
        val raw = prefs.getString(KEY_BOOKMARKS, null)
        return if (raw != null) {
            try {
                json.decodeFromString<List<Bookmark>>(raw)
            } catch (error: Exception) {
                Log.e(TAG, "Could not read bookmarks", error)
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
            prefs.edit { putString(KEY_BOOKMARKS, raw) }
        } catch (error: Exception) {
            Log.e(TAG, "Could not save bookmarks", error)
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
            } catch (error: Exception) {
                Log.e(TAG, "Could not read settings", error)
                BrowserSettings().withPrivacyDefaults()
            }
        } else {
            BrowserSettings().withPrivacyDefaults()
        }
    }

    fun saveSettings(settings: BrowserSettings) {
        try {
            val raw = json.encodeToString(settings)
            prefs.edit { putString(KEY_SETTINGS, raw) }
        } catch (error: Exception) {
            Log.e(TAG, "Could not save settings", error)
        }
    }

    fun loadHistory(): List<HistoryEntry> = decodeList(KEY_HISTORY)

    fun recordHistoryVisit(title: String, url: String): List<HistoryEntry> {
        val updated = updateHistory(loadHistory(), title, url, System.currentTimeMillis())
        saveValue(KEY_HISTORY, updated)
        return updated
    }

    fun clearHistory() {
        prefs.edit { remove(KEY_HISTORY) }
    }

    fun loadSession(): BrowserSession = decodeValue(KEY_SESSION) ?: BrowserSession()

    fun saveSession(session: BrowserSession) {
        saveValue(KEY_SESSION, session)
    }

    fun loadAdblockFilterLists(ids: Collection<String>): Map<String, String> =
        ids.mapNotNull { id ->
            prefs.getString(KEY_ADBLOCK_FILTER_PREFIX + id, null)?.let { id to it }
        }.toMap()

    fun saveAdblockFilterList(id: String, content: String) {
        prefs.edit { putString(KEY_ADBLOCK_FILTER_PREFIX + id, content) }
    }

    private inline fun <reified T> decodeValue(key: String): T? {
        val raw = prefs.getString(key, null) ?: return null
        return try {
            json.decodeFromString<T>(raw)
        } catch (error: Exception) {
            Log.e(TAG, "Could not read $key", error)
            null
        }
    }

    private inline fun <reified T> decodeList(key: String): List<T> =
        decodeValue<List<T>>(key).orEmpty()

    private inline fun <reified T> saveValue(key: String, value: T) {
        try {
            prefs.edit { putString(key, json.encodeToString(value)) }
        } catch (error: Exception) {
            Log.e(TAG, "Could not save $key", error)
        }
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
