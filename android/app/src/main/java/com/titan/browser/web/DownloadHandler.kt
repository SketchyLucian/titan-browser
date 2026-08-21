package com.titan.browser.web

import android.app.DownloadManager
import android.content.Context
import android.os.Environment
import android.webkit.URLUtil
import androidx.core.net.toUri
import java.net.URI

data class DownloadRequestSpec(
    val url: String,
    val userAgent: String?,
    val contentDisposition: String?,
    val mimeType: String?,
    val contentLength: Long,
    val referringUrl: String?,
    val cookieHeader: String?
) {
    fun validatedUrl(): String? = runCatching {
        val parsed = URI(url)
        url.takeIf {
            parsed.scheme.equals("http", ignoreCase = true) && !parsed.host.isNullOrBlank() ||
                parsed.scheme.equals("https", ignoreCase = true) && !parsed.host.isNullOrBlank()
        }
    }.getOrNull()
}

object DownloadHandler {

    fun enqueue(context: Context, spec: DownloadRequestSpec): Result<Long> = runCatching {
        val url = requireNotNull(spec.validatedUrl()) { "Unsupported download URL" }
        val fileName = URLUtil.guessFileName(url, spec.contentDisposition, spec.mimeType)
        val request = DownloadManager.Request(url.toUri()).apply {
            setTitle(fileName)
            setDescription("Downloading with Titan Browser")
            setNotificationVisibility(
                DownloadManager.Request.VISIBILITY_VISIBLE_NOTIFY_COMPLETED
            )
            setDestinationInExternalPublicDir(Environment.DIRECTORY_DOWNLOADS, fileName)
            setAllowedOverMetered(true)
            setAllowedOverRoaming(true)
            spec.mimeType?.takeIf { it.isNotBlank() }?.let(::setMimeType)

            safeHeader(spec.userAgent)?.let { addRequestHeader("User-Agent", it) }
            safeHeader(spec.referringUrl)?.let { addRequestHeader("Referer", it) }
            safeHeader(spec.cookieHeader)?.let {
                addRequestHeader("Cookie", it)
            }
        }

        val manager = context.getSystemService(DownloadManager::class.java)
            ?: error("Android download service is unavailable")
        manager.enqueue(request)
    }

    internal fun safeHeader(value: String?): String? = value
        ?.replace('\r', ' ')
        ?.replace('\n', ' ')
        ?.trim()
        ?.takeIf { it.isNotEmpty() }
}
