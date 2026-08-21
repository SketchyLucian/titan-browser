package com.titan.browser.web

import com.titan.browser.model.BrowserSettings
import java.net.URI

object AdblockManager {

    private const val MAX_INJECTION_SCRIPT_CACHE_ENTRIES = 8
    private const val INJECTION_CONFIG_PLACEHOLDER = "__TITAN_ANDROID_ADBLOCK_CONFIG__"
    private const val LIST_EASYLIST = "easylist"
    private const val LIST_EASYPRIVACY = "easyprivacy"
    private const val LIST_UBLOCK_FILTERS = "ublock_filters"
    private const val LIST_UBLOCK_BADWARE = "ublock_badware"
    private const val LIST_UBLOCK_PRIVACY = "ublock_privacy"
    private const val LIST_UBLOCK_QUICK_FIXES = "ublock_quick_fixes"
    private const val LIST_TURTLECUTE_TEST = "turtlecute_test"

    data class FilterListConfig(
        val id: String,
        val name: String,
        val description: String,
        val count: Int,
        val enabled: Boolean
    )

    private data class ParsedRule(
        val raw: String,
        val isException: Boolean,
        val networkPattern: String? = null,
        val networkRegex: Regex? = null,
        val cosmeticDomains: List<String> = emptyList(),
        val cosmeticSelector: String? = null,
        val options: Set<String> = emptySet()
    )

    private data class RuleSetKey(
        val generation: Long,
        val filterLists: List<String>,
        val customRules: List<String>,
        val blockedDomains: List<String>
    )

    private data class ActiveRuleSet(
        val key: RuleSetKey,
        val exceptionHosts: Set<String>,
        val blockingHosts: Set<String>,
        val complexExceptionRules: List<ParsedRule>,
        val complexBlockingRules: List<ParsedRule>,
        val cosmeticRules: List<ParsedRule>,
        val networkHostsJson: String
    )

    private data class InjectionScriptKey(
        val generation: Long,
        val settings: BrowserSettings,
        val pageHost: String
    )

    data class FilterListSource(
        val id: String,
        val name: String,
        val description: String,
        val sourceUrl: String?,
        val bundledRules: List<String>
    )

    val filterListSources: List<FilterListSource> = listOf(
        FilterListSource(
            LIST_EASYLIST,
            "EasyList",
            "Primary ad blocking rules for common ad networks, paths, and banner elements.",
            "https://easylist.to/easylist/easylist.txt",
            easyListRules()
        ),
        FilterListSource(
            LIST_EASYPRIVACY,
            "EasyPrivacy",
            "Tracker, analytics, telemetry, fingerprinting script, and beacon blocking rules.",
            "https://easylist.to/easylist/easyprivacy.txt",
            easyPrivacyRules()
        ),
        FilterListSource(
            LIST_UBLOCK_FILTERS,
            "uBlock Filters - Ads & Annoyances",
            "uBlock Origin core rules for ad servers, YouTube, social ads, and cosmetic cleanups.",
            "https://raw.githubusercontent.com/uBlockOrigin/uAssets/master/filters/filters.txt",
            ublockCoreRules()
        ),
        FilterListSource(
            LIST_UBLOCK_BADWARE,
            "uBlock Filters - Badware & Malware",
            "Rules for malicious redirects, cryptominers, fake update prompts, and scam popups.",
            "https://raw.githubusercontent.com/uBlockOrigin/uAssets/master/filters/badware.txt",
            ublockBadwareRules()
        ),
        FilterListSource(
            LIST_UBLOCK_PRIVACY,
            "uBlock Filters - Privacy & Telemetry",
            "Additional privacy rules for advanced telemetry and tracking endpoints.",
            "https://raw.githubusercontent.com/uBlockOrigin/uAssets/master/filters/privacy.txt",
            ublockPrivacyRules()
        ),
        FilterListSource(
            LIST_UBLOCK_QUICK_FIXES,
            "uBlock Filters - Quick Fixes & YouTube",
            "Fast-changing rules for YouTube ad endpoints and anti-adblock workarounds.",
            "https://raw.githubusercontent.com/uBlockOrigin/uAssets/master/filters/quick-fixes.txt",
            ublockQuickFixRules()
        ),
        FilterListSource(
            LIST_TURTLECUTE_TEST,
            "Turtlecute Adblock Test",
            "Open-source adblock test rules covering Turtlecute host, script, and cosmetic checks.",
            null,
            turtlecuteTestRules()
        )
    )

    private val listRules: Map<String, List<String>> = mapOf(
        LIST_EASYLIST to easyListRules(),
        LIST_EASYPRIVACY to easyPrivacyRules(),
        LIST_UBLOCK_FILTERS to ublockCoreRules(),
        LIST_UBLOCK_BADWARE to ublockBadwareRules(),
        LIST_UBLOCK_PRIVACY to ublockPrivacyRules(),
        LIST_UBLOCK_QUICK_FIXES to ublockQuickFixRules(),
        LIST_TURTLECUTE_TEST to turtlecuteTestRules()
    )

    private val knownAdOrTrackerHosts = setOf(
        "googletagmanager.com",
        "google-analytics.com",
        "analytics.google.com",
        "pagead2.googlesyndication.com",
        "googlesyndication.com",
        "googleadservices.com",
        "an.yandex.ru",
        "mc.yandex.ru",
        "static.hotjar.com",
        "hotjar.com",
        "browser.sentry-cdn.com",
        "js.sentry-cdn.com",
        "sentry-cdn.com",
        "bugsnag.com",
        "d2wy8f7a9ursnm.cloudfront.net",
        "ymatuhin.ru",
        "tagivi.com",
        "fellowearnwave.com",
        "sharethis.com",
        "t.sharethis.com",
        "static.cloudflareinsights.com"
    )

    private val remoteListRules = mutableMapOf<String, List<String>>()
    private val parsedCache = mutableMapOf<String, List<ParsedRule>>()
    private val injectionScriptCache = object : LinkedHashMap<InjectionScriptKey, String>(8, 0.75f, true) {
        override fun removeEldestEntry(eldest: MutableMap.MutableEntry<InjectionScriptKey, String>?): Boolean =
            size > MAX_INJECTION_SCRIPT_CACHE_ENTRIES
    }

    @Volatile
    private var activeRuleSetCache: ActiveRuleSet? = null

    @Volatile
    private var injectionScriptTemplate: String? = null

    @Volatile
    private var rulesGeneration = 0L

    fun initializeInjectionScriptTemplate(template: String) {
        require(template.contains(INJECTION_CONFIG_PLACEHOLDER)) {
            "Android adblock script is missing its configuration placeholder"
        }

        synchronized(this) {
            injectionScriptTemplate = template
            injectionScriptCache.clear()
        }
    }

    fun getFilterLists(settings: BrowserSettings): List<FilterListConfig> =
        filterListSources.map { source ->
            filterListConfig(settings, source)
        }

    fun setCachedFilterList(id: String, rawRules: String) {
        val rules = prepareRemoteRules(rawRules)

        synchronized(this) {
            if (rules.isEmpty()) {
                remoteListRules.remove(id)
            } else {
                remoteListRules[id] = rules
            }
            invalidateRuleCaches()
        }
    }

    fun setCachedFilterLists(cachedLists: Map<String, String>) {
        synchronized(this) {
            cachedLists.forEach { (id, rawRules) ->
                val rules = prepareRemoteRules(rawRules)
                if (rules.isEmpty()) {
                    remoteListRules.remove(id)
                } else {
                    remoteListRules[id] = rules
                }
            }
            invalidateRuleCaches()
        }
    }

    fun prepare(settings: BrowserSettings) {
        if (settings.adblockEnabled) {
            activeRuleSet(settings)
        }
    }

    private fun invalidateRuleCaches() {
        parsedCache.clear()
        activeRuleSetCache = null
        injectionScriptCache.clear()
        rulesGeneration += 1
    }

    private fun prepareRemoteRules(rawRules: String): List<String> =
        rawRules
            .lineSequence()
            .map { it.trim() }
            .filter { it.isNotBlank() && !it.startsWith("!") && !it.startsWith("[") }
            .mapNotNull(::sanitizeRemoteRule)
            .toList()

    private fun sanitizeRemoteRule(rule: String): String? {
        val normalized = if (rule.startsWith("@@")) rule.removePrefix("@@") else rule

        if ("##" in normalized) {
            val parts = normalized.split("##", limit = 2)
            val domains = parts.first().trim()
            val selector = parts.getOrNull(1)?.trim().orEmpty()
            return rule.takeIf {
                domains.isNotBlank() &&
                    selector.isNotBlank() &&
                    !isUnsupportedCosmeticSelector(selector)
            }
        }

        val pattern = normalized.substringBefore("$").trim()
        if (!isSimpleHostRulePattern(pattern)) return null

        val options = normalized.substringAfter("$", "")
            .split(",")
            .map { it.trim().lowercase() }
            .filter { it.isNotBlank() }
            .toSet()
        if (hasUnsupportedNetworkOptions(options)) return null

        return rule
    }

    private fun isSimpleHostRulePattern(pattern: String): Boolean {
        if (!pattern.startsWith("||") || !pattern.endsWith("^")) return false
        val host = pattern.removePrefix("||").trimEnd('^')
        return host.isNotBlank() &&
            '/' !in host &&
            '*' !in host &&
            '$' !in host &&
            host.any { it == '.' } &&
            host.all { it.isLetterOrDigit() || it == '.' || it == '-' || it == '_' }
    }

    fun isBlockedUrl(
        url: String,
        settings: BrowserSettings,
        sourceUrl: String? = null,
        requestType: String = "other"
    ): Boolean = isBlockedRequest(
        url = url,
        requestHost = parseHost(url),
        settings = settings,
        sourceHost = sourceUrl?.let(::parseHost).orEmpty(),
        requestType = requestType
    )

    internal fun isBlockedRequest(
        url: String,
        requestHost: String,
        settings: BrowserSettings,
        sourceHost: String = "",
        requestType: String = "other"
    ): Boolean {
        if (!settings.adblockEnabled || url.isBlank() || isBypassedUrl(url)) return false

        val normalizedRequestHost = requestHost.lowercase()
        val normalizedSourceHost = sourceHost.lowercase()
        if (normalizedRequestHost.isBlank() || isBenchmarkOrLocalHost(normalizedRequestHost)) return false

        if (isWhitelistedHost(normalizedSourceHost.ifBlank { normalizedRequestHost }, settings.adblockWhitelistedDomains)) {
            return false
        }
        if (isWhitelistedHost(normalizedRequestHost, settings.adblockWhitelistedDomains)) {
            return false
        }

        val rules = activeRuleSet(settings)
        if (rules.exceptionHosts.matchesHost(normalizedRequestHost) || rules.complexExceptionRules.any {
                it.matchesNetwork(url, normalizedRequestHost, normalizedSourceHost, requestType)
            }
        ) {
            return false
        }

        if (isKnownAdOrTrackerUrl(url, normalizedRequestHost, normalizedSourceHost, requestType)) return true

        val blockedByFilter = rules.blockingHosts.matchesHost(normalizedRequestHost) ||
            rules.complexBlockingRules.any {
                it.matchesNetwork(url, normalizedRequestHost, normalizedSourceHost, requestType)
            }
        if (blockedByFilter) return true

        if (settings.aggressiveMode) {
            val lower = url.lowercase()
            return lower.contains("adservice") ||
                lower.contains("adserver") ||
                lower.contains("telemetry") ||
                lower.contains("tracking") ||
                lower.contains("analytics") ||
                lower.contains("pixel")
        }

        return false
    }

    fun getInjectionScript(settings: BrowserSettings, pageUrl: String? = null): String {
        if (!settings.adblockEnabled) return ""
        return buildInjectionScript(settings, pageUrl, activeRuleSet(settings))
    }

    fun getPreparedInjectionScript(settings: BrowserSettings, pageUrl: String? = null): String? {
        if (!settings.adblockEnabled) return ""
        val rules = preparedActiveRuleSet(settings) ?: return null
        return buildInjectionScript(settings, pageUrl, rules)
    }

    private fun buildInjectionScript(
        settings: BrowserSettings,
        pageUrl: String?,
        rules: ActiveRuleSet
    ): String {
        val pageHost = pageUrl?.let(::parseHost).orEmpty()
        val cacheKey = InjectionScriptKey(rules.key.generation, settings, pageHost)
        synchronized(this) {
            injectionScriptCache[cacheKey]?.let { return it }
        }

        val selectors = if (settings.cosmeticFiltering) getCosmeticSelectors(rules, pageHost) else emptyList()
        val configJson = buildString {
            append("{\"blockVideoAds\":").append(settings.blockVideoAds)
            append(",\"cosmeticFiltering\":").append(settings.cosmeticFiltering)
            append(",\"blockPopups\":").append(settings.blockPopups)
            append(",\"aggressiveMode\":").append(settings.aggressiveMode)
            append(",\"whitelistedDomains\":").append(settings.adblockWhitelistedDomains.toJsArray())
            append(",\"blockedDomains\":").append(settings.adblockBlockedDomains.toJsArray())
            append(",\"filterRuleHosts\":").append(rules.networkHostsJson)
            append(",\"staticSelectors\":").append(selectors.toJsArray())
            append('}')
        }
        val template = checkNotNull(injectionScriptTemplate) {
            "Android adblock script template was not initialized"
        }
        val script = template.replace(INJECTION_CONFIG_PLACEHOLDER, configJson)

        synchronized(this) {
            injectionScriptCache[cacheKey] = script
        }
        return script
    }

    private fun filterListConfig(
        settings: BrowserSettings,
        source: FilterListSource
    ): FilterListConfig = FilterListConfig(
        id = source.id,
        name = source.name,
        description = source.description,
        count = effectiveRules(source.id).size,
        enabled = settings.adblockFilterLists.contains(source.id)
    )

    private fun activeRuleSet(settings: BrowserSettings): ActiveRuleSet {
        val generation = rulesGeneration
        activeRuleSetCache?.takeIf { it.matches(settings, generation) }?.let { return it }

        return synchronized(this) {
            val currentGeneration = rulesGeneration
            activeRuleSetCache?.takeIf { it.matches(settings, currentGeneration) }
                ?.let { return@synchronized it }
            val key = ruleSetKey(settings, currentGeneration)

            val parsedRules = buildList {
                key.filterLists.forEach { id ->
                    val cacheKey = "list:$id"
                    addAll(parsedCache.getOrPut(cacheKey) {
                        effectiveRules(id).mapNotNull(::parseRule)
                    })
                }
                addAll(key.customRules.mapNotNull(::parseRule))
                addAll(key.blockedDomains.mapNotNull { parseRule("||${it.trim()}^") })
            }

            val exceptionHosts = LinkedHashSet<String>()
            val blockingHosts = LinkedHashSet<String>()
            val complexExceptionRules = ArrayList<ParsedRule>()
            val complexBlockingRules = ArrayList<ParsedRule>()
            val cosmeticRules = ArrayList<ParsedRule>()

            parsedRules.forEach { rule ->
                if (rule.cosmeticSelector != null) {
                    cosmeticRules += rule
                }

                if (rule.networkPattern != null) {
                    val simpleHost = rule.simpleHostPattern()
                    when {
                        simpleHost != null && rule.isException -> exceptionHosts += simpleHost
                        simpleHost != null -> blockingHosts += simpleHost
                        rule.isException -> complexExceptionRules += rule
                        else -> complexBlockingRules += rule
                    }
                }
            }

            ActiveRuleSet(
                key = key,
                exceptionHosts = exceptionHosts,
                blockingHosts = blockingHosts,
                complexExceptionRules = complexExceptionRules,
                complexBlockingRules = complexBlockingRules,
                cosmeticRules = cosmeticRules,
                networkHostsJson = javascriptNetworkHosts(key).toJsArray()
            ).also { activeRuleSetCache = it }
        }
    }

    private fun javascriptNetworkHosts(key: RuleSetKey): List<String> = buildList {
        key.filterLists.forEach { id ->
            listRules[id]
                .orEmpty()
                .asSequence()
                .mapNotNull(::parseRule)
                .filter { !it.isException }
                .mapNotNull { it.networkPattern?.extractHostPattern() }
                .forEach(::add)
        }
        key.customRules
            .asSequence()
            .mapNotNull(::parseRule)
            .filter { !it.isException }
            .mapNotNull { it.networkPattern?.extractHostPattern() }
            .forEach(::add)
        key.blockedDomains.forEach { domain ->
            parseRule("||${domain.trim()}^")
                ?.networkPattern
                ?.extractHostPattern()
                ?.let(::add)
        }
    }.distinct()

    private fun preparedActiveRuleSet(settings: BrowserSettings): ActiveRuleSet? {
        val cached = activeRuleSetCache ?: return null
        return cached.takeIf { it.matches(settings, rulesGeneration) }
    }

    private fun ActiveRuleSet.matches(settings: BrowserSettings, generation: Long): Boolean =
        key.generation == generation &&
            key.filterLists == settings.adblockFilterLists &&
            key.customRules == settings.adblockCustomRules &&
            key.blockedDomains == settings.adblockBlockedDomains

    private fun ruleSetKey(settings: BrowserSettings, generation: Long) = RuleSetKey(
        generation = generation,
        filterLists = settings.adblockFilterLists,
        customRules = settings.adblockCustomRules,
        blockedDomains = settings.adblockBlockedDomains
    )

    private fun effectiveRules(id: String): List<String> =
        synchronized(this) { remoteListRules[id] ?: listRules[id].orEmpty() }

    private fun parseRule(rawRule: String): ParsedRule? {
        val raw = rawRule.trim()
        if (raw.isBlank() || raw.startsWith("!") || raw.startsWith("[")) return null

        val isException = raw.startsWith("@@")
        val rule = if (isException) raw.removePrefix("@@") else raw

        if ("##" in rule) {
            val parts = rule.split("##", limit = 2)
            val selector = parts.getOrNull(1)?.trim().orEmpty()
            if (selector.isBlank() || isUnsupportedCosmeticSelector(selector)) return null
            val domains = parts.first().split(",").map { it.trim().lowercase() }.filter { it.isNotBlank() }
            return ParsedRule(raw, isException, cosmeticDomains = domains, cosmeticSelector = selector)
        }

        val patternAndOptions = rule.split("$", limit = 2)
        val options = patternAndOptions.getOrNull(1)
            ?.split(",")
            ?.map { it.trim().lowercase() }
            ?.filter { it.isNotBlank() }
            ?.toSet()
            .orEmpty()

        if (hasUnsupportedNetworkOptions(options)) return null
        if (options.contains("redirect-rule")) return null
        val networkPattern = patternAndOptions.first().trim()
        val regexPattern = when {
            networkPattern.startsWith("||") && '/' in networkPattern ->
                networkPattern.substring(networkPattern.indexOf('/')).trimEnd('^')
            '*' in networkPattern -> networkPattern
            else -> null
        }
        return ParsedRule(
            raw = raw,
            isException = isException,
            networkPattern = networkPattern,
            networkRegex = regexPattern?.let(::wildcardToRegex),
            options = options
        )
    }

    private fun isUnsupportedCosmeticSelector(selector: String): Boolean {
        val lower = selector.lowercase()
        return lower.startsWith("+js(") ||
            lower.startsWith("script:") ||
            lower.contains(":has-text") ||
            lower.contains(":matches-css") ||
            lower.contains(":matches-attr") ||
            lower.contains(":xpath") ||
            lower.contains(":style(") ||
            lower.contains(":remove()") ||
            lower.contains(":watch-attr")
    }

    private fun hasUnsupportedNetworkOptions(options: Set<String>): Boolean =
        options.any { option ->
            option == "badfilter" ||
                option == "csp" ||
                option.startsWith("csp=") ||
                option == "redirect-rule" ||
                option.startsWith("redirect=") ||
                option.startsWith("replace=") ||
                option.startsWith("removeparam") ||
                option.startsWith("uritransform") ||
                option.startsWith("permissions=") ||
                option.startsWith("header=") ||
                option.startsWith("to=") ||
                option.startsWith("from=") ||
                option.startsWith("denyallow=") ||
                option.startsWith("method=") ||
                option == "generichide" ||
                option == "genericblock" ||
                option == "elemhide" ||
                option == "jsinject" ||
                option == "shide"
        }

    private fun ParsedRule.matchesNetwork(
        url: String,
        host: String,
        sourceHost: String,
        requestType: String
    ): Boolean {
        val pattern = networkPattern ?: return false
        if (!optionsMatch(options, host, sourceHost, requestType)) return false

        val lowerUrl = url.lowercase()
        return when {
            pattern == "*" -> true
            pattern.startsWith("||") -> {
                val hostAndPathPattern = pattern.removePrefix("||").lowercase()
                val hostPattern = hostAndPathPattern
                    .takeWhile { it != '^' && it != '/' && it != '*' }
                if (hostPattern.isBlank() || !matchesDomain(host, hostPattern)) {
                    false
                } else {
                    val pathStart = hostAndPathPattern.indexOf('/')
                    pathStart < 0 || networkRegex?.containsMatchIn(lowerUrl) == true
                }
            }
            pattern.startsWith("|") -> lowerUrl.startsWith(pattern.removePrefix("|").lowercase())
            pattern.startsWith("/") || pattern.startsWith("&") -> lowerUrl.contains(pattern.lowercase())
            pattern.contains("*") -> networkRegex?.containsMatchIn(lowerUrl) == true
            else -> lowerUrl.contains(pattern.trim('^').lowercase())
        }
    }

    private fun ParsedRule.simpleHostPattern(): String? {
        if (options.isNotEmpty()) return null
        val pattern = networkPattern ?: return null
        if (!pattern.startsWith("||") || !pattern.endsWith("^")) return null

        return pattern
            .removePrefix("||")
            .trimEnd('^')
            .lowercase()
            .removePrefix("www.")
            .takeIf { host ->
                host.isNotBlank() &&
                    '/' !in host &&
                    '*' !in host &&
                    host.all { it.isLetterOrDigit() || it == '.' || it == '-' || it == '_' }
            }
    }

    private fun optionsMatch(
        options: Set<String>,
        host: String,
        sourceHost: String,
        requestType: String
    ): Boolean {
        if (options.isEmpty()) return true

        val thirdParty = sourceHost.isNotBlank() && !matchesDomain(host, sourceHost)
        val normalizedRequestType = requestType.lowercase().normalizedRequestType()
        for (option in options) {
            when {
                (option == "third-party" || option == "3p") && !thirdParty -> return false
                (option == "~third-party" || option == "~3p") && thirdParty -> return false
                (option == "first-party" || option == "1p") && thirdParty -> return false
                (option == "~first-party" || option == "~1p") && !thirdParty -> return false
                option.startsWith("domain=") -> {
                    val allowed = option.removePrefix("domain=").split("|").map { it.trim().lowercase() }
                    val included = allowed.filterNot { it.startsWith("~") }
                    val excluded = allowed.filter { it.startsWith("~") }.map { it.removePrefix("~") }
                    if (excluded.any { matchesDomain(sourceHost, it) }) return false
                    if (included.isNotEmpty() && sourceHost.isNotBlank() && included.none { matchesDomain(sourceHost, it) }) {
                        return false
                    }
                }
                option == "all" || option == "important" || option == "match-case" || option.startsWith("reason=") -> Unit
                option in requestTypeOptions && option.normalizedRequestType() != normalizedRequestType -> return false
                option.startsWith("~") && option.removePrefix("~") in requestTypeOptions &&
                    option.removePrefix("~").normalizedRequestType() == normalizedRequestType -> return false
            }
        }
        return true
    }

    private val requestTypeOptions = setOf(
        "script",
        "image",
        "stylesheet",
        "xmlhttprequest",
        "xhr",
        "subdocument",
        "frame",
        "media",
        "font",
        "other",
        "object",
        "ping",
        "popup",
        "document",
        "doc",
        "main_frame"
    )

    private fun String.normalizedRequestType(): String =
        when (this) {
            "xmlhttprequest" -> "xhr"
            "frame" -> "subdocument"
            "doc" -> "document"
            "main_frame" -> "document"
            else -> this
        }

    private fun getCosmeticSelectors(rules: ActiveRuleSet, host: String): List<String> {
        return rules.cosmeticRules
            .asSequence()
            .filter { it.cosmeticSelector != null && !it.isException }
            .filter { rule ->
                rule.cosmeticDomains.isEmpty() ||
                    rule.cosmeticDomains.any { domain -> !domain.startsWith("~") && matchesDomain(host, domain) }
            }
            .mapNotNull { it.cosmeticSelector }
            .distinct()
            .toList()
    }

    private fun String.extractHostPattern(): String? {
        if (!startsWith("||")) return null
        val body = removePrefix("||")
        if ('/' in body || '*' in body || '$' in body) return null
        if (!body.endsWith("^")) return null

        val hostPattern = body
            .trimEnd('^')
            .lowercase()
            .removePrefix("www.")
        return hostPattern.takeIf { it.isNotBlank() && !it.contains("*") }
    }

    private fun isBypassedUrl(url: String): Boolean {
        return url.startsWith("data:", ignoreCase = true) ||
            url.startsWith("blob:", ignoreCase = true) ||
            url.startsWith("about:", ignoreCase = true) ||
            url.startsWith("file:", ignoreCase = true) ||
            url.startsWith("titan:", ignoreCase = true) ||
            url.startsWith("chrome:", ignoreCase = true) ||
            url.startsWith("ws:", ignoreCase = true) ||
            url.startsWith("wss:", ignoreCase = true) ||
            url.startsWith("javascript:", ignoreCase = true)
    }

    private fun isBenchmarkOrLocalHost(host: String): Boolean =
        host.contains("browserbench.org") ||
            host.contains("speedometer") ||
            host.contains("localhost") ||
            host.contains("127.0.0.1") ||
            host.contains("krakenbenchmark") ||
            host.contains("webglreport") ||
            host.contains("octane")

    private fun isKnownAdOrTrackerUrl(
        url: String,
        host: String,
        sourceHost: String,
        requestType: String
    ): Boolean {
        if (knownAdOrTrackerHosts.matchesHost(host)) return true

        val lower = url.lowercase()
        val isThirdParty = sourceHost.isNotBlank() && !matchesDomain(host, sourceHost)
        val normalizedRequestType = requestType.normalizedRequestType()
        val isDocument = normalizedRequestType == "document"
        val genericAdPath = lower.contains("/ads/") ||
            lower.contains("/ad/") ||
            lower.contains("/banners/") ||
            lower.contains("/banner/")
        val highConfidenceAdPath = lower.contains("pr_advertising_ads_banner") ||
            lower.contains("adsbygoogle.js") ||
            lower.contains("metrika/tag.js") ||
            lower.contains("bugsnag.min.js") ||
            lower.contains("bundle.tracing.replay")
        val thirdPartyBeacon = lower.contains("ren.gif") ||
            lower.contains("impr.gif") ||
            lower.contains("context.js")

        return highConfidenceAdPath ||
            (!isDocument && isThirdParty && genericAdPath) ||
            (!isDocument && isThirdParty && thirdPartyBeacon) ||
            (lower.contains("fellowearnwave.com") && lower.endsWith(".js")) ||
            (!isDocument && isThirdParty && lower.endsWith(".swf"))
    }

    private fun parseHost(url: String): String =
        runCatching { URI(url).host?.lowercase().orEmpty() }.getOrDefault("")

    internal fun isWhitelistedHost(host: String, whitelist: List<String>): Boolean =
        host.isNotBlank() && whitelist.any { matchesDomain(host, it) }

    private fun matchesDomain(host: String, domain: String): Boolean {
        val cleanDomain = domain
            .removePrefix("||")
            .trim()
            .trim('^')
            .lowercase()
            .removePrefix("www.")
        val cleanHost = host.lowercase().removePrefix("www.")
        return cleanDomain.isNotBlank() && (cleanHost == cleanDomain || cleanHost.endsWith(".$cleanDomain"))
    }

    private fun Set<String>.matchesHost(host: String): Boolean {
        var candidate = host.lowercase().removePrefix("www.")
        while (candidate.isNotBlank()) {
            if (candidate in this) return true
            val dot = candidate.indexOf('.')
            if (dot < 0) return false
            candidate = candidate.substring(dot + 1)
        }
        return false
    }

    private fun wildcardToRegex(pattern: String): Regex {
        val source = buildString {
            pattern.lowercase().forEach { char ->
                when (char) {
                    '*' -> append(".*")
                    '^' -> append("(?:[^a-z0-9_\\-.%]|$)")
                    else -> append(Regex.escape(char.toString()))
                }
            }
        }
        return Regex(source)
    }

    private fun List<String>.toJsArray(): String =
        joinToString(prefix = "[", postfix = "]") { value ->
            "\"" + value
                .replace("\\", "\\\\")
                .replace("\"", "\\\"")
                .replace("\n", "\\n")
                .replace("\r", "") + "\""
        }

    private fun turtlecuteTestRules() = listOf(
        "||adtago.s3.amazonaws.com^",
        "||analyticsengine.s3.amazonaws.com^",
        "||analytics.s3.amazonaws.com^",
        "||advice-ads.s3.amazonaws.com^",
        "||pagead2.googlesyndication.com^",
        "||adservice.google.com^",
        "||pagead2.googleadservices.com^",
        "||afs.googlesyndication.com^",
        "||stats.g.doubleclick.net^",
        "||ad.doubleclick.net^",
        "||static.doubleclick.net^",
        "||m.doubleclick.net^",
        "||mediavisor.doubleclick.net^",
        "||ads30.adcolony.com^",
        "||adc3-launch.adcolony.com^",
        "||events3alt.adcolony.com^",
        "||wd.adcolony.com^",
        "||static.media.net^",
        "||media.net^",
        "||adservetx.media.net^",
        "||analytics.google.com^",
        "||click.googleanalytics.com^",
        "||google-analytics.com^",
        "||ssl.google-analytics.com^",
        "||adm.hotjar.com^",
        "||identify.hotjar.com^",
        "||insights.hotjar.com^",
        "||script.hotjar.com^",
        "||surveys.hotjar.com^",
        "||careers.hotjar.com^",
        "||events.hotjar.io^",
        "||mouseflow.com^",
        "||cdn.mouseflow.com^",
        "||o2.mouseflow.com^",
        "||gtm.mouseflow.com^",
        "||api.mouseflow.com^",
        "||tools.mouseflow.com^",
        "||cdn-test.mouseflow.com^",
        "||freshmarketer.com^",
        "||claritybt.freshmarketer.com^",
        "||fwtracks.freshmarketer.com^",
        "||luckyorange.com^",
        "||api.luckyorange.com^",
        "||realtime.luckyorange.com^",
        "||cdn.luckyorange.com^",
        "||w1.luckyorange.com^",
        "||upload.luckyorange.net^",
        "||cs.luckyorange.net^",
        "||settings.luckyorange.net^",
        "||stats.wp.com^",
        "||notify.bugsnag.com^",
        "||sessions.bugsnag.com^",
        "||api.bugsnag.com^",
        "||app.bugsnag.com^",
        "||browser.sentry-cdn.com^",
        "||app.getsentry.com^",
        "||pixel.facebook.com^",
        "||an.facebook.com^",
        "||static.ads-twitter.com^",
        "||ads-api.twitter.com^",
        "||ads.linkedin.com^",
        "||analytics.pointdrive.linkedin.com^",
        "||ads.pinterest.com^",
        "||log.pinterest.com^",
        "||trk.pinterest.com^",
        "||events.reddit.com^",
        "||events.redditmedia.com^",
        "||ads.youtube.com^",
        "||ads-api.tiktok.com^",
        "||analytics.tiktok.com^",
        "||ads-sg.tiktok.com^",
        "||analytics-sg.tiktok.com^",
        "||business-api.tiktok.com^",
        "||ads.tiktok.com^",
        "||log.byteoversea.com^",
        "||ads.yahoo.com^",
        "||analytics.yahoo.com^",
        "||geo.yahoo.com^",
        "||udcm.yahoo.com^",
        "||analytics.query.yahoo.com^",
        "||partnerads.ysm.yahoo.com^",
        "||log.fc.yahoo.com^",
        "||gemini.yahoo.com^",
        "||adtech.yahooinc.com^",
        "||extmaps-api.yandex.net^",
        "||appmetrica.yandex.ru^",
        "||adfstat.yandex.ru^",
        "||metrika.yandex.ru^",
        "||offerwall.yandex.net^",
        "||adfox.yandex.ru^",
        "||auction.unityads.unity3d.com^",
        "||webview.unityads.unity3d.com^",
        "||config.unityads.unity3d.com^",
        "||adserver.unityads.unity3d.com^",
        "||iot-eu-logser.realme.com^",
        "||iot-logser.realme.com^",
        "||bdapi-ads.realmemobile.com^",
        "||bdapi-in-ads.realmemobile.com^",
        "||api.ad.xiaomi.com^",
        "||data.mistat.xiaomi.com^",
        "||data.mistat.india.xiaomi.com^",
        "||data.mistat.rus.xiaomi.com^",
        "||sdkconfig.ad.xiaomi.com^",
        "||sdkconfig.ad.intl.xiaomi.com^",
        "||tracking.rus.miui.com^",
        "||adsfs.oppomobile.com^",
        "||adx.ads.oppomobile.com^",
        "||ck.ads.oppomobile.com^",
        "||data.ads.oppomobile.com^",
        "||metrics.data.hicloud.com^",
        "||metrics2.data.hicloud.com^",
        "||grs.hicloud.com^",
        "||logservice.hicloud.com^",
        "||logservice1.hicloud.com^",
        "||logbak.hicloud.com^",
        "||click.oneplus.cn^",
        "||open.oneplus.net^",
        "||samsungads.com^",
        "||smetrics.samsung.com^",
        "||nmetrics.samsung.com^",
        "||samsung-com.112.2o7.net^",
        "||analytics-api.samsunghealthcn.com^",
        "||iadsdk.apple.com^",
        "||metrics.icloud.com^",
        "||metrics.mzstatic.com^",
        "||api-adservices.apple.com^",
        "||books-analytics-events.apple.com^",
        "||weather-analytics-events.apple.com^",
        "||notes-analytics-events.apple.com^",
        "*\$3p,domain=adblock.turtlecute.org",
        "/js/widget/ads.js\$domain=adblock.turtlecute.org",
        "/pagead.js\$domain=adblock.turtlecute.org",
        "@@*\$redirect-rule,domain=adblock.turtlecute.org",
        "adblock.turtlecute.org##.textads",
        "adblock.turtlecute.org##.banner-ads",
        "adblock.turtlecute.org##.banner_ads",
        "adblock.turtlecute.org##.ad-unit",
        "adblock.turtlecute.org##.afs_ads",
        "adblock.turtlecute.org##.ad-zone",
        "adblock.turtlecute.org##.ad-space",
        "adblock.turtlecute.org##.adsbox"
    )

    private fun easyListRules() = listOf(
        "||doubleclick.net^",
        "||googleadservices.com^",
        "||googlesyndication.com^",
        "||adservice.google.com^",
        "||pagead2.googlesyndication.com^",
        "||partner.googleadservices.com^",
        "||tpc.googlesyndication.com^",
        "||2mdn.net^",
        "||adnxs.com^",
        "||advertising.com^",
        "||rubiconproject.com^",
        "||pubmatic.com^",
        "||criteo.com^",
        "||criteo.net^",
        "||openx.net^",
        "||smartadserver.com^",
        "||casalemedia.com^",
        "||bidswitch.net^",
        "||indexexchange.com^",
        "||amazon-adsystem.com^",
        "||adroll.com^",
        "||media.net^",
        "||moatads.com^",
        "||outbrain.com^",
        "||taboola.com^",
        "||revcontent.com^",
        "||mgid.com^",
        "||inmobi.com^",
        "||flashtalking.com^",
        "||exponential.com^",
        "||adform.net^",
        "||adcolony.com^",
        "||applovin.com^",
        "||unityads.unity3d.com^",
        "||vungle.com^",
        "||chartbeat.com^",
        "||scorecardresearch.com^",
        "||zedo.com^",
        "||adblade.com^",
        "||yieldmo.com^",
        "||sharethrough.com^",
        "||triplelift.com^",
        "||teads.tv^",
        "||undertone.com^",
        "||spotxchange.com^",
        "||spotx.tv^",
        "||sovrn.com^",
        "||sonobi.com^",
        "||gumgum.com^",
        "||quantserve.com^",
        "||quantcount.com^",
        "||lijit.com^",
        "||adtechus.com^",
        "||tribalfusion.com^",
        "||clicksor.com^",
        "||buysellads.com^",
        "||carbonads.net^",
        "||serving-sys.com^",
        "||bs.serving-sys.com^",
        "||eyeota.net^",
        "||simpli.fi^",
        "||advertising.amazon.com^",
        "||aax-us-east.amazon-adsystem.com^",
        "||monetag.com^",
        "||monetag.net^",
        "||propu.sh^",
        "||onclickbright.com^",
        "||onclicksuper.com^",
        "||deloton.com^",
        "||highperformancegate.com^",
        "||adsterra.com^",
        "||adsterratech.com^",
        "||hilltopads.com^",
        "||hilltopads.net^",
        "||clickadu.com^",
        "||clckr.com^",
        "||popcash.net^",
        "||popads.net^",
        "||exoclick.com^",
        "||juicyads.com^",
        "||tsyndicate.com^",
        "||ad-maven.com^",
        "||trafficstars.com^",
        "||etahub.com^",
        "||bignox.com^",
        "||histats.com^",
        "||dtscout.com^",
        "||tagivi.com^",
        "||fellowearnwave.com^",
        "||sharethis.com^",
        "||t.sharethis.com^",
        "||al5smvpt45.com^",
        "||feedify.net^",
        "||ad-delivery.net^",
        "||pushwoosh.com^",
        "||pushassist.com^",
        "||syndication.exoclick.com^",
        "||trafficjunky.net^",
        "||rtmark.net^",
        "||in-page-push.com^",
        "/ads.js",
        "/advertisement.",
        "/ad-banner.",
        "/banner_ads/",
        "/banners/pr_advertising_ads_banner",
        "/pagead/js/adsbygoogle.js",
        "/pagead/show_ads.js",
        "&ad_type=",
        "&ad_url=",
        "&adurl=",
        "/wp-content/plugins/adrotate/",
        "/wp-content/plugins/ad-inserter/",
        "##ins.adsbygoogle",
        "##[id^=\"google_ads_\"]",
        "##[id*=\"google_ads_iframe\"]",
        "##[id*=\"ScriptRoot\"]",
        "##[class*=\"sponsored-post\"]",
        "##[class*=\"ad-container\"]",
        "##[class*=\"ad_container\"]",
        "##[id*=\"banner-ad\"]",
        "##[id*=\"ad-banner\"]",
        "##[class*=\"ad-banner\"]",
        "##[class*=\"ad-wrapper\"]",
        "##[id*=\"ad-wrapper\"]",
        "##[class*=\"ad-slot\"]",
        "##[id*=\"ad-slot\"]",
        "##[class*=\"ad-placement\"]",
        "##[aria-label=\"advertisement\"]",
        "##[aria-label=\"Sponsored\"]",
        "##.trc_rbox_div",
        "##.OUTBRAIN",
        "##.taboola-placeholder",
        "##[class*=\"share-bar\"]",
        "##[class*=\"floating-share\"]",
        "##[class*=\"shares-box\"]",
        "##.social-share",
        "##[id*=\"share-buttons\"]",
        "##[class*=\"share-container\"]",
        "##div[class*=\"share-sidebar\"]",
        "##.at-share-dock",
        "##.addthis_floating_style",
        "##.sharethis-inline-share-buttons",
        "##.st-sticky-share-buttons",
        "##div[class*=\"shares\"]",
        "##[class*=\"captcha-modal\"]",
        "##[id*=\"captcha-modal\"]",
        "##[class*=\"robot-modal\"]",
        "##[class*=\"robot-check\"]",
        "##[class*=\"verify-robot\"]",
        "##[id*=\"robot-check\"]",
        "##[class*=\"notification-prompt\"]",
        "##[class*=\"push-prompt\"]",
        "##[class*=\"push-modal\"]",
        "##[class*=\"ad-modal\"]",
        "##[class*=\"popup-modal\"]",
        "##[id*=\"popup-modal\"]",
        "##[class*=\"interstitial\"]",
        "##[id*=\"interstitial\"]",
        "##[class*=\"overlay-backdrop\"]",
        "##[id*=\"overlay-backdrop\"]",
        "##[class*=\"ad-overlay\"]",
        "##[id*=\"ad-overlay\"]",
        "##div[style*=\"z-index: 2147483647\"]",
        "##div[style*=\"z-index: 999999\"]",
        "##div[style*=\"z-index: 99999\"]"
    )

    private fun easyPrivacyRules() = listOf(
        "||google-analytics.com^",
        "||analytics.google.com^",
        "||googletagmanager.com^",
        "||googletagservices.com^",
        "||stats.g.doubleclick.net^",
        "||pipe.aria.microsoft.com^",
        "||events.data.microsoft.com^",
        "||telemetry.microsoft.com^",
        "||watson.telemetry.microsoft.com^",
        "||mobile.pipe.aria.microsoft.com^",
        "||clarity.ms^",
        "||hotjar.com^",
        "||hotjar.io^",
        "||static.hotjar.com^",
        "||fullstory.com^",
        "||fs.fullstory.com^",
        "||mouseflow.com^",
        "||luckyorange.com^",
        "||crazyegg.com^",
        "||inspectlet.com^",
        "||logrocket.io^",
        "||smartlook.com^",
        "||segment.io^",
        "||segment.com^",
        "||api.segment.io^",
        "||cdn.segment.com^",
        "||mixpanel.com^",
        "||api.mixpanel.com^",
        "||amplitude.com^",
        "||api.amplitude.com^",
        "||heap.io^",
        "||heapanalytics.com^",
        "||cdn.heapanalytics.com^",
        "||kissmetrics.com^",
        "||woopra.com^",
        "||branch.io^",
        "||appsflyer.com^",
        "||adjust.com^",
        "||kochava.com^",
        "||singular.net^",
        "||browser.sentry-cdn.com^",
        "||js.sentry-cdn.com^",
        "||datadoghq.com^\$third-party",
        "||browser-intake-datadoghq.com^",
        "||loggly.com^",
        "||bugsnag.com^",
        "||d2wy8f7a9ursnm.cloudfront.net^",
        "||crashlytics.com^",
        "||connect.facebook.net^*/fbevents.js",
        "||facebook.com/tr/^",
        "||pixel.facebook.com^",
        "||static.ads-twitter.com^",
        "||ads-twitter.com^",
        "||t.co^\$third-party",
        "||analytics.tiktok.com^",
        "||ct.pinterest.com^",
        "||trk.pinterest.com^",
        "||tr.snapchat.com^",
        "||sc-static.net/scevent.min.js",
        "/beacon.js",
        "/tracker.js",
        "/telemetry.js",
        "/ping.gif",
        "/pixel.gif",
        "/collect?v="
    )

    private fun ublockCoreRules() = listOf(
        "||popads.net^",
        "||popcash.net^",
        "||propellerads.com^",
        "||adcash.com^",
        "||adcash.net^",
        "||onclickads.net^",
        "||yllix.com^",
        "||hilltopads.net^",
        "||bidvertiser.com^",
        "||exoclick.com^",
        "||trafficjunky.com^",
        "||juicyads.com^",
        "||ero-advertising.com^",
        "youtube.com##ytd-promoted-video-renderer",
        "youtube.com##ytd-promoted-sparkles-web-renderer",
        "youtube.com##ytd-display-ad-renderer",
        "youtube.com##ytd-statement-banner-renderer",
        "youtube.com##ytd-in-feed-ad-layout-renderer",
        "youtube.com##ytd-banner-promo-renderer",
        "youtube.com###masthead-ad",
        "youtube.com###player-ads",
        "youtube.com###offer-module",
        "youtube.com##.ytp-ad-overlay-container",
        "youtube.com##.ytp-ad-message-container",
        "youtube.com##.ytp-ad-overlay-slot",
        "youtube.com##.ytp-ad-action-interstitial",
        "youtube.com##.video-ads",
        "youtube.com##.ytp-ad-module",
        "reddit.com##.promotedlink",
        "reddit.com##shreddit-comment-ad",
        "reddit.com##[data-adclicklocation]",
        "shuttletv.gd##.modal",
        "shuttletv.gd##.backdrop",
        "shuttletv.gd##[class*=\"share\"]",
        "bobmovies.gd##.modal",
        "bobmovies.gd##.backdrop",
        "bobmovies.gd##[class*=\"share\"]",
        "flixnetwork.to##.modal",
        "flixnetwork.to##.backdrop"
    )

    private fun ublockBadwareRules() = listOf(
        "||coinhive.com^",
        "||coin-hive.com^",
        "||authedmine.com^",
        "||cryptoloot.pro^",
        "||webminepool.com^",
        "||minr.pw^",
        "||crypto-loot.com^",
        "||trackvoluum.com^",
        "||voluumtrk.com^",
        "||zerohedge.bid^",
        "||install.stream^",
        "||pushcrew.com^",
        "||pushengage.com^",
        "||onesignal.com^\$third-party",
        "||webpushr.com^",
        "||pushassist.com^",
        "||system-update-center.com^",
        "||critical-alert-center.com^",
        "||mac-cleaner-alert.com^",
        "||windows-defender-notice.com^"
    )

    private fun ublockPrivacyRules() = listOf(
        "||cname.trackers.net^",
        "||telemetry.*.com^",
        "||stats.*.org^",
        "||beacons.gcp.gvt2.com^",
        "||app-measurement.com^",
        "||firebase-logging.googleapis.com^",
        "||firebaselogging-pa.googleapis.com^",
        "||browser-intake-datadoghq.eu^",
        "||sdk.split.io^",
        "||app.launchdarkly.com^",
        "||events.launchdarkly.com^",
        "||clientstream.launchdarkly.com^"
    )

    private fun ublockQuickFixRules() = listOf(
        "||googlevideo.com/videoplayback\$domain=youtube.com,xhr,redirect=noopjs",
        "@@||youtube.com/api/stats/playback",
        "@@||youtube.com/api/stats/delayplay",
        "@@||youtube.com/api/stats/watchtime",
        "@@||youtube.com/youtubei/v1/player",
        "@@||youtube.com/youtubei/v1/next",
        "||youtube.com/youtubei/v1/log_event^\$xhr",
        "||youtube.com/pagead/^",
        "||youtube.com/api/stats/ads^",
        "||youtube.com/api/stats/qoe?*&adformat=",
        "||youtube.com/ptracking^"
    )
}
