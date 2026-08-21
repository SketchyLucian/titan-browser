package com.titan.browser.ui.screens

import android.content.Intent
import android.view.ViewGroup
import android.widget.FrameLayout
import androidx.activity.compose.BackHandler
import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.slideInVertically
import androidx.compose.animation.slideOutVertically
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.imePadding
import androidx.compose.foundation.layout.statusBarsPadding
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.key
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.viewinterop.AndroidView
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.titan.browser.model.Tab
import com.titan.browser.ui.components.BookmarksSheet
import com.titan.browser.ui.components.BottomToolbar
import com.titan.browser.ui.components.FindInPageBar
import com.titan.browser.ui.components.HistorySheet
import com.titan.browser.ui.components.MenuBottomSheet
import com.titan.browser.ui.components.Omnibar
import com.titan.browser.ui.components.TabGrid
import com.titan.browser.ui.theme.TitanBackground
import com.titan.browser.viewmodel.BrowserViewModel

@Composable
fun BrowserScreen(
    viewModel: BrowserViewModel,
    modifier: Modifier = Modifier
) {
    val tabs by viewModel.tabs.collectAsStateWithLifecycle()
    val activeTabId by viewModel.activeTabId.collectAsStateWithLifecycle()
    val bookmarks by viewModel.bookmarks.collectAsStateWithLifecycle()
    val history by viewModel.history.collectAsStateWithLifecycle()
    val settings by viewModel.settings.collectAsStateWithLifecycle()
    val updateState by viewModel.updateState.collectAsStateWithLifecycle()

    val isTabGridVisible by viewModel.isTabGridVisible.collectAsStateWithLifecycle()
    val isBookmarksVisible by viewModel.isBookmarksVisible.collectAsStateWithLifecycle()
    val isHistoryVisible by viewModel.isHistoryVisible.collectAsStateWithLifecycle()
    val isSettingsVisible by viewModel.isSettingsVisible.collectAsStateWithLifecycle()
    val isFindInPageVisible by viewModel.isFindInPageVisible.collectAsStateWithLifecycle()
    val fullscreenView by viewModel.customFullscreenView.collectAsStateWithLifecycle()

    val activeTab = tabs.firstOrNull { it.id == activeTabId }

    // Intercept back button for fullscreen video, sheets, or webpage back navigation
    BackHandler {
        if (fullscreenView != null) {
            viewModel.hideFullscreenVideo()
        } else if (isSettingsVisible) {
            viewModel.setSettingsVisible(false)
        } else if (isTabGridVisible) {
            viewModel.setTabGridVisible(false)
        } else if (isBookmarksVisible) {
            viewModel.setBookmarksVisible(false)
        } else if (isHistoryVisible) {
            viewModel.setHistoryVisible(false)
        } else if (isFindInPageVisible) {
            viewModel.setFindInPageVisible(false)
        } else {
            val handled = viewModel.goBack()
            if (!handled) {
                // If can't go back, and multiple tabs exist, close current tab
                if (tabs.size > 1 && activeTab != null) {
                    viewModel.closeTab(activeTab.id)
                }
            }
        }
    }

    Box(
        modifier = modifier
            .fillMaxSize()
            .background(TitanBackground)
    ) {
        Column(
            modifier = Modifier
                .fillMaxSize()
                .statusBarsPadding()
        ) {
            // Find in page bar (if active)
            if (isFindInPageVisible) {
                FindInPageBar(
                    onSearch = { query ->
                        activeTab?.webView?.findAllAsync(query)
                    },
                    onFindNext = {
                        activeTab?.webView?.findNext(true)
                    },
                    onFindPrevious = {
                        activeTab?.webView?.findNext(false)
                    },
                    onClose = {
                        activeTab?.webView?.clearMatches()
                        viewModel.setFindInPageVisible(false)
                    }
                )
            }

            // Central Content Container (NewTabScreen or WebView)
            Box(
                modifier = Modifier
                    .weight(1f)
                    .fillMaxWidth()
            ) {
                val isNewTab = activeTab?.url == "titan://newtab" || activeTab?.url == "about:blank"
                if (isNewTab) {
                    NewTabScreen(
                        onNavigate = { viewModel.navigate(it) },
                        bookmarks = bookmarks,
                        isPrivate = activeTab?.isPrivate == true,
                        modifier = Modifier.fillMaxSize()
                    )
                } else {
                    key(activeTabId) {
                        activeTab?.webView?.let { webView ->
                            AndroidView(
                                factory = {
                                    (webView.parent as? ViewGroup)?.removeView(webView)
                                    webView.layoutParams = ViewGroup.LayoutParams(
                                        ViewGroup.LayoutParams.MATCH_PARENT,
                                        ViewGroup.LayoutParams.MATCH_PARENT
                                    )
                                    webView
                                },
                                update = { view ->
                                    if (view != webView) {
                                        (webView.parent as? ViewGroup)?.removeView(webView)
                                    }
                                },
                                modifier = Modifier.fillMaxSize()
                            )
                        }
                    }
                }
            }
        }

        // Firefox Android Style Bottom Search & Navigation Bar
        BrowserToolbar(
            viewModel = viewModel,
            currentUrl = activeTab?.url ?: "titan://newtab",
            tabCount = tabs.size,
            canShow = fullscreenView == null &&
                !isSettingsVisible &&
                !isTabGridVisible &&
                !isBookmarksVisible &&
                !isHistoryVisible,
            modifier = Modifier
                .align(Alignment.BottomCenter)
                .imePadding()
        )

        // Tab Grid Switcher Screen
        AnimatedVisibility(
            visible = isTabGridVisible,
            enter = slideInVertically(initialOffsetY = { it }),
            exit = slideOutVertically(targetOffsetY = { it })
        ) {
            TabGrid(
                tabs = tabs,
                activeTabId = activeTabId,
                onSelectTab = { viewModel.switchTab(it) },
                onCloseTab = { viewModel.closeTab(it) },
                onNewTab = { viewModel.openNewTab() },
                onClose = { viewModel.setTabGridVisible(false) }
            )
        }

        // Settings Screen
        AnimatedVisibility(
            visible = isSettingsVisible,
            enter = slideInVertically(initialOffsetY = { it }),
            exit = slideOutVertically(targetOffsetY = { it })
        ) {
            SettingsScreen(
                settings = settings,
                updateState = updateState,
                onUpdateSearchEngine = { viewModel.updateSearchEngine(it) },
                onToggleDarkTheme = { viewModel.setDarkTheme(it) },
                onToggleAdblock = { viewModel.toggleAdblock(it) },
                onToggleBlockVideoAds = { viewModel.toggleBlockVideoAds(it) },
                onToggleCosmeticFiltering = { viewModel.toggleCosmeticFiltering(it) },
                onToggleBlockPopups = { viewModel.toggleBlockPopups(it) },
                onToggleAggressiveAdblock = { viewModel.toggleAggressiveAdblock(it) },
                onToggleFilterList = { listId, enabled -> viewModel.toggleAdblockFilterList(listId, enabled) },
                onUpdatePrivacySettings = { viewModel.updatePrivacySettings(it) },
                onToggleAutoUpdateFilterLists = { viewModel.toggleAutoUpdateFilterLists(it) },
                onRefreshFilterLists = { viewModel.refreshAdblockFilterLists() },
                onClearBrowsingData = { viewModel.clearBrowsingData() },
                onToggleAutoUpdate = { viewModel.toggleAutoUpdate(it) },
                onCheckForUpdates = { viewModel.checkForUpdates() },
                onOpenUpdate = { viewModel.openUpdateRelease() },
                onClose = { viewModel.setSettingsVisible(false) }
            )

        }

        // Bookmarks BottomSheet
        if (isBookmarksVisible) {
            BookmarksSheet(
                bookmarks = bookmarks,
                onSelectBookmark = { url ->
                    viewModel.navigate(url)
                    viewModel.setBookmarksVisible(false)
                },
                onDeleteBookmark = { url ->
                    viewModel.removeBookmark(url)
                },
                onDismiss = { viewModel.setBookmarksVisible(false) }
            )
        }

        if (isHistoryVisible) {
            HistorySheet(
                history = history,
                onSelect = { url ->
                    viewModel.navigate(url)
                    viewModel.setHistoryVisible(false)
                },
                onDismiss = { viewModel.setHistoryVisible(false) }
            )
        }

        BrowserMenuHost(
            viewModel = viewModel,
            activeTab = activeTab,
            isBookmarked = activeTab?.let { tab -> bookmarks.any { it.url == tab.url } } == true
        )

        // Fullscreen YouTube / HTML5 Video Overlay Container
        fullscreenView?.let { customView ->
            Box(
                modifier = Modifier
                    .fillMaxSize()
                    .background(Color.Black)
            ) {
                AndroidView(
                    factory = {
                        FrameLayout(it).apply {
                            (customView.parent as? ViewGroup)?.removeView(customView)
                            addView(
                                customView,
                                FrameLayout.LayoutParams(
                                    ViewGroup.LayoutParams.MATCH_PARENT,
                                    ViewGroup.LayoutParams.MATCH_PARENT
                                )
                            )
                        }
                    },
                    modifier = Modifier.fillMaxSize()
                )
            }
        }
    }
}

@Composable
private fun BrowserToolbar(
    viewModel: BrowserViewModel,
    currentUrl: String,
    tabCount: Int,
    canShow: Boolean,
    modifier: Modifier = Modifier
) {
    val loadingProgress by viewModel.loadingProgress.collectAsStateWithLifecycle()
    val isLoading by viewModel.isLoading.collectAsStateWithLifecycle()
    val isToolbarVisible by viewModel.isToolbarVisible.collectAsStateWithLifecycle()

    AnimatedVisibility(
        visible = canShow && isToolbarVisible,
        enter = slideInVertically(initialOffsetY = { it }),
        exit = slideOutVertically(targetOffsetY = { it }),
        modifier = modifier
    ) {
        BottomToolbar(
            currentUrl = currentUrl,
            isLoading = isLoading,
            progress = loadingProgress,
            tabCount = tabCount,
            onNavigate = viewModel::navigate,
            onReloadOrStop = viewModel::reload,
            onTabsClick = { viewModel.setTabGridVisible(true) },
            onMenuClick = { viewModel.setMenuVisible(true) }
        )
    }
}

@Composable
private fun BrowserMenuHost(
    viewModel: BrowserViewModel,
    activeTab: Tab?,
    isBookmarked: Boolean
) {
    val isMenuVisible by viewModel.isMenuVisible.collectAsStateWithLifecycle()
    val context = LocalContext.current
    MenuBottomSheet(
        visible = isMenuVisible,
        canGoBack = activeTab?.canGoBack ?: false,
        canGoForward = activeTab?.canGoForward ?: false,
        isBookmarked = isBookmarked,
        isDesktopMode = activeTab?.isDesktopMode ?: false,
        onBack = viewModel::goBack,
        onForward = viewModel::goForward,
        onHome = { viewModel.navigate("titan://newtab") },
        onToggleBookmark = viewModel::toggleBookmarkCurrentPage,
        onReload = viewModel::reload,
        onNewTab = viewModel::openNewTab,
        onNewPrivateTab = viewModel::openPrivateTab,
        onOpenBookmarks = { viewModel.setBookmarksVisible(true) },
        onOpenHistory = { viewModel.setHistoryVisible(true) },
        onOpenDownloads = viewModel::openDownloads,
        onOpenDefaultBrowserSettings = viewModel::openDefaultBrowserSettings,
        onFindInPage = { viewModel.setFindInPageVisible(true) },
        onToggleDesktopMode = viewModel::toggleDesktopMode,
        onShare = {
            activeTab?.let { tab ->
                val shareIntent = Intent(Intent.ACTION_SEND).apply {
                    type = "text/plain"
                    putExtra(Intent.EXTRA_SUBJECT, tab.title)
                    putExtra(Intent.EXTRA_TEXT, tab.url)
                }
                context.startActivity(Intent.createChooser(shareIntent, "Share link"))
            }
        },
        onOpenSettings = { viewModel.setSettingsVisible(true) },
        onDismiss = { viewModel.setMenuVisible(false) }
    )
}
