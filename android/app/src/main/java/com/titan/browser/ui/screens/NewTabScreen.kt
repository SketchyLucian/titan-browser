package com.titan.browser.ui.screens

import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.grid.GridCells
import androidx.compose.foundation.lazy.grid.LazyVerticalGrid
import androidx.compose.foundation.lazy.grid.items
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Public
import androidx.compose.material3.Icon
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.titan.browser.model.Bookmark
import com.titan.browser.ui.theme.TitanBackground
import com.titan.browser.ui.theme.TitanBorder
import com.titan.browser.ui.theme.TitanPrimary
import com.titan.browser.ui.theme.TitanSurface
import com.titan.browser.ui.theme.TitanSurfaceVariant
import com.titan.browser.ui.theme.TitanTextPrimary
import com.titan.browser.ui.theme.TitanTextSecondary
import com.titan.browser.web.UrlUtils

@Composable
fun NewTabScreen(
    modifier: Modifier = Modifier,
    onNavigate: (String) -> Unit = {},
    bookmarks: List<Bookmark> = emptyList(),
    isPrivate: Boolean = false
) {
    Column(
        modifier = modifier
            .fillMaxSize()
            .background(TitanBackground)
            .padding(horizontal = 22.dp, vertical = 28.dp),
        horizontalAlignment = Alignment.CenterHorizontally
    ) {
        Box(
            modifier = Modifier
                .size(58.dp)
                .clip(RoundedCornerShape(17.dp))
                .background(TitanPrimary),
            contentAlignment = Alignment.Center
        ) {
            Icon(
                imageVector = Icons.Default.Public,
                contentDescription = null,
                tint = TitanTextPrimary,
                modifier = Modifier.size(30.dp)
            )
        }
        Spacer(modifier = Modifier.height(12.dp))
        Text(
            text = "TITAN",
            color = TitanTextPrimary,
            fontWeight = FontWeight.Bold,
            fontSize = 24.sp,
            letterSpacing = 2.sp
        )
        Text(
            text = if (isPrivate) {
                "Private browsing · history and site data are not retained"
            } else {
                "Search or enter an address below"
            },
            color = TitanTextSecondary,
            fontSize = 13.sp
        )
        Spacer(modifier = Modifier.height(32.dp))

        if (bookmarks.isNotEmpty()) {
            Text(
                text = "BOOKMARKS",
                color = TitanTextSecondary,
                fontWeight = FontWeight.SemiBold,
                fontSize = 12.sp,
                modifier = Modifier.fillMaxWidth()
            )
            Spacer(modifier = Modifier.height(12.dp))
            LazyVerticalGrid(
                columns = GridCells.Adaptive(92.dp),
                horizontalArrangement = Arrangement.spacedBy(10.dp),
                verticalArrangement = Arrangement.spacedBy(10.dp),
                modifier = Modifier.fillMaxWidth()
            ) {
                items(bookmarks.take(12), key = { it.url }) { bookmark ->
                    Surface(
                        color = TitanSurface,
                        shape = RoundedCornerShape(14.dp),
                        border = BorderStroke(0.5.dp, TitanBorder),
                        modifier = Modifier
                            .height(104.dp)
                            .clickable { onNavigate(bookmark.url) }
                    ) {
                        Column(
                            modifier = Modifier.padding(12.dp),
                            horizontalAlignment = Alignment.CenterHorizontally,
                            verticalArrangement = Arrangement.Center
                        ) {
                            Box(
                                modifier = Modifier
                                    .size(34.dp)
                                    .clip(CircleShape)
                                    .background(TitanSurfaceVariant),
                                contentAlignment = Alignment.Center
                            ) {
                                Text(
                                    text = bookmark.title.take(1).uppercase(),
                                    color = TitanPrimary,
                                    fontWeight = FontWeight.Bold
                                )
                            }
                            Spacer(modifier = Modifier.height(8.dp))
                            Text(
                                text = bookmark.title,
                                color = TitanTextPrimary,
                                fontSize = 12.sp,
                                fontWeight = FontWeight.Medium,
                                maxLines = 1,
                                overflow = TextOverflow.Ellipsis,
                                textAlign = TextAlign.Center
                            )
                            Text(
                                text = UrlUtils.getDomain(bookmark.url),
                                color = TitanTextSecondary,
                                fontSize = 10.sp,
                                maxLines = 1,
                                overflow = TextOverflow.Ellipsis
                            )
                        }
                    }
                }
            }
        }
    }
}
