package com.titan.browser.viewmodel

import android.app.Application
import android.content.ComponentCallbacks2
import android.graphics.Bitmap
import android.view.View
import android.webkit.CookieManager
import android.webkit.WebChromeClient
import android.webkit.WebStorage
import android.webkit.WebView
import androidx.lifecycle.AndroidViewModel
import androidx.lifecycle.viewModelScope
import com.titan.browser.model.Bookmark
import com.titan.browser.model.BrowserSession
import com.titan.browser.model.BrowserSettings
import com.titan.browser.model.HistoryEntry
import com.titan.browser.model.PersistedTab
import com.titan.browser.model.SearchEngine
import com.titan.browser.model.SessionPolicy
import com.titan.browser.model.Tab
import com.titan.browser.model.UpdateState
import com.titan.browser.model.UpdateStatus
import com.titan.browser.storage.StorageManager
import com.titan.browser.update.UpdateChecker
import com.titan.browser.BuildConfig
import com.titan.browser.web.AdblockFilterUpdater
import com.titan.browser.web.AdblockManager
import com.titan.browser.web.BrowserHostDelegate
import com.titan.browser.web.DownloadRequestSpec
import com.titan.browser.web.PrivacyManager
import com.titan.browser.web.TitanWebChromeClient
import com.titan.browser.web.TitanWebViewClient
import com.titan.browser.web.TitanWebViewFactory
import com.titan.browser.web.UrlUtils
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import java.lang.ref.WeakReference
import kotlin.math.abs

class BrowserViewModel(application: Application) : AndroidViewModel(application) {

    private val storageManager = StorageManager(application)
    private var hostDelegate = WeakReference<BrowserHostDelegate>(null)
    private var initialized = false
    private var isRestoringSession = false
    private var hostResumed = false
    private var pendingExternalUrl: String? = null
    private var sessionSaveJob: Job? = null

    fun attachBrowserHost(delegate: BrowserHostDelegate) {
        hostDelegate = WeakReference(delegate)
    }

    fun detachBrowserHost(delegate: BrowserHostDelegate) {
        if (hostDelegate.get() === delegate) {
            hostDelegate.clear()
        }
    }

    private val _tabs = MutableStateFlow<List<Tab>>(emptyList())
    val tabs: StateFlow<List<Tab>> = _tabs.asStateFlow()

    private val _activeTabId = MutableStateFlow<String?>(null)
    val activeTabId: StateFlow<String?> = _activeTabId.asStateFlow()

    private val _bookmarks = MutableStateFlow<List<Bookmark>>(emptyList())
    val bookmarks: StateFlow<List<Bookmark>> = _bookmarks.asStateFlow()

    private val _history = MutableStateFlow<List<HistoryEntry>>(emptyList())
    val history: StateFlow<List<HistoryEntry>> = _history.asStateFlow()

    private val _settings = MutableStateFlow(BrowserSettings())
    val settings: StateFlow<BrowserSettings> = _settings.asStateFlow()

    private val _updateState = MutableStateFlow(
        UpdateState(currentVersion = BuildConfig.VERSION_NAME)
    )
    val updateState: StateFlow<UpdateState> = _updateState.asStateFlow()

    // UI Overlay / Sheet Visibilities
    private val _isTabGridVisible = MutableStateFlow(false)
    val isTabGridVisible: StateFlow<Boolean> = _isTabGridVisible.asStateFlow()

    private val _isMenuVisible = MutableStateFlow(false)
    val isMenuVisible: StateFlow<Boolean> = _isMenuVisible.asStateFlow()

    private val _isBookmarksVisible = MutableStateFlow(false)
    val isBookmarksVisible: StateFlow<Boolean> = _isBookmarksVisible.asStateFlow()

    private val _isHistoryVisible = MutableStateFlow(false)
    val isHistoryVisible: StateFlow<Boolean> = _isHistoryVisible.asStateFlow()

    private val _isSettingsVisible = MutableStateFlow(false)
    val isSettingsVisible: StateFlow<Boolean> = _isSettingsVisible.asStateFlow()

    private val _isFindInPageVisible = MutableStateFlow(false)
    val isFindInPageVisible: StateFlow<Boolean> = _isFindInPageVisible.asStateFlow()

    private val _isToolbarVisible = MutableStateFlow(true)
    val isToolbarVisible: StateFlow<Boolean> = _isToolbarVisible.asStateFlow()

    // High-frequency loading states decoupled from _tabs to eliminate Compose recomposition overhead
    private val _loadingProgress = MutableStateFlow(0)
    val loadingProgress: StateFlow<Int> = _loadingProgress.asStateFlow()

    private val _isLoading = MutableStateFlow(false)
    val isLoading: StateFlow<Boolean> = _isLoading.asStateFlow()

    // Fullscreen video playback state
    private val _customFullscreenView = MutableStateFlow<View?>(null)
    val customFullscreenView: StateFlow<View?> = _customFullscreenView.asStateFlow()
    private var customViewCallback: WebChromeClient.CustomViewCallback? = null

    init {
        loadData()
    }

    private data class LoadedData(
        val bookmarks: List<Bookmark>,
        val filterLists: Map<String, String>,
        val settings: BrowserSettings,
        val history: List<HistoryEntry>,
        val session: BrowserSession
    )

    private fun loadData() {
        viewModelScope.launch {
            val filterSourceIds = AdblockManager.filterListSources.map { it.id }
            val loaded = withContext(Dispatchers.IO) {
                LoadedData(
                    bookmarks = storageManager.loadBookmarks(),
                    filterLists = storageManager.loadAdblockFilterLists(filterSourceIds),
                    settings = storageManager.loadSettings(),
                    history = storageManager.loadHistory(),
                    session = storageManager.loadSession()
                )
            }

            _bookmarks.value = loaded.bookmarks
            _history.value = loaded.history
            _settings.value = loaded.settings
            withContext(Dispatchers.Default) {
                AdblockManager.setCachedFilterLists(loaded.filterLists)
                AdblockManager.prepare(loaded.settings)
            }
            restoreSession(loaded.session)
            initialized = true
            scheduleSessionSave()
            pendingExternalUrl?.also {
                pendingExternalUrl = null
                openUrlFromExternalIntent(it)
            }
            if (loaded.settings.adblockEnabled && loaded.settings.autoUpdateFilterLists) {
                refreshAdblockFilterLists()
            }
            if (loaded.settings.autoUpdateEnabled) {
                checkForUpdates()
            }
        }
    }

    private fun restoreSession(session: BrowserSession) {
        isRestoringSession = true
        val persistedTabs = session.tabs
            .take(25)
            .filter { SessionPolicy.isRestorableUrl(it.url) }
        if (persistedTabs.isEmpty()) {
            openNewTab("titan://newtab")
            isRestoringSession = false
            return
        }

        val restoredTabs = persistedTabs.map { persisted ->
            Tab(
                url = if (persisted.url == "about:blank") "titan://newtab" else persisted.url,
                title = persisted.title.ifBlank { "New Tab" },
                isDesktopMode = persisted.isDesktopMode,
                webView = null
            )
        }
        _tabs.value = restoredTabs
        _activeTabId.value = null
        val activeIndex = session.activeIndex.coerceIn(0, restoredTabs.lastIndex)
        switchTab(restoredTabs[activeIndex].id)
        isRestoringSession = false
        scheduleSessionSave()
    }

    private fun scheduleSessionSave() {
        if (!initialized || isRestoringSession) return
        val session = currentSession()
        sessionSaveJob?.cancel()
        sessionSaveJob = viewModelScope.launch {
            delay(250)
            withContext(Dispatchers.IO) {
                storageManager.saveSession(session)
            }
        }
    }

    private fun currentSession(): BrowserSession {
        val currentTabs = _tabs.value.filterNot { it.isPrivate }
        val activeIndex = _activeTabId.value
            ?.let { id -> currentTabs.indexOfFirst { it.id == id } }
            ?.takeIf { it >= 0 }
            ?: 0
        return BrowserSession(
            tabs = currentTabs.map {
                PersistedTab(
                    url = it.url,
                    title = it.title,
                    isDesktopMode = it.isDesktopMode
                )
            },
            activeIndex = activeIndex
        )
    }

    private fun saveSessionNow() {
        if (!initialized || isRestoringSession) return
        sessionSaveJob?.cancel()
        storageManager.saveSession(currentSession())
    }

    fun getActiveTab(): Tab? {
        val currentId = _activeTabId.value ?: return null
        return _tabs.value.firstOrNull { it.id == currentId }
    }

    fun openNewTab(url: String = "titan://newtab") {
        openTab(url, isPrivate = false)
    }

    fun openPrivateTab() {
        if (!TitanWebViewFactory.isPrivateModeSupported()) {
            hostDelegate.get()?.onShowBrowserMessage(
                "Private tabs require a newer Android System WebView"
            )
            return
        }
        openTab("titan://newtab", isPrivate = true)
    }

    private fun openTab(url: String, isPrivate: Boolean) {
        // Pause previous active tab to save CPU cycles and battery
        getActiveTab()?.webView?.onPause()

        val normalizedUrl = if (url == "titan://newtab" || url == "about:blank") {
            "titan://newtab"
        } else {
            val norm = UrlUtils.normalizeOrSearch(
                url,
                SearchEngine.fromName(_settings.value.searchEngine)
            )
            if (_settings.value.stripTrackingParameters) UrlUtils.stripTrackingParameters(norm) else norm
        }
        val newTab = createTabInstance(normalizedUrl, isPrivate)

        _tabs.update { it + newTab }
        _activeTabId.value = newTab.id
        resumeWebViewIfHostActive(newTab.webView)
        _loadingProgress.value = 0
        _isLoading.value = false
        _isToolbarVisible.value = true
        _isTabGridVisible.value = false
        scheduleSessionSave()
    }

    fun openDownloads() {
        hostDelegate.get()?.onOpenDownloads()
    }

    fun openDefaultBrowserSettings() {
        hostDelegate.get()?.onOpenDefaultBrowserSettings()
    }

    fun openUrlFromExternalIntent(url: String) {
        if (!initialized) {
            pendingExternalUrl = url
            return
        }
        val active = getActiveTab()
        if (active?.url == "titan://newtab" && active.webView == null) {
            navigate(url)
        } else {
            openNewTab(url)
        }
    }

    private fun createTabInstance(initialUrl: String, isPrivate: Boolean): Tab {
        val tabId = java.util.UUID.randomUUID().toString()
        val isNewTab = initialUrl == "titan://newtab" || initialUrl == "about:blank"

        if (isNewTab) {
            return Tab(
                id = tabId,
                url = "titan://newtab",
                title = "New Tab",
                isPrivate = isPrivate,
                webView = null
            )
        }

        val webView = createConfiguredWebView(tabId, isPrivate)
        webView.loadUrl(initialUrl, PrivacyManager.navigationHeaders(_settings.value))

        return Tab(
            id = tabId,
            url = initialUrl,
            title = "Loading...",
            isPrivate = isPrivate,
            webView = webView
        )
    }

    private fun createConfiguredWebView(tabId: String, isPrivate: Boolean): WebView {
        val context = getApplication<Application>()
        val webView = TitanWebViewFactory.createWebView(
            context,
            _settings.value,
            isPrivate = isPrivate
        )
        webView.setOnScrollChangeListener { _, _, scrollY, _, oldScrollY ->
            if (_activeTabId.value == tabId) {
                updateToolbarVisibilityForScroll(scrollY, oldScrollY)
            }
        }

        webView.webChromeClient = TitanWebChromeClient(
            onProgressUpdate = { progress ->
                if (_activeTabId.value == tabId) {
                    if (progress == 100 || abs(progress - _loadingProgress.value) >= 5) {
                        _loadingProgress.value = progress
                    }
                    _isLoading.value = progress < 100
                }
            },
            onTitleUpdate = { title ->
                updateTab(tabId) { it.copy(title = title) }
            },
            onFaviconUpdate = { favicon ->
                updateTab(tabId) { it.copy(favicon = favicon) }
            },
            onShowFullscreen = { view, callback ->
                _customFullscreenView.value = view
                customViewCallback = callback
            },
            onHideFullscreen = {
                hideFullscreenVideo()
            },
            settingsProvider = { _settings.value },
            onCreatePopupTab = { createPopupTab(isPrivate) },
            onShowFileChooserRequest = { callback, params ->
                hostDelegate.get()?.onShowFileChooser(callback, params) ?: false
            },
            onGeolocationPermissionRequest = { origin, callback ->
                hostDelegate.get()?.onGeolocationPermissionRequest(origin, callback)
                    ?: callback.invoke(origin, false, false)
            },
            onWebPermissionRequest = { request ->
                hostDelegate.get()?.onWebPermissionRequest(request) ?: request.deny()
            }
        )

        webView.setDownloadListener { url, userAgent, contentDisposition, mimeType, contentLength ->
            val request = DownloadRequestSpec(
                url = url,
                userAgent = userAgent,
                contentDisposition = contentDisposition,
                mimeType = mimeType,
                contentLength = contentLength,
                referringUrl = webView.url,
                cookieHeader = TitanWebViewFactory.cookieHeader(webView, url)
            )
            hostDelegate.get()?.onDownloadRequested(request)
        }

        webView.webViewClient = TitanWebViewClient(
            context = context,
            settingsProvider = { _settings.value },
            onPageStartedCallback = { pageUrl ->
                if (_activeTabId.value == tabId) {
                    _loadingProgress.value = 10
                    _isLoading.value = true
                    _isToolbarVisible.value = true
                }
                updateTab(tabId) { it.copy(url = pageUrl, isLoading = true) }
            },
            onPageFinishedCallback = { pageUrl, canGoBack, canGoForward ->
                if (_activeTabId.value == tabId) {
                    _loadingProgress.value = 100
                    _isLoading.value = false
                }
                updateTab(tabId) {
                    it.copy(
                        url = pageUrl,
                        isLoading = false,
                        canGoBack = canGoBack,
                        canGoForward = canGoForward
                    )
                }
                recordHistoryVisit(tabId, pageUrl)
            },
            onErrorCallback = { _, _, _ ->
                if (_activeTabId.value == tabId) {
                    _isLoading.value = false
                }
                updateTab(tabId) { it.copy(isLoading = false) }
            }
        )

        return webView
    }

    private fun createPopupTab(isPrivate: Boolean): WebView {
        getActiveTab()?.webView?.onPause()

        val tabId = java.util.UUID.randomUUID().toString()
        val webView = createConfiguredWebView(tabId, isPrivate)
        val tab = Tab(
            id = tabId,
            url = "about:blank",
            title = "Loading...",
            webView = webView,
            isLoading = true,
            isPrivate = isPrivate
        )

        _tabs.update { it + tab }
        _activeTabId.value = tabId
        resumeWebViewIfHostActive(webView)
        _loadingProgress.value = 10
        _isLoading.value = true
        _isToolbarVisible.value = true
        _isTabGridVisible.value = false
        return webView
    }

    private fun updateTab(tabId: String, transform: (Tab) -> Tab) {
        _tabs.update { list ->
            list.map { if (it.id == tabId) transform(it) else it }
        }
        scheduleSessionSave()
    }

    private fun recordHistoryVisit(tabId: String, url: String) {
        if (!SessionPolicy.isRestorableUrl(url) || url.startsWith("titan://")) return
        val tab = _tabs.value.firstOrNull { it.id == tabId } ?: return
        if (tab.isPrivate) return
        val title = tab.title
        viewModelScope.launch {
            _history.value = withContext(Dispatchers.IO) {
                storageManager.recordHistoryVisit(title, url)
            }
        }
    }

    fun switchTab(tabId: String) {
        val currentId = _activeTabId.value
        if (currentId == tabId) {
            _isTabGridVisible.value = false
            return
        }

        val nextTab = _tabs.value.firstOrNull { it.id == tabId } ?: return
        _tabs.value.firstOrNull { it.id == currentId }?.webView?.onPause()
        _activeTabId.value = tabId

        val resumedTab = if (nextTab.webView == null && nextTab.url != "titan://newtab") {
            val webView = createConfiguredWebView(nextTab.id, nextTab.isPrivate)
            val restoredTab = nextTab.copy(webView = webView, isLoading = true)
            updateTab(nextTab.id) { restoredTab }
            webView.loadUrl(nextTab.url, PrivacyManager.navigationHeaders(_settings.value))
            resumeWebViewIfHostActive(webView)
            restoredTab
        } else {
            resumeWebViewIfHostActive(nextTab.webView)
            nextTab
        }

        _isLoading.value = resumedTab.isLoading
        _loadingProgress.value = if (resumedTab.isLoading) 10 else 100
        _isToolbarVisible.value = true
        _isTabGridVisible.value = false
        scheduleSessionSave()
    }

    fun closeTab(tabId: String) {
        val currentTabs = _tabs.value
        val tabToClose = currentTabs.firstOrNull { it.id == tabId }
        tabToClose?.webView?.destroy()

        val updatedTabs = currentTabs.filterNot { it.id == tabId }
        _tabs.value = updatedTabs
        if (tabToClose?.isPrivate == true && updatedTabs.none { it.isPrivate }) {
            TitanWebViewFactory.clearPrivateProfile()
        }

        if (updatedTabs.isEmpty()) {
            openNewTab("titan://newtab")
        } else if (_activeTabId.value == tabId) {
            val nextTab = updatedTabs.last()
            resumeWebViewIfHostActive(nextTab.webView)
            _activeTabId.value = nextTab.id
            _isLoading.value = nextTab.isLoading
            _loadingProgress.value = if (nextTab.isLoading) 50 else 100
            _isToolbarVisible.value = true
        }
        scheduleSessionSave()
    }

    fun navigate(rawInput: String) {
        val active = getActiveTab() ?: return
        if (rawInput == "titan://newtab" || rawInput == "about:blank") {
            active.webView?.destroy()
            _loadingProgress.value = 100
            _isLoading.value = false
            _isToolbarVisible.value = true
            updateTab(active.id) {
                it.copy(
                    url = "titan://newtab",
                    title = "New Tab",
                    isLoading = false,
                    progress = 0,
                    canGoBack = false,
                    canGoForward = false,
                    favicon = null,
                    webView = null
                )
            }
            scheduleSessionSave()
            return
        }
        var url = UrlUtils.normalizeOrSearch(
            rawInput,
            SearchEngine.fromName(_settings.value.searchEngine)
        )
        if (_settings.value.stripTrackingParameters) {
            url = UrlUtils.stripTrackingParameters(url)
        }
        _loadingProgress.value = 10
        _isLoading.value = true
        _isToolbarVisible.value = true
        val webView = active.webView ?: createConfiguredWebView(active.id, active.isPrivate)
        updateTab(active.id) {
            it.copy(
                url = url,
                title = "Loading...",
                isLoading = true,
                webView = webView
            )
        }
        webView.loadUrl(url, PrivacyManager.navigationHeaders(_settings.value))
        resumeWebViewIfHostActive(webView)
        scheduleSessionSave()
    }


    fun goBack(): Boolean {
        val active = getActiveTab() ?: return false
        return if (active.webView?.canGoBack() == true) {
            active.webView.goBack()
            true
        } else {
            false
        }
    }

    fun goForward() {
        val active = getActiveTab() ?: return
        if (active.webView?.canGoForward() == true) {
            active.webView.goForward()
        }
    }

    fun reload() {
        val active = getActiveTab() ?: return
        if (active.isLoading) {
            active.webView?.stopLoading()
        } else {
            active.webView?.reload()
        }
    }

    fun toggleDesktopMode() {
        val active = getActiveTab() ?: return
        val newMode = !active.isDesktopMode
        active.webView?.let {
            TitanWebViewFactory.configureSettings(
                it,
                browserSettings = _settings.value,
                isDesktopMode = newMode,
                isDarkTheme = _settings.value.darkTheme
            )
            it.reload()
        }
        updateTab(active.id) { it.copy(isDesktopMode = newMode) }
        scheduleSessionSave()
    }

    fun toggleBookmarkCurrentPage() {
        val active = getActiveTab() ?: return
        val url = active.url
        val title = active.title.ifBlank { url }

        if (storageManager.isBookmarked(url)) {
            storageManager.removeBookmark(url)
        } else {
            storageManager.addBookmark(Bookmark(title, url))
        }
        _bookmarks.value = storageManager.loadBookmarks()
    }

    fun isCurrentPageBookmarked(): Boolean {
        val active = getActiveTab() ?: return false
        return storageManager.isBookmarked(active.url)
    }

    fun removeBookmark(url: String) {
        storageManager.removeBookmark(url)
        _bookmarks.value = storageManager.loadBookmarks()
    }

    fun updateSearchEngine(engineName: String) {
        val newSettings = _settings.value.copy(searchEngine = engineName)
        _settings.value = newSettings
        storageManager.saveSettings(newSettings)
    }

    fun setDarkTheme(enabled: Boolean) {
        val newSettings = _settings.value.copy(darkTheme = enabled)
        _settings.value = newSettings
        storageManager.saveSettings(newSettings)
    }

    fun toggleAdblock(enabled: Boolean) {
        val newSettings = _settings.value.copy(adblockEnabled = enabled)
        _settings.value = newSettings
        storageManager.saveSettings(newSettings)
        prepareAdblockRules(newSettings)
    }

    fun toggleBlockVideoAds(enabled: Boolean) {
        val newSettings = _settings.value.copy(blockVideoAds = enabled)
        _settings.value = newSettings
        storageManager.saveSettings(newSettings)
    }

    fun toggleCosmeticFiltering(enabled: Boolean) {
        val newSettings = _settings.value.copy(cosmeticFiltering = enabled)
        _settings.value = newSettings
        storageManager.saveSettings(newSettings)
    }

    fun toggleBlockPopups(enabled: Boolean) {
        val newSettings = _settings.value.copy(blockPopups = enabled)
        _settings.value = newSettings
        storageManager.saveSettings(newSettings)
    }

    fun toggleAggressiveAdblock(enabled: Boolean) {
        val newSettings = _settings.value.copy(aggressiveMode = enabled)
        _settings.value = newSettings
        storageManager.saveSettings(newSettings)
    }

    fun toggleAdblockFilterList(listId: String, enabled: Boolean) {
        val current = _settings.value.adblockFilterLists.toMutableList()
        if (enabled && !current.contains(listId)) {
            current.add(listId)
        } else if (!enabled) {
            current.remove(listId)
        }

        val newSettings = _settings.value.copy(adblockFilterLists = current)
        _settings.value = newSettings
        storageManager.saveSettings(newSettings)
        prepareAdblockRules(newSettings)
    }

    fun refreshAdblockFilterLists() {
        viewModelScope.launch {
            val result = AdblockFilterUpdater.update(AdblockManager.filterListSources)
            withContext(Dispatchers.IO) {
                result.updated.forEach { (id, content) ->
                    storageManager.saveAdblockFilterList(id, content)
                }
            }
            withContext(Dispatchers.Default) {
                AdblockManager.setCachedFilterLists(result.updated)
                AdblockManager.prepare(_settings.value)
            }
        }
    }

    private fun prepareAdblockRules(settings: BrowserSettings) {
        if (!settings.adblockEnabled) return
        viewModelScope.launch(Dispatchers.Default) {
            AdblockManager.prepare(settings)
        }
    }

    fun toggleStripTrackingParameters(enabled: Boolean) {
        val newSettings = _settings.value.copy(stripTrackingParameters = enabled)
        _settings.value = newSettings
        storageManager.saveSettings(newSettings)
    }

    fun updatePrivacySettings(newSettings: BrowserSettings) {
        _settings.value = newSettings
        storageManager.saveSettings(newSettings)
        val privacyScript = PrivacyManager.getInjectionScript(newSettings)
        _tabs.value.forEach { tab ->
            tab.webView?.let { webView ->
                TitanWebViewFactory.configureSettings(
                    webView,
                    browserSettings = newSettings,
                    isDesktopMode = tab.isDesktopMode,
                    isDarkTheme = newSettings.darkTheme
                )
                if (privacyScript.isNotEmpty()) {
                    webView.evaluateJavascript(privacyScript, null)
                }
            }
        }
    }

    fun toggleAutoUpdateFilterLists(enabled: Boolean) {
        val newSettings = _settings.value.copy(autoUpdateFilterLists = enabled)
        _settings.value = newSettings
        storageManager.saveSettings(newSettings)
        if (enabled) refreshAdblockFilterLists()
    }

    fun clearBrowsingData() {
        CookieManager.getInstance().removeAllCookies(null)
        CookieManager.getInstance().flush()
        WebStorage.getInstance().deleteAllData()
        _tabs.value.forEach { tab ->
            tab.webView?.apply {
                clearCache(true)
                clearHistory()
                clearFormData()
            }
        }
        storageManager.clearHistory()
        _history.value = emptyList()
    }

    fun toggleAutoUpdate(enabled: Boolean) {
        val newSettings = _settings.value.copy(autoUpdateEnabled = enabled)
        _settings.value = newSettings
        storageManager.saveSettings(newSettings)
        if (enabled) {
            checkForUpdates()
        }
    }

    fun checkForUpdates() {
        _updateState.value = _updateState.value.copy(
            status = UpdateStatus.Checking,
            message = "Checking for updates..."
        )
        viewModelScope.launch {
            _updateState.value = UpdateChecker.check(BuildConfig.VERSION_NAME)
        }
    }

    fun openUpdateRelease() {
        val url = _updateState.value.releaseUrl
            ?: "https://github.com/SketchyLucian/titan-browser/releases/latest"
        openNewTab(url)
        _isSettingsVisible.value = false
    }


    fun hideFullscreenVideo() {
        _customFullscreenView.value = null
        customViewCallback?.onCustomViewHidden()
        customViewCallback = null
    }

    fun onHostResume() {
        hostResumed = true
        resumeWebViewIfHostActive(getActiveTab()?.webView)
    }

    fun onHostPause() {
        hostResumed = false
        CookieManager.getInstance().flush()
        saveSessionNow()
        getActiveTab()?.webView?.apply {
            onPause()
            pauseTimers()
        }
    }

    private fun resumeWebViewIfHostActive(webView: WebView?) {
        if (!hostResumed || webView == null) return
        webView.resumeTimers()
        webView.onResume()
    }

    @Suppress("DEPRECATION")
    fun onTrimMemory(level: Int) {
        if (level < ComponentCallbacks2.TRIM_MEMORY_RUNNING_LOW ||
            level == ComponentCallbacks2.TRIM_MEMORY_UI_HIDDEN
        ) {
            return
        }

        val activeId = _activeTabId.value
        val currentTabs = _tabs.value
        currentTabs.forEach { tab ->
            if (tab.id != activeId) {
                tab.webView?.apply {
                    stopLoading()
                    onPause()
                    destroy()
                }
            }
        }
        _tabs.value = currentTabs.map { tab ->
            if (tab.id != activeId && tab.webView != null) {
                tab.copy(webView = null, isLoading = false, progress = 0)
            } else {
                tab
            }
        }
    }

    private fun updateToolbarVisibilityForScroll(scrollY: Int, oldScrollY: Int) {
        val delta = scrollY - oldScrollY
        if (scrollY <= 8) {
            _isToolbarVisible.value = true
        } else if (delta > 14) {
            _isToolbarVisible.value = false
        } else if (delta < -14) {
            _isToolbarVisible.value = true
        }
    }

    // Sheet visibility toggles
    fun setTabGridVisible(visible: Boolean) {
        _isToolbarVisible.value = true
        _isTabGridVisible.value = visible
    }

    fun setMenuVisible(visible: Boolean) {
        _isToolbarVisible.value = true
        _isMenuVisible.value = visible
    }

    fun setBookmarksVisible(visible: Boolean) {
        _isToolbarVisible.value = true
        _isBookmarksVisible.value = visible
    }

    fun setHistoryVisible(visible: Boolean) {
        _isToolbarVisible.value = true
        _isHistoryVisible.value = visible
    }

    fun setSettingsVisible(visible: Boolean) {
        _isToolbarVisible.value = true
        _isSettingsVisible.value = visible
    }

    fun setFindInPageVisible(visible: Boolean) {
        _isToolbarVisible.value = true
        _isFindInPageVisible.value = visible
    }

    override fun onCleared() {
        saveSessionNow()
        _tabs.value.forEach { it.webView?.destroy() }
        TitanWebViewFactory.clearPrivateProfile()
        super.onCleared()
    }
}
