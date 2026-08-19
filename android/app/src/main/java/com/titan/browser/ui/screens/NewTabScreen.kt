package com.titan.browser.ui.screens

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.size
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.AutoAwesome
import androidx.compose.material3.Icon
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.unit.dp
import com.titan.browser.model.Bookmark
import com.titan.browser.ui.theme.TitanBackground
import com.titan.browser.ui.theme.TitanTextTertiary

@Composable
fun NewTabScreen(
    onNavigate: (String) -> Unit = {},
    bookmarks: List<Bookmark> = emptyList(),
    modifier: Modifier = Modifier
) {
    Box(
        modifier = modifier
            .fillMaxSize()
            .background(TitanBackground),
        contentAlignment = Alignment.Center
    ) {
        // Subtle, elegant minimal watermark in center
        Icon(
            imageVector = Icons.Default.AutoAwesome,
            contentDescription = null,
            tint = TitanTextTertiary,
            modifier = Modifier
                .size(48.dp)
                .alpha(0.18f)
        )
    }
}
