package com.titan.browser.ui.components

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Add
import androidx.compose.material.icons.filled.Bookmark
import androidx.compose.material.icons.filled.DesktopWindows
import androidx.compose.material.icons.filled.FindInPage
import androidx.compose.material.icons.filled.Settings
import androidx.compose.material.icons.filled.Share
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.ModalBottomSheet
import androidx.compose.material3.Switch
import androidx.compose.material3.SwitchDefaults
import androidx.compose.material3.Text
import androidx.compose.material3.rememberModalBottomSheetState
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.titan.browser.ui.theme.TitanBorder
import com.titan.browser.ui.theme.TitanPrimary
import com.titan.browser.ui.theme.TitanSurface
import com.titan.browser.ui.theme.TitanSurfaceVariant
import com.titan.browser.ui.theme.TitanTextPrimary
import com.titan.browser.ui.theme.TitanTextSecondary

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun MenuBottomSheet(
    isDesktopMode: Boolean,
    onNewTab: () -> Unit,
    onOpenBookmarks: () -> Unit,
    onFindInPage: () -> Unit,
    onToggleDesktopMode: () -> Unit,
    onShare: () -> Unit,
    onOpenSettings: () -> Unit,
    onDismiss: () -> Unit
) {
    val sheetState = rememberModalBottomSheetState()

    ModalBottomSheet(
        onDismissRequest = onDismiss,
        sheetState = sheetState,
        containerColor = TitanSurface,
        shape = RoundedCornerShape(topStart = 16.dp, topEnd = 16.dp)
    ) {
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .padding(vertical = 8.dp)
        ) {
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

            Spacer(modifier = Modifier.height(16.dp))
        }
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
