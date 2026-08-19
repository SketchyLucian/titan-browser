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
import androidx.compose.foundation.layout.statusBarsPadding
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.key
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.viewinterop.AndroidView
import com.titan.browser.ui.components.BookmarksSheet
import com.titan.browser.ui.components.BottomToolbar
import com.titan.browser.ui.components.FindInPageBar
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
    val context = LocalContext.current
    val tabs by viewModel.tabs.collectAsState()
    val activeTabId by viewModel.activeTabId.collectAsState()
    val bookmarks by viewModel.bookmarks.collectAsState()
    val settings by viewModel.settings.collectAsState()

    val isTabGridVisible by viewModel.isTabGridVisible.collectAsState()
    val isMenuVisible by viewModel.isMenuVisible.collectAsState()
    val isBookmarksVisible by viewModel.isBookmarksVisible.collectAsState()
    val isSettingsVisible by viewModel.isSettingsVisible.collectAsState()
    val isFindInPageVisible by viewModel.isFindInPageVisible.collectAsState()
    val fullscreenView by viewModel.customFullscreenView.collectAsState()

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
        } else if (isMenuVisible) {
            viewModel.setMenuVisible(false)
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
        Column(modifier = Modifier.fillMaxSize()) {
            // Omnibar Top Bar
            Box(
                modifier = Modifier
                    .fillMaxWidth()
                    .statusBarsPadding()
            ) {
                Omnibar(
                    currentUrl = activeTab?.url ?: "titan://newtab",
                    isLoading = activeTab?.isLoading ?: false,
                    progress = activeTab?.progress ?: 0,
                    onNavigate = { viewModel.navigate(it) },
                    onReloadOrStop = { viewModel.reload() }
                )
            }

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
                        modifier = Modifier.fillMaxSize()
                    )
                } else {
                    key(activeTabId) {
                        activeTab?.webView?.let { webView ->
                            AndroidView(
                                factory = {
                                    (webView.parent as? ViewGroup)?.removeView(webView)
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

            // Bottom Navigation Toolbar
            BottomToolbar(
                canGoBack = activeTab?.canGoBack ?: false,
                canGoForward = activeTab?.canGoForward ?: false,
                tabCount = tabs.size,
                isBookmarked = viewModel.isCurrentPageBookmarked(),
                onBack = { viewModel.goBack() },
                onForward = { viewModel.goForward() },
                onHome = { viewModel.navigate("titan://newtab") },
                onToggleBookmark = { viewModel.toggleBookmarkCurrentPage() },
                onTabsClick = { viewModel.setTabGridVisible(true) },
                onMenuClick = { viewModel.setMenuVisible(true) }
            )
        }


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
                onUpdateSearchEngine = { viewModel.updateSearchEngine(it) },
                onToggleDarkTheme = { viewModel.setDarkTheme(it) },
                onToggleAdblock = { viewModel.toggleAdblock(it) },
                onToggleBlockVideoAds = { viewModel.toggleBlockVideoAds(it) },
                onToggleCosmeticFiltering = { viewModel.toggleCosmeticFiltering(it) },
                onToggleBlockPopups = { viewModel.toggleBlockPopups(it) },
                onToggleStripTrackingParameters = { viewModel.toggleStripTrackingParameters(it) },
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

        // Menu BottomSheet
        if (isMenuVisible) {
            MenuBottomSheet(
                isDesktopMode = activeTab?.isDesktopMode ?: false,
                onNewTab = { viewModel.openNewTab() },
                onOpenBookmarks = { viewModel.setBookmarksVisible(true) },
                onFindInPage = { viewModel.setFindInPageVisible(true) },
                onToggleDesktopMode = { viewModel.toggleDesktopMode() },
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
