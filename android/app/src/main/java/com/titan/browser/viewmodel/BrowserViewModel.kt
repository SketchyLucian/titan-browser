package com.titan.browser.viewmodel

import android.app.Application
import android.graphics.Bitmap
import android.view.View
import android.webkit.WebChromeClient
import android.webkit.WebView
import androidx.lifecycle.AndroidViewModel
import androidx.lifecycle.viewModelScope
import com.titan.browser.model.Bookmark
import com.titan.browser.model.BrowserSettings
import com.titan.browser.model.SearchEngine
import com.titan.browser.model.Tab
import com.titan.browser.model.UpdateState
import com.titan.browser.model.UpdateStatus
import com.titan.browser.storage.StorageManager
import com.titan.browser.update.UpdateChecker
import com.titan.browser.BuildConfig
import com.titan.browser.web.AdblockFilterUpdater
import com.titan.browser.web.AdblockManager
import com.titan.browser.web.TitanWebChromeClient
import com.titan.browser.web.TitanWebViewClient
import com.titan.browser.web.TitanWebViewFactory
import com.titan.browser.web.UrlUtils
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext

class BrowserViewModel(application: Application) : AndroidViewModel(application) {

    private val storageManager = StorageManager(application)

    private val _tabs = MutableStateFlow<List<Tab>>(emptyList())
    val tabs: StateFlow<List<Tab>> = _tabs.asStateFlow()

    private val _activeTabId = MutableStateFlow<String?>(null)
    val activeTabId: StateFlow<String?> = _activeTabId.asStateFlow()

    private val _bookmarks = MutableStateFlow<List<Bookmark>>(emptyList())
    val bookmarks: StateFlow<List<Bookmark>> = _bookmarks.asStateFlow()

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
        // Open default initial tab (Titan New Tab)
        openNewTab("titan://newtab")
    }

    private fun loadData() {
        viewModelScope.launch {
            val filterSourceIds = AdblockManager.filterListSources.map { it.id }
            val (loadedBookmarks, cachedFilterLists, loadedSettings) = withContext(Dispatchers.IO) {
                Triple(
                    storageManager.loadBookmarks(),
                    storageManager.loadAdblockFilterLists(filterSourceIds),
                    storageManager.loadSettings()
                )
            }

            _bookmarks.value = loadedBookmarks
            _settings.value = loadedSettings
            withContext(Dispatchers.Default) {
                AdblockManager.setCachedFilterLists(cachedFilterLists)
                AdblockManager.prepare(loadedSettings)
            }
            if (loadedSettings.adblockEnabled) {
                refreshAdblockFilterLists()
            }
            if (loadedSettings.autoUpdateEnabled) {
                checkForUpdates()
            }
        }
    }

    fun getActiveTab(): Tab? {
        val currentId = _activeTabId.value ?: return null
        return _tabs.value.firstOrNull { it.id == currentId }
    }

    fun openNewTab(url: String = "titan://newtab") {
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
        val newTab = createTabInstance(normalizedUrl)

        _tabs.update { it + newTab }
        _activeTabId.value = newTab.id
        _loadingProgress.value = 0
        _isLoading.value = false
        _isToolbarVisible.value = true
        _isTabGridVisible.value = false
    }

    fun openUrlFromExternalIntent(url: String) {
        val active = getActiveTab()
        if (active?.url == "titan://newtab" && active.webView == null) {
            navigate(url)
        } else {
            openNewTab(url)
        }
    }

    private fun createTabInstance(initialUrl: String): Tab {
        val tabId = java.util.UUID.randomUUID().toString()
        val isNewTab = initialUrl == "titan://newtab" || initialUrl == "about:blank"

        if (isNewTab) {
            return Tab(
                id = tabId,
                url = "titan://newtab",
                title = "New Tab",
                webView = null
            )
        }

        val webView = createConfiguredWebView(tabId)
        webView.loadUrl(initialUrl)

        return Tab(
            id = tabId,
            url = initialUrl,
            title = "Loading...",
            webView = webView
        )
    }

    private fun createConfiguredWebView(tabId: String): WebView {
        val context = getApplication<Application>()
        val webView = TitanWebViewFactory.createWebView(context)
        webView.setOnScrollChangeListener { _, _, scrollY, _, oldScrollY ->
            if (_activeTabId.value == tabId) {
                updateToolbarVisibilityForScroll(scrollY, oldScrollY)
            }
        }

        webView.webChromeClient = TitanWebChromeClient(
            onProgressUpdate = { progress ->
                if (_activeTabId.value == tabId) {
                    _loadingProgress.value = progress
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
            }
        )

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

    private fun updateTab(tabId: String, transform: (Tab) -> Tab) {
        _tabs.update { list ->
            list.map { if (it.id == tabId) transform(it) else it }
        }
    }

    fun switchTab(tabId: String) {
        val currentId = _activeTabId.value
        if (currentId == tabId) {
            _isTabGridVisible.value = false
            return
        }

        _tabs.value.forEach { tab ->
            if (tab.id == tabId) {
                tab.webView?.onResume()
            } else if (tab.id == currentId) {
                tab.webView?.onPause()
            }
        }

        if (_tabs.value.any { it.id == tabId }) {
            _activeTabId.value = tabId
            val nextTab = _tabs.value.firstOrNull { it.id == tabId }
            _isLoading.value = nextTab?.isLoading ?: false
            _loadingProgress.value = if (nextTab?.isLoading == true) 50 else 100
            _isToolbarVisible.value = true
            _isTabGridVisible.value = false
        }
    }

    fun closeTab(tabId: String) {
        val currentTabs = _tabs.value
        val tabToClose = currentTabs.firstOrNull { it.id == tabId }
        tabToClose?.webView?.destroy()

        val updatedTabs = currentTabs.filterNot { it.id == tabId }
        _tabs.value = updatedTabs

        if (updatedTabs.isEmpty()) {
            openNewTab("titan://newtab")
        } else if (_activeTabId.value == tabId) {
            val nextTab = updatedTabs.last()
            nextTab.webView?.onResume()
            _activeTabId.value = nextTab.id
            _isLoading.value = nextTab.isLoading
            _loadingProgress.value = if (nextTab.isLoading) 50 else 100
            _isToolbarVisible.value = true
        }
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
        val webView = active.webView ?: createConfiguredWebView(active.id)
        updateTab(active.id) {
            it.copy(
                url = url,
                title = "Loading...",
                isLoading = true,
                webView = webView
            )
        }
        webView.loadUrl(url)
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
                isDesktopMode = newMode,
                isDarkTheme = _settings.value.darkTheme
            )
            it.reload()
        }
        updateTab(active.id) { it.copy(isDesktopMode = newMode) }
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

    fun setSettingsVisible(visible: Boolean) {
        _isToolbarVisible.value = true
        _isSettingsVisible.value = visible
    }

    fun setFindInPageVisible(visible: Boolean) {
        _isToolbarVisible.value = true
        _isFindInPageVisible.value = visible
    }

    override fun onCleared() {
        super.onCleared()
        _tabs.value.forEach { it.webView?.destroy() }
    }
}
