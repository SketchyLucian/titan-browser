package com.titan.browser.web

import com.titan.browser.model.BrowserSettings
import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable
import kotlinx.serialization.json.Json
import org.junit.Assert.assertEquals
import org.junit.BeforeClass
import org.junit.Test
import java.io.File

class AdblockManagerTest {

    companion object {
        @JvmStatic
        @BeforeClass
        fun initializeInjectionScript() {
            AdblockManager.initializeInjectionScriptTemplate(androidAdblockScriptFile().readText())
        }

        private fun androidAdblockScriptFile(): File {
            val candidates = listOf(
                File("../../web-scripts/dist/android-adblock.js"),
                File("../web-scripts/dist/android-adblock.js"),
                File("web-scripts/dist/android-adblock.js")
            )

            return candidates.firstOrNull { it.exists() }
                ?: error("Could not find generated Android adblock script from ${File(".").absolutePath}")
        }
    }

    @Serializable
    private data class AdblockContractCase(
        val name: String,
        val url: String,
        @SerialName("source_url")
        val sourceUrl: String,
        @SerialName("request_type")
        val requestType: String,
        val blocked: Boolean
    )

    @Test
    fun sharedAdblockContractMatchesAndroidBehavior() {
        val contract = Json.decodeFromString<List<AdblockContractCase>>(
            sharedContractFile().readText()
        )
        val settings = BrowserSettings()

        contract.forEach { case ->
            val blocked = AdblockManager.isBlockedUrl(
                url = case.url,
                settings = settings,
                sourceUrl = case.sourceUrl,
                requestType = case.requestType
            )

            assertEquals(case.name, case.blocked, blocked)
        }
    }

    @Test
    fun turtlecuteOpenSourceHostRulesMatchAndroidBehavior() {
        val settings = BrowserSettings()
        val hosts = sharedTurtlecuteFile()
            .readLines()
            .map { it.trim() }
            .filter { it.startsWith("||") }
            .map { it.removePrefix("||").trimEnd('^') }

        hosts.forEach { host ->
            val blocked = AdblockManager.isBlockedUrl(
                url = "https://$host/fakepage.html",
                settings = settings,
                sourceUrl = "https://adblock.turtlecute.org/",
                requestType = "other"
            )

            assertEquals("Turtlecute host should block: $host", true, blocked)
        }
    }

    @Test
    fun turtlecuteScriptAndThirdPartyRulesMatchAndroidBehavior() {
        val settings = BrowserSettings()

        assertEquals(
            true,
            AdblockManager.isBlockedUrl(
                url = "https://not-in-host-list.example/fakepage.html",
                settings = settings,
                sourceUrl = "https://adblock.turtlecute.org/",
                requestType = "other"
            )
        )
        assertEquals(
            false,
            AdblockManager.isBlockedUrl(
                url = "https://adblock.turtlecute.org/",
                settings = settings,
                sourceUrl = "https://adblock.turtlecute.org/",
                requestType = "document"
            )
        )
        assertEquals(
            true,
            AdblockManager.isBlockedUrl(
                url = "https://adblock.turtlecute.org/js/widget/ads.js",
                settings = settings,
                sourceUrl = "https://adblock.turtlecute.org/",
                requestType = "script"
            )
        )
        assertEquals(
            true,
            AdblockManager.isBlockedUrl(
                url = "https://adblock.turtlecute.org/js/pagead.js",
                settings = settings,
                sourceUrl = "https://adblock.turtlecute.org/",
                requestType = "script"
            )
        )

        val script = AdblockManager.getInjectionScript(settings, "https://adblock.turtlecute.org/")
        listOf(".textads", ".banner-ads", ".banner_ads", ".ad-unit", ".afs_ads", ".ad-zone", ".ad-space", ".adsbox")
            .forEach { selector ->
                assertEquals("Injection should include $selector", true, script.contains(selector))
            }
        assertEquals(
            "Blocked fetches must reject instead of returning a successful empty response",
            true,
            script.contains("throw new TypeError('Failed to fetch')")
        )
        assertEquals(
            "Fetch blocking must not special-case Turtlecute probe URLs",
            false,
            script.contains("/fakepage.html")
        )
    }

    @Test
    fun cachedMaintainedFilterListRulesAreUsed() {
        AdblockManager.setCachedFilterList(
            "easylist",
            """
                ! downloaded upstream-like list
                ||cached-ads.example^
                /js/index.js${'$'}script
                /css/index.css${'$'}stylesheet
                cached.example##.sponsored-card
            """.trimIndent()
        )

        val settings = BrowserSettings(adblockFilterLists = listOf("easylist"))
        assertEquals(
            true,
            AdblockManager.isBlockedUrl(
                url = "https://cached-ads.example/banner.js",
                settings = settings,
                sourceUrl = "https://news.example/",
                requestType = "script"
            )
        )

        val script = AdblockManager.getInjectionScript(settings, "https://cached.example/")
        assertEquals(true, script.contains(".sponsored-card"))

        assertEquals(
            "Downloaded generic path rules are skipped because the lightweight parser cannot apply full ABP context safely",
            false,
            AdblockManager.isBlockedUrl(
                url = "https://adblock.turtlecute.org/js/index.js",
                settings = settings,
                sourceUrl = "https://adblock.turtlecute.org/",
                requestType = "script"
            )
        )
        assertEquals(
            "Downloaded generic stylesheet path rules must not empty first-party CSS",
            false,
            AdblockManager.isBlockedUrl(
                url = "https://adblock.turtlecute.org/css/index.css",
                settings = settings,
                sourceUrl = "https://adblock.turtlecute.org/",
                requestType = "stylesheet"
            )
        )

        AdblockManager.setCachedFilterList("easylist", "")
    }

    @Test
    fun hostPathExceptionsDoNotAllowWholeDomain() {
        val settings = BrowserSettings(
            adblockFilterLists = emptyList(),
            adblockCustomRules = listOf(
                "@@||youtube.com/api/stats/playback",
                "||ads.youtube.com^"
            ),
            adblockBlockedDomains = emptyList()
        )

        assertEquals(
            true,
            AdblockManager.isBlockedUrl(
                url = "https://ads.youtube.com/fakepage.html",
                settings = settings,
                sourceUrl = "https://example.com/",
                requestType = "other"
            )
        )
    }

    @Test
    fun unsupportedProceduralCosmeticRulesAreSkipped() {
        AdblockManager.setCachedFilterList(
            "ublock_filters",
            """
                example.com##+js(set, foo, true)
                example.com##div:has-text(Advertisement)
                example.com##.plain-ad
            """.trimIndent()
        )

        val settings = BrowserSettings(adblockFilterLists = listOf("ublock_filters"))
        val script = AdblockManager.getInjectionScript(settings, "https://example.com/")

        assertEquals(false, script.contains("+js("))
        assertEquals(false, script.contains(":has-text"))
        assertEquals(true, script.contains(".plain-ad"))

        AdblockManager.setCachedFilterList("ublock_filters", "")
    }

    @Test
    fun requestTypeOptionsDoNotMatchEveryRequest() {
        val settings = BrowserSettings(
            adblockFilterLists = emptyList(),
            adblockCustomRules = listOf(
                "||document-only.example^\$doc",
                "||ping-only.example^\$ping"
            ),
            adblockBlockedDomains = emptyList()
        )

        assertEquals(
            false,
            AdblockManager.isBlockedUrl(
                url = "https://document-only.example/ad.js",
                settings = settings,
                sourceUrl = "https://site.example/",
                requestType = "script"
            )
        )
        assertEquals(
            true,
            AdblockManager.isBlockedUrl(
                url = "https://document-only.example/",
                settings = settings,
                sourceUrl = "https://site.example/",
                requestType = "document"
            )
        )
        assertEquals(
            false,
            AdblockManager.isBlockedUrl(
                url = "https://ping-only.example/pixel",
                settings = settings,
                sourceUrl = "https://site.example/",
                requestType = "image"
            )
        )
        assertEquals(
            true,
            AdblockManager.isBlockedUrl(
                url = "https://ping-only.example/pixel",
                settings = settings,
                sourceUrl = "https://site.example/",
                requestType = "ping"
            )
        )
    }

    @Test
    fun builtInPathHeuristicsDoNotBlockFirstPartyPagesOrAssets() {
        val settings = BrowserSettings(
            adblockFilterLists = emptyList(),
            adblockBlockedDomains = emptyList()
        )

        assertEquals(
            false,
            AdblockManager.isBlockedUrl(
                url = "https://example.com/ad/article",
                settings = settings,
                sourceUrl = "https://example.com/",
                requestType = "document"
            )
        )
        assertEquals(
            false,
            AdblockManager.isBlockedUrl(
                url = "https://example.com/ads/app.js",
                settings = settings,
                sourceUrl = "https://example.com/",
                requestType = "script"
            )
        )
    }

    @Test
    fun builtInPathHeuristicsStillBlockThirdPartyAdAssets() {
        val settings = BrowserSettings(
            adblockFilterLists = emptyList(),
            adblockBlockedDomains = emptyList()
        )

        assertEquals(
            true,
            AdblockManager.isBlockedUrl(
                url = "https://cdn.example-ad-server.test/ads/banner.js",
                settings = settings,
                sourceUrl = "https://publisher.example/",
                requestType = "script"
            )
        )
    }

    @Test
    fun whitelistAndExceptionsOverrideBuiltInHostBlocking() {
        val whitelisted = BrowserSettings(
            adblockFilterLists = emptyList(),
            adblockBlockedDomains = emptyList(),
            adblockWhitelistedDomains = listOf("googletagmanager.com")
        )
        assertEquals(
            false,
            AdblockManager.isBlockedUrl(
                url = "https://www.googletagmanager.com/gtm.js",
                settings = whitelisted,
                sourceUrl = "https://example.com/",
                requestType = "script"
            )
        )

        val excepted = BrowserSettings(
            adblockFilterLists = emptyList(),
            adblockBlockedDomains = emptyList(),
            adblockCustomRules = listOf("@@||googletagmanager.com^")
        )
        assertEquals(
            false,
            AdblockManager.isBlockedUrl(
                url = "https://www.googletagmanager.com/gtm.js",
                settings = excepted,
                sourceUrl = "https://example.com/",
                requestType = "script"
            )
        )
    }

    @Test
    fun injectedAnnoyanceCleanerRunsOnlyInAggressiveMode() {
        val defaultScript = AdblockManager.getInjectionScript(
            BrowserSettings(
                adblockFilterLists = emptyList(),
                adblockBlockedDomains = emptyList(),
                aggressiveMode = false
            ),
            "https://example.com/"
        )
        val aggressiveScript = AdblockManager.getInjectionScript(
            BrowserSettings(
                adblockFilterLists = emptyList(),
                adblockBlockedDomains = emptyList(),
                aggressiveMode = true
            ),
            "https://example.com/"
        )

        assertEquals(true, defaultScript.contains("if (aggressiveMode) {"))
        assertEquals(true, defaultScript.contains("\"aggressiveMode\":false"))
        assertEquals(true, aggressiveScript.contains("\"aggressiveMode\":true"))
    }

    private fun sharedContractFile(): File {
        val candidates = listOf(
            File("../../shared/adblock_contract.json"),
            File("../shared/adblock_contract.json"),
            File("shared/adblock_contract.json")
        )

        return candidates.firstOrNull { it.exists() }
            ?: error("Could not find shared/adblock_contract.json from ${File(".").absolutePath}")
    }

    private fun sharedTurtlecuteFile(): File {
        val candidates = listOf(
            File("../../shared/turtlecute_d3host.adblock"),
            File("../shared/turtlecute_d3host.adblock"),
            File("shared/turtlecute_d3host.adblock")
        )

        return candidates.firstOrNull { it.exists() }
            ?: error("Could not find shared/turtlecute_d3host.adblock from ${File(".").absolutePath}")
    }
}
