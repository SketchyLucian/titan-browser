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
import com.titan.browser.storage.StorageManager
import com.titan.browser.web.TitanWebChromeClient
import com.titan.browser.web.TitanWebViewClient
import com.titan.browser.web.TitanWebViewFactory
import com.titan.browser.web.UrlUtils
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch

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
            _bookmarks.value = storageManager.loadBookmarks()
            _settings.value = storageManager.loadSettings()
        }
    }

    fun getActiveTab(): Tab? {
        val currentId = _activeTabId.value ?: return null
        return _tabs.value.firstOrNull { it.id == currentId }
    }

    fun openNewTab(url: String = "titan://newtab") {
        var normalizedUrl = if (url == "titan://newtab" || url == "about:blank") {
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
        _isTabGridVisible.value = false
    }

    private fun createTabInstance(initialUrl: String): Tab {
        val context = getApplication<Application>()
        val tabId = java.util.UUID.randomUUID().toString()
        val isNewTab = initialUrl == "titan://newtab" || initialUrl == "about:blank"

        val webView = TitanWebViewFactory.createWebView(context)
        val tab = Tab(
            id = tabId,
            url = initialUrl,
            title = if (isNewTab) "New Tab" else "Loading...",
            webView = webView
        )

        webView.webChromeClient = TitanWebChromeClient(
            onProgressUpdate = { progress ->
                updateTab(tabId) { it.copy(progress = progress, isLoading = progress < 100) }
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
                updateTab(tabId) { it.copy(url = pageUrl, isLoading = true) }
            },
            onPageFinishedCallback = { pageUrl, canGoBack, canGoForward ->
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
                updateTab(tabId) { it.copy(isLoading = false) }
            }
        )

        if (!isNewTab) {
            webView.loadUrl(initialUrl)
        }
        return tab
    }

    private fun updateTab(tabId: String, transform: (Tab) -> Tab) {
        _tabs.update { list ->
            list.map { if (it.id == tabId) transform(it) else it }
        }
    }

    fun switchTab(tabId: String) {
        if (_tabs.value.any { it.id == tabId }) {
            _activeTabId.value = tabId
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
            _activeTabId.value = updatedTabs.last().id
        }
    }

    fun navigate(rawInput: String) {
        val active = getActiveTab() ?: return
        if (rawInput == "titan://newtab" || rawInput == "about:blank") {
            updateTab(active.id) {
                it.copy(
                    url = "titan://newtab",
                    title = "New Tab",
                    isLoading = false,
                    progress = 0
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
        updateTab(active.id) { it.copy(url = url, isLoading = true) }
        active.webView?.loadUrl(url)
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

    fun toggleStripTrackingParameters(enabled: Boolean) {
        val newSettings = _settings.value.copy(stripTrackingParameters = enabled)
        _settings.value = newSettings
        storageManager.saveSettings(newSettings)
    }


    fun hideFullscreenVideo() {
        _customFullscreenView.value = null
        customViewCallback?.onCustomViewHidden()
        customViewCallback = null
    }

    // Sheet visibility toggles
    fun setTabGridVisible(visible: Boolean) { _isTabGridVisible.value = visible }
    fun setMenuVisible(visible: Boolean) { _isMenuVisible.value = visible }
    fun setBookmarksVisible(visible: Boolean) { _isBookmarksVisible.value = visible }
    fun setSettingsVisible(visible: Boolean) { _isSettingsVisible.value = visible }
    fun setFindInPageVisible(visible: Boolean) { _isFindInPageVisible.value = visible }

    override fun onCleared() {
        super.onCleared()
        _tabs.value.forEach { it.webView?.destroy() }
    }
}
