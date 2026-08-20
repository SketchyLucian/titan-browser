package com.titan.browser.update

import com.titan.browser.model.UpdateState
import com.titan.browser.model.UpdateStatus
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive
import java.net.HttpURLConnection
import java.net.URL

object UpdateChecker {
    private const val LATEST_RELEASE_URL =
        "https://api.github.com/repos/SketchyLucian/titan-browser/releases/latest"

    suspend fun check(currentVersion: String): UpdateState = withContext(Dispatchers.IO) {
        try {
            val connection = (URL(LATEST_RELEASE_URL).openConnection() as HttpURLConnection).apply {
                requestMethod = "GET"
                connectTimeout = 10_000
                readTimeout = 10_000
                setRequestProperty("Accept", "application/vnd.github+json")
                setRequestProperty("User-Agent", "Titan-Browser-Android-Updater")
            }

            try {
                if (connection.responseCode !in 200..299) {
                    return@withContext UpdateState(
                        currentVersion = currentVersion,
                        status = UpdateStatus.Error,
                        message = "Could not reach update service (${connection.responseCode})."
                    )
                }

                val body = connection.inputStream.bufferedReader().use { reader -> reader.readText() }
                val release = Json.parseToJsonElement(body).jsonObject
                val latestVersion = release["tag_name"]?.jsonPrimitive?.content ?: currentVersion
                val releaseUrl = release["html_url"]?.jsonPrimitive?.content

                if (isNewerVersion(latestVersion, currentVersion)) {
                    UpdateState(
                        currentVersion = currentVersion,
                        latestVersion = latestVersion,
                        releaseUrl = releaseUrl,
                        status = UpdateStatus.UpdateAvailable,
                        message = "Version $latestVersion is available."
                    )
                } else {
                    UpdateState(
                        currentVersion = currentVersion,
                        latestVersion = latestVersion,
                        releaseUrl = releaseUrl,
                        status = UpdateStatus.UpToDate,
                        message = "Titan Browser is up to date ($latestVersion)."
                    )
                }
            } finally {
                connection.disconnect()
            }
        } catch (error: Exception) {
            UpdateState(
                currentVersion = currentVersion,
                status = UpdateStatus.Error,
                message = "Could not check for updates: ${error.localizedMessage ?: "network error"}"
            )
        }
    }

    private fun isNewerVersion(candidate: String, current: String): Boolean {
        val candidateParts = parseVersion(candidate)
        val currentParts = parseVersion(current)
        val maxSize = maxOf(candidateParts.size, currentParts.size)

        for (index in 0 until maxSize) {
            val candidatePart = candidateParts.getOrElse(index) { 0 }
            val currentPart = currentParts.getOrElse(index) { 0 }
            if (candidatePart > currentPart) return true
            if (candidatePart < currentPart) return false
        }

        return false
    }

    private fun parseVersion(version: String): List<Int> {
        return version
            .trim()
            .removePrefix("v")
            .split(".")
            .map { part ->
                part.takeWhile { it.isDigit() }.toIntOrNull() ?: 0
            }
    }
}
