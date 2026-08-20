package com.titan.browser.web

import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import java.net.HttpURLConnection
import java.net.URL

object AdblockFilterUpdater {
    private const val CONNECT_TIMEOUT_MS = 10_000
    private const val READ_TIMEOUT_MS = 20_000
    private const val MAX_FILTER_LIST_BYTES = 12 * 1024 * 1024
    private const val USER_AGENT = "TitanBrowser/0.4 Android Adblock Updater"

    data class Result(
        val updated: Map<String, String>,
        val failed: Map<String, String>
    )

    suspend fun update(sources: List<AdblockManager.FilterListSource>): Result =
        withContext(Dispatchers.IO) {
            val updated = mutableMapOf<String, String>()
            val failed = mutableMapOf<String, String>()

            sources
                .filter { it.sourceUrl != null }
                .forEach { source ->
                    val url = source.sourceUrl ?: return@forEach
                    try {
                        updated[source.id] = downloadText(url)
                    } catch (error: Exception) {
                        failed[source.id] = error.message ?: error::class.java.simpleName
                    }
                }

            Result(updated = updated, failed = failed)
        }

    private fun downloadText(rawUrl: String): String {
        val connection = (URL(rawUrl).openConnection() as HttpURLConnection).apply {
            connectTimeout = CONNECT_TIMEOUT_MS
            readTimeout = READ_TIMEOUT_MS
            requestMethod = "GET"
            instanceFollowRedirects = true
            setRequestProperty("User-Agent", USER_AGENT)
            setRequestProperty("Accept", "text/plain,*/*")
        }

        return connection.use { conn ->
            val status = conn.responseCode
            if (status !in 200..299) {
                throw IllegalStateException("HTTP $status")
            }

            conn.inputStream.use { input ->
                val buffer = ByteArray(DEFAULT_BUFFER_SIZE)
                val output = StringBuilder()
                var totalBytes = 0

                while (true) {
                    val read = input.read(buffer)
                    if (read < 0) break
                    totalBytes += read
                    if (totalBytes > MAX_FILTER_LIST_BYTES) {
                        throw IllegalStateException("filter list exceeds $MAX_FILTER_LIST_BYTES bytes")
                    }
                    output.append(String(buffer, 0, read, Charsets.UTF_8))
                }

                output.toString()
            }
        }
    }

    private inline fun <T> HttpURLConnection.use(block: (HttpURLConnection) -> T): T =
        try {
            block(this)
        } finally {
            disconnect()
        }
}
