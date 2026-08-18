package com.titan.browser.ui.components

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.automirrored.filled.ArrowForward
import androidx.compose.material.icons.filled.Home
import androidx.compose.material.icons.filled.MoreVert
import androidx.compose.material.icons.filled.Star
import androidx.compose.material.icons.filled.StarOutline
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.titan.browser.ui.theme.TitanBorder
import com.titan.browser.ui.theme.TitanPrimary
import com.titan.browser.ui.theme.TitanSurface
import com.titan.browser.ui.theme.TitanTextPrimary
import com.titan.browser.ui.theme.TitanTextSecondary
import com.titan.browser.ui.theme.TitanTextTertiary

@Composable
fun BottomToolbar(
    canGoBack: Boolean,
    canGoForward: Boolean,
    tabCount: Int,
    isBookmarked: Boolean,
    onBack: () -> Unit,
    onForward: () -> Unit,
    onHome: () -> Unit,
    onToggleBookmark: () -> Unit,
    onTabsClick: () -> Unit,
    onMenuClick: () -> Unit,
    modifier: Modifier = Modifier
) {
    Row(
        modifier = modifier
            .fillMaxWidth()
            .background(TitanSurface)
            .border(width = 0.5.dp, color = TitanBorder)
            .navigationBarsPadding()
            .height(56.dp)
            .padding(horizontal = 8.dp),
        horizontalArrangement = Arrangement.SpaceAround,
        verticalAlignment = Alignment.CenterVertically
    ) {
        // Back
        IconButton(
            onClick = onBack,
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
            onClick = onForward,
            enabled = canGoForward
        ) {
            Icon(
                imageVector = Icons.AutoMirrored.Filled.ArrowForward,
                contentDescription = "Forward",
                tint = if (canGoForward) TitanTextPrimary else TitanTextTertiary
            )
        }

        // Home
        IconButton(onClick = onHome) {
            Icon(
                imageVector = Icons.Default.Home,
                contentDescription = "Home",
                tint = TitanTextPrimary
            )
        }

        // Bookmark Toggle
        IconButton(onClick = onToggleBookmark) {
            Icon(
                imageVector = if (isBookmarked) Icons.Default.Star else Icons.Default.StarOutline,
                contentDescription = "Bookmark",
                tint = if (isBookmarked) Color(0xFFFFB800) else TitanTextPrimary
            )
        }

        // Tabs Count Badge
        Box(
            modifier = Modifier
                .size(32.dp)
                .clip(RoundedCornerShape(8.dp))
                .border(width = 1.5.dp, color = TitanPrimary, shape = RoundedCornerShape(8.dp))
                .clickable { onTabsClick() },
            contentAlignment = Alignment.Center
        ) {
            Text(
                text = tabCount.toString(),
                color = TitanTextPrimary,
                fontSize = 12.sp,
                fontWeight = FontWeight.Bold
            )
        }

        // Overflow Menu
        IconButton(onClick = onMenuClick) {
            Icon(
                imageVector = Icons.Default.MoreVert,
                contentDescription = "Menu",
                tint = TitanTextPrimary
            )
        }
    }
}
