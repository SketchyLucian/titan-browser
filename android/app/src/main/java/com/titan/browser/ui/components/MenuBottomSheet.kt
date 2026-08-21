package com.titan.browser.ui.components

import androidx.compose.foundation.clickable
import androidx.compose.foundation.gestures.detectTapGestures
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.automirrored.filled.ArrowForward
import androidx.compose.material.icons.filled.Add
import androidx.compose.material.icons.filled.Bookmark
import androidx.compose.material.icons.filled.DesktopWindows
import androidx.compose.material.icons.filled.Download
import androidx.compose.material.icons.filled.FindInPage
import androidx.compose.material.icons.filled.Home
import androidx.compose.material.icons.filled.History
import androidx.compose.material.icons.filled.Lock
import androidx.compose.material.icons.filled.Refresh
import androidx.compose.material.icons.filled.Public
import androidx.compose.material.icons.filled.Settings
import androidx.compose.material.icons.filled.Share
import androidx.compose.material.icons.filled.Star
import androidx.compose.material.icons.filled.StarOutline
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.Surface
import androidx.compose.material3.Switch
import androidx.compose.material3.SwitchDefaults
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.runtime.withFrameNanos
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.CompositingStrategy
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.titan.browser.ui.theme.TitanBorder
import com.titan.browser.ui.theme.TitanPrimary
import com.titan.browser.ui.theme.TitanSurface
import com.titan.browser.ui.theme.TitanSurfaceVariant
import com.titan.browser.ui.theme.TitanTextPrimary
import com.titan.browser.ui.theme.TitanTextSecondary
import com.titan.browser.ui.theme.TitanTextTertiary

@Composable
fun MenuBottomSheet(
    visible: Boolean,
    canGoBack: Boolean,
    canGoForward: Boolean,
    isBookmarked: Boolean,
    isDesktopMode: Boolean,
    onBack: () -> Unit,
    onForward: () -> Unit,
    onHome: () -> Unit,
    onToggleBookmark: () -> Unit,
    onReload: () -> Unit,
    onNewTab: () -> Unit,
    onNewPrivateTab: () -> Unit,
    onOpenBookmarks: () -> Unit,
    onOpenHistory: () -> Unit,
    onOpenDownloads: () -> Unit,
    onOpenDefaultBrowserSettings: () -> Unit,
    onFindInPage: () -> Unit,
    onToggleDesktopMode: () -> Unit,
    onShare: () -> Unit,
    onOpenSettings: () -> Unit,
    onDismiss: () -> Unit
) {
    val dismissInteractionSource = remember { MutableInteractionSource() }
    var isLayerWarmed by remember { mutableStateOf(false) }
    val isWarmingLayer = !isLayerWarmed && !visible

    LaunchedEffect(Unit) {
        withFrameNanos { }
        withFrameNanos { }
        isLayerWarmed = true
    }

    Box(modifier = Modifier.fillMaxSize()) {
        if (visible) {
            Box(
                modifier = Modifier
                    .fillMaxSize()
                    .clickable(
                        interactionSource = dismissInteractionSource,
                        indication = null,
                        onClick = onDismiss
                    )
            )
        }

        Surface(
            color = TitanSurface,
            tonalElevation = 0.dp,
            shadowElevation = 0.dp,
            shape = RoundedCornerShape(topStart = 20.dp, topEnd = 20.dp),
            modifier = Modifier
                .align(Alignment.BottomCenter)
                .fillMaxWidth()
                .navigationBarsPadding()
                .graphicsLayer {
                    compositingStrategy = CompositingStrategy.Offscreen
                    alpha = if (isWarmingLayer) 1f / 255f else 1f
                    translationY = if (isWarmingLayer) {
                        0f
                    } else {
                        if (visible) 0f else size.height
                    }
                }
                .pointerInput(Unit) { detectTapGestures { } }
        ) {
            MenuContent(
                canGoBack = canGoBack,
                canGoForward = canGoForward,
                isBookmarked = isBookmarked,
                isDesktopMode = isDesktopMode,
                onBack = onBack,
                onForward = onForward,
                onHome = onHome,
                onToggleBookmark = onToggleBookmark,
                onReload = onReload,
                onNewTab = onNewTab,
                onNewPrivateTab = onNewPrivateTab,
                onOpenBookmarks = onOpenBookmarks,
                onOpenHistory = onOpenHistory,
                onOpenDownloads = onOpenDownloads,
                onOpenDefaultBrowserSettings = onOpenDefaultBrowserSettings,
                onFindInPage = onFindInPage,
                onToggleDesktopMode = onToggleDesktopMode,
                onShare = onShare,
                onOpenSettings = onOpenSettings,
                onDismiss = onDismiss
            )
        }
    }
}

@Composable
private fun MenuContent(
    canGoBack: Boolean,
    canGoForward: Boolean,
    isBookmarked: Boolean,
    isDesktopMode: Boolean,
    onBack: () -> Unit,
    onForward: () -> Unit,
    onHome: () -> Unit,
    onToggleBookmark: () -> Unit,
    onReload: () -> Unit,
    onNewTab: () -> Unit,
    onNewPrivateTab: () -> Unit,
    onOpenBookmarks: () -> Unit,
    onOpenHistory: () -> Unit,
    onOpenDownloads: () -> Unit,
    onOpenDefaultBrowserSettings: () -> Unit,
    onFindInPage: () -> Unit,
    onToggleDesktopMode: () -> Unit,
    onShare: () -> Unit,
    onOpenSettings: () -> Unit,
    onDismiss: () -> Unit
) {
    Column(
        modifier = Modifier
            .fillMaxWidth()
            .padding(vertical = 4.dp)
    ) {
            // Firefox Style Quick Action Row at the Top of Menu
            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(horizontal = 16.dp, vertical = 4.dp),
                horizontalArrangement = Arrangement.SpaceAround,
                verticalAlignment = Alignment.CenterVertically
            ) {
                // Back
                IconButton(
                    onClick = {
                        onBack()
                        onDismiss()
                    },
                    enabled = canGoBack
                ) {
                    Icon(
                        imageVector = Icons.AutoMirrored.Filled.ArrowBack,
                        contentDescription = "Back",
                        tint = if (canGoBack) TitanTextPrimary else TitanTextTertiary
                    )
                }

                // Forward
                IconButton(
                    onClick = {
                        onForward()
                        onDismiss()
                    },
                    enabled = canGoForward
                ) {
                    Icon(
                        imageVector = Icons.AutoMirrored.Filled.ArrowForward,
                        contentDescription = "Forward",
                        tint = if (canGoForward) TitanTextPrimary else TitanTextTertiary
                    )
                }

                // Reload
                IconButton(
                    onClick = {
                        onReload()
                        onDismiss()
                    }
                ) {
                    Icon(
                        imageVector = Icons.Default.Refresh,
                        contentDescription = "Reload",
                        tint = TitanTextPrimary
                    )
                }

                // Bookmark
                IconButton(
                    onClick = {
                        onToggleBookmark()
                        onDismiss()
                    }
                ) {
                    Icon(
                        imageVector = if (isBookmarked) Icons.Default.Star else Icons.Default.StarOutline,
                        contentDescription = "Bookmark",
                        tint = if (isBookmarked) Color(0xFFFFB800) else TitanTextPrimary
                    )
                }

                // Home
                IconButton(
                    onClick = {
                        onHome()
                        onDismiss()
                    }
                ) {
                    Icon(
                        imageVector = Icons.Default.Home,
                        contentDescription = "Home",
                        tint = TitanTextPrimary
                    )
                }
            }

            HorizontalDivider(
                color = TitanBorder,
                thickness = 0.5.dp,
                modifier = Modifier.padding(vertical = 6.dp)
            )

            MenuItem(
                icon = Icons.Default.Add,
                title = "New tab",
                onClick = {
                    onNewTab()
                    onDismiss()
                }
            )

            MenuItem(
                icon = Icons.Default.Bookmark,
                title = "Bookmarks",
                onClick = {
                    onOpenBookmarks()
                    onDismiss()
                }
            )

            MenuItem(
                icon = Icons.Default.Lock,
                title = "New private tab",
                onClick = {
                    onNewPrivateTab()
                    onDismiss()
                }
            )

            MenuItem(
                icon = Icons.Default.History,
                title = "History",
                onClick = {
                    onOpenHistory()
                    onDismiss()
                }
            )

            MenuItem(
                icon = Icons.Default.Download,
                title = "Downloads",
                onClick = {
                    onOpenDownloads()
                    onDismiss()
                }
            )

            MenuItem(
                icon = Icons.Default.FindInPage,
                title = "Find in page",
                onClick = {
                    onFindInPage()
                    onDismiss()
                }
            )

            MenuItem(
                icon = Icons.Default.Share,
                title = "Share",
                onClick = {
                    onShare()
                    onDismiss()
                }
            )

            HorizontalDivider(
                color = TitanBorder,
                thickness = 0.5.dp,
                modifier = Modifier.padding(vertical = 4.dp)
            )

            // Desktop Site Switch Item
            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .clickable { onToggleDesktopMode() }
                    .padding(horizontal = 20.dp, vertical = 12.dp),
                verticalAlignment = Alignment.CenterVertically
            ) {
                Icon(
                    imageVector = Icons.Default.DesktopWindows,
                    contentDescription = null,
                    tint = if (isDesktopMode) TitanPrimary else TitanTextSecondary,
                    modifier = Modifier.size(22.dp)
                )
                Spacer(modifier = Modifier.width(16.dp))
                Text(
                    text = "Desktop site",
                    color = TitanTextPrimary,
                    fontSize = 15.sp,
                    fontWeight = FontWeight.Medium,
                    modifier = Modifier.weight(1f)
                )
                Switch(
                    checked = isDesktopMode,
                    onCheckedChange = { onToggleDesktopMode() },
                    colors = SwitchDefaults.colors(
                        checkedThumbColor = TitanPrimary,
                        checkedTrackColor = TitanSurfaceVariant,
                        uncheckedTrackColor = TitanBorder
                    )
                )
            }

            HorizontalDivider(
                color = TitanBorder,
                thickness = 0.5.dp,
                modifier = Modifier.padding(vertical = 4.dp)
            )

            MenuItem(
                icon = Icons.Default.Settings,
                title = "Settings",
                onClick = {
                    onOpenSettings()
                    onDismiss()
                }
            )

            MenuItem(
                icon = Icons.Default.Public,
                title = "Set as default browser",
                onClick = {
                    onOpenDefaultBrowserSettings()
                    onDismiss()
                }
            )

            Spacer(modifier = Modifier.height(16.dp))
    }
}

@Composable
fun MenuItem(
    icon: ImageVector,
    title: String,
    onClick: () -> Unit
) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .clickable { onClick() }
            .padding(horizontal = 20.dp, vertical = 14.dp),
        verticalAlignment = Alignment.CenterVertically
    ) {
        Icon(
            imageVector = icon,
            contentDescription = null,
            tint = TitanTextSecondary,
            modifier = Modifier.size(22.dp)
        )
        Spacer(modifier = Modifier.width(16.dp))
        Text(
            text = title,
            color = TitanTextPrimary,
            fontSize = 15.sp,
            fontWeight = FontWeight.Medium
        )
    }
}
