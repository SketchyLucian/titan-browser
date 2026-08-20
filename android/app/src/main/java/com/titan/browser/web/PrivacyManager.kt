package com.titan.browser.web

import com.titan.browser.model.BrowserSettings
import kotlinx.serialization.Serializable
import kotlinx.serialization.encodeToString
import kotlinx.serialization.json.Json

object PrivacyManager {
    private const val INJECTION_CONFIG_PLACEHOLDER = "__TITAN_ANDROID_PRIVACY_CONFIG__"

    private val blockedTelemetryDomains = setOf(
        "pipe.aria.microsoft.com",
        "events.data.microsoft.com",
        "telemetry.microsoft.com",
        "watson.telemetry.microsoft.com",
        "mobile.pipe.aria.microsoft.com",
        "google-analytics.com",
        "analytics.google.com",
        "googletagmanager.com",
        "stats.g.doubleclick.net",
        "app-measurement.com",
        "crashlyticsreports-pa.googleapis.com",
        "crashlytics.com",
        "firebase-logging.googleapis.com",
        "firebaselogging-pa.googleapis.com",
        "sentry.io",
        "browser.sentry-cdn.com",
        "js.sentry-cdn.com",
        "sentry-cdn.com",
        "bugsnag.com",
        "sessions.bugsnag.com",
        "segment.io",
        "segment.com",
        "api.segment.io",
        "cdn.segment.com",
        "mixpanel.com",
        "api.mixpanel.com",
        "amplitude.com",
        "api2.amplitude.com",
        "api.amplitude.com",
        "clarity.ms",
        "hotjar.com",
        "hotjar.io",
        "static.hotjar.com",
        "fullstory.com",
        "mouseflow.com",
        "heapanalytics.com",
        "heap.io",
        "datadoghq.com",
        "browser-intake-datadoghq.com",
        "browser-intake-datadoghq.eu",
        "newrelic.com",
        "nr-data.net",
        "bam.nr-data.net",
        "loggly.com",
        "scorecardresearch.com",
        "quantserve.com",
        "bat.bing.com",
        "snap.licdn.com",
        "px.ads.linkedin.com",
        "analytics.twitter.com",
        "ads-twitter.com",
        "analytics.tiktok.com",
        "analytics.yahoo.com",
        "plausible.io"
    )

    @Serializable
    private data class ScriptConfig(
        val doNotTrack: Boolean,
        val globalPrivacyControl: Boolean,
        val blockWebRtc: Boolean,
        val reduceFingerprinting: Boolean,
        val blockHyperlinkAuditing: Boolean,
        val blockedDomains: List<String>
    )

    private data class CachedScript(
        val config: ScriptConfig,
        val script: String
    )

    private val sortedBlockedTelemetryDomains = blockedTelemetryDomains.sorted()

    @Volatile
    private var injectionScriptTemplate: String? = null

    @Volatile
    private var cachedScript: CachedScript? = null

    fun initializeInjectionScriptTemplate(template: String) {
        require(template.contains(INJECTION_CONFIG_PLACEHOLDER)) {
            "Android privacy script is missing its configuration placeholder"
        }
        synchronized(this) {
            injectionScriptTemplate = template
            cachedScript = null
        }
    }

    fun getInjectionScript(settings: BrowserSettings): String {
        val template = injectionScriptTemplate ?: return ""
        val config = ScriptConfig(
            doNotTrack = settings.doNotTrackEnabled,
            globalPrivacyControl = settings.globalPrivacyControlEnabled,
            blockWebRtc = settings.blockWebRtc,
            reduceFingerprinting = settings.reduceFingerprinting,
            blockHyperlinkAuditing = settings.blockHyperlinkAuditing,
            blockedDomains = sortedBlockedTelemetryDomains
        )
        cachedScript?.takeIf { it.config == config }?.let { return it.script }

        return synchronized(this) {
            cachedScript?.takeIf { it.config == config }?.script ?: template
                .replace(INJECTION_CONFIG_PLACEHOLDER, Json.encodeToString(config))
                .also { script -> cachedScript = CachedScript(config, script) }
        }
    }

    fun navigationHeaders(settings: BrowserSettings): Map<String, String> = buildMap {
        if (settings.doNotTrackEnabled) put("DNT", "1")
        if (settings.globalPrivacyControlEnabled) put("Sec-GPC", "1")
    }

    fun isBlockedTelemetryHost(host: String?): Boolean {
        var candidate = host.orEmpty().trim().trimEnd('.').lowercase()
        while (candidate.isNotEmpty()) {
            if (candidate in blockedTelemetryDomains) return true
            val dot = candidate.indexOf('.')
            if (dot < 0) return false
            candidate = candidate.substring(dot + 1)
        }
        return false
    }
}
