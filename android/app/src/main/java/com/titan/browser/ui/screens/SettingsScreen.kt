package com.titan.browser.ui.screens

import android.webkit.CookieManager
import android.webkit.WebStorage
import android.widget.Toast
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.statusBarsPadding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.filled.Brightness4
import androidx.compose.material.icons.filled.Check
import androidx.compose.material.icons.filled.Delete
import androidx.compose.material.icons.filled.Info
import androidx.compose.material.icons.filled.Lock
import androidx.compose.material.icons.filled.Refresh
import androidx.compose.material.icons.filled.Search
import androidx.compose.material.icons.filled.Security
import androidx.compose.material.icons.filled.SystemUpdate
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.Switch
import androidx.compose.material3.SwitchDefaults
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.window.Dialog
import com.titan.browser.model.BrowserSettings
import com.titan.browser.model.SearchEngine
import com.titan.browser.model.UpdateState
import com.titan.browser.model.UpdateStatus
import com.titan.browser.ui.theme.TitanAccentRed
import com.titan.browser.ui.theme.TitanBackground
import com.titan.browser.ui.theme.TitanBorder
import com.titan.browser.ui.theme.TitanPrimary
import com.titan.browser.ui.theme.TitanSurface
import com.titan.browser.ui.theme.TitanSurfaceVariant
import com.titan.browser.ui.theme.TitanTextPrimary
import com.titan.browser.ui.theme.TitanTextSecondary

@Composable
fun SettingsScreen(
    settings: BrowserSettings,
    updateState: UpdateState,
    onUpdateSearchEngine: (String) -> Unit,
    onToggleDarkTheme: (Boolean) -> Unit,
    onToggleAdblock: (Boolean) -> Unit,
    onToggleBlockVideoAds: (Boolean) -> Unit,
    onToggleCosmeticFiltering: (Boolean) -> Unit,
    onToggleBlockPopups: (Boolean) -> Unit,
    onToggleStripTrackingParameters: (Boolean) -> Unit,
    onToggleAutoUpdate: (Boolean) -> Unit,
    onCheckForUpdates: () -> Unit,
    onOpenUpdate: () -> Unit,
    onClose: () -> Unit,
    modifier: Modifier = Modifier
) {
    val context = LocalContext.current
    var showSearchEngineDialog by remember { mutableStateOf(false) }

    Column(
        modifier = modifier
            .fillMaxSize()
            .background(TitanBackground)
            .statusBarsPadding()
            .navigationBarsPadding()
    ) {
        // App Bar
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .height(56.dp)
                .padding(horizontal = 8.dp),
            verticalAlignment = Alignment.CenterVertically
        ) {
            IconButton(onClick = onClose) {
                Icon(
                    imageVector = Icons.AutoMirrored.Filled.ArrowBack,
                    contentDescription = "Back",
                    tint = TitanTextPrimary
                )
            }
            Spacer(modifier = Modifier.width(8.dp))
            Text(
                text = "Settings",
                color = TitanTextPrimary,
                fontSize = 20.sp,
                fontWeight = FontWeight.Bold
            )
        }

        HorizontalDivider(color = TitanBorder, thickness = 0.5.dp)

        Column(
            modifier = Modifier
                .fillMaxSize()
                .verticalScroll(rememberScrollState())
                .padding(16.dp)
        ) {
            // Section: Shields & Adblock
            Text(
                text = "SHIELDS & ADBLOCK",
                color = TitanPrimary,
                fontSize = 12.sp,
                fontWeight = FontWeight.Bold,
                modifier = Modifier.padding(vertical = 8.dp)
            )

            // Global AdBlocker Toggle
            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(vertical = 8.dp),
                verticalAlignment = Alignment.CenterVertically
            ) {
                Icon(
                    imageVector = Icons.Default.Security,
                    contentDescription = null,
                    tint = TitanPrimary,
                    modifier = Modifier.size(24.dp)
                )
                Spacer(modifier = Modifier.width(16.dp))
                Column(modifier = Modifier.weight(1f)) {
                    Text(
                        text = "Titan Shield (AdBlock)",
                        color = TitanTextPrimary,
                        fontSize = 15.sp,
                        fontWeight = FontWeight.Medium
                    )
                    Text(
                        text = "Block ads, trackers, popunders, and fake robot modals",
                        color = TitanTextSecondary,
                        fontSize = 12.sp
                    )
                }
                Switch(
                    checked = settings.adblockEnabled,
                    onCheckedChange = { onToggleAdblock(it) },
                    colors = SwitchDefaults.colors(
                        checkedThumbColor = TitanPrimary,
                        checkedTrackColor = TitanSurfaceVariant,
                        uncheckedTrackColor = TitanBorder
                    )
                )
            }

            // Block Video Ads Toggle
            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(vertical = 8.dp),
                verticalAlignment = Alignment.CenterVertically
            ) {
                Spacer(modifier = Modifier.width(40.dp))
                Column(modifier = Modifier.weight(1f)) {
                    Text(
                        text = "Block Video Ads",
                        color = TitanTextPrimary,
                        fontSize = 14.sp,
                        fontWeight = FontWeight.Medium
                    )
                    Text(
                        text = "Auto-skip and fast-forward YouTube pre-roll ads",
                        color = TitanTextSecondary,
                        fontSize = 12.sp
                    )
                }
                Switch(
                    checked = settings.blockVideoAds,
                    enabled = settings.adblockEnabled,
                    onCheckedChange = { onToggleBlockVideoAds(it) },
                    colors = SwitchDefaults.colors(
                        checkedThumbColor = TitanPrimary,
                        checkedTrackColor = TitanSurfaceVariant,
                        uncheckedTrackColor = TitanBorder
                    )
                )
            }

            // Cosmetic Element Hiding Toggle
            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(vertical = 8.dp),
                verticalAlignment = Alignment.CenterVertically
            ) {
                Spacer(modifier = Modifier.width(40.dp))
                Column(modifier = Modifier.weight(1f)) {
                    Text(
                        text = "Cosmetic Element Hiding",
                        color = TitanTextPrimary,
                        fontSize = 14.sp,
                        fontWeight = FontWeight.Medium
                    )
                    Text(
                        text = "Hide floating social bars, scam overlays & blank boxes",
                        color = TitanTextSecondary,
                        fontSize = 12.sp
                    )
                }
                Switch(
                    checked = settings.cosmeticFiltering,
                    enabled = settings.adblockEnabled,
                    onCheckedChange = { onToggleCosmeticFiltering(it) },
                    colors = SwitchDefaults.colors(
                        checkedThumbColor = TitanPrimary,
                        checkedTrackColor = TitanSurfaceVariant,
                        uncheckedTrackColor = TitanBorder
                    )
                )
            }

            // Block Popups Toggle
            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(vertical = 8.dp),
                verticalAlignment = Alignment.CenterVertically
            ) {
                Spacer(modifier = Modifier.width(40.dp))
                Column(modifier = Modifier.weight(1f)) {
                    Text(
                        text = "Block Pop-ups & Redirects",
                        color = TitanTextPrimary,
                        fontSize = 14.sp,
                        fontWeight = FontWeight.Medium
                    )
                    Text(
                        text = "Prevent unrequested popunder window opens",
                        color = TitanTextSecondary,
                        fontSize = 12.sp
                    )
                }
                Switch(
                    checked = settings.blockPopups,
                    enabled = settings.adblockEnabled,
                    onCheckedChange = { onToggleBlockPopups(it) },
                    colors = SwitchDefaults.colors(
                        checkedThumbColor = TitanPrimary,
                        checkedTrackColor = TitanSurfaceVariant,
                        uncheckedTrackColor = TitanBorder
                    )
                )
            }

            // Strip Tracking Parameters Toggle
            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(vertical = 8.dp),
                verticalAlignment = Alignment.CenterVertically
            ) {
                Icon(
                    imageVector = Icons.Default.Lock,
                    contentDescription = null,
                    tint = TitanPrimary,
                    modifier = Modifier.size(24.dp)
                )
                Spacer(modifier = Modifier.width(16.dp))
                Column(modifier = Modifier.weight(1f)) {
                    Text(
                        text = "Strip Tracking Parameters",
                        color = TitanTextPrimary,
                        fontSize = 15.sp,
                        fontWeight = FontWeight.Medium
                    )
                    Text(
                        text = "Remove utm_*, fbclid, gclid tracking tags from URLs",
                        color = TitanTextSecondary,
                        fontSize = 12.sp
                    )
                }
                Switch(
                    checked = settings.stripTrackingParameters,
                    onCheckedChange = { onToggleStripTrackingParameters(it) },
                    colors = SwitchDefaults.colors(
                        checkedThumbColor = TitanPrimary,
                        checkedTrackColor = TitanSurfaceVariant,
                        uncheckedTrackColor = TitanBorder
                    )
                )
            }

            Spacer(modifier = Modifier.height(16.dp))

            // Section: Search & Browsing
            Text(
                text = "GENERAL",
                color = TitanPrimary,
                fontSize = 12.sp,
                fontWeight = FontWeight.Bold,
                modifier = Modifier.padding(vertical = 8.dp)
            )

            // Search Engine Setting Item
            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .clickable { showSearchEngineDialog = true }
                    .padding(vertical = 12.dp),
                verticalAlignment = Alignment.CenterVertically
            ) {
                Icon(
                    imageVector = Icons.Default.Search,
                    contentDescription = null,
                    tint = TitanTextSecondary,
                    modifier = Modifier.size(24.dp)
                )
                Spacer(modifier = Modifier.width(16.dp))
                Column(modifier = Modifier.weight(1f)) {
                    Text(
                        text = "Search engine",
                        color = TitanTextPrimary,
                        fontSize = 15.sp,
                        fontWeight = FontWeight.Medium
                    )
                    Text(
                        text = settings.searchEngine,
                        color = TitanTextSecondary,
                        fontSize = 13.sp
                    )
                }
            }

            // Dark Theme Setting Item
            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(vertical = 12.dp),
                verticalAlignment = Alignment.CenterVertically
            ) {
                Icon(
                    imageVector = Icons.Default.Brightness4,
                    contentDescription = null,
                    tint = TitanTextSecondary,
                    modifier = Modifier.size(24.dp)
                )
                Spacer(modifier = Modifier.width(16.dp))
                Text(
                    text = "Dark theme",
                    color = TitanTextPrimary,
                    fontSize = 15.sp,
                    fontWeight = FontWeight.Medium,
                    modifier = Modifier.weight(1f)
                )
                Switch(
                    checked = settings.darkTheme,
                    onCheckedChange = { onToggleDarkTheme(it) },
                    colors = SwitchDefaults.colors(
                        checkedThumbColor = TitanPrimary,
                        checkedTrackColor = TitanSurfaceVariant,
                        uncheckedTrackColor = TitanBorder
                    )
                )
            }

            // Automatic Updates
            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(vertical = 12.dp),
                verticalAlignment = Alignment.CenterVertically
            ) {
                Icon(
                    imageVector = Icons.Default.SystemUpdate,
                    contentDescription = null,
                    tint = TitanTextSecondary,
                    modifier = Modifier.size(24.dp)
                )
                Spacer(modifier = Modifier.width(16.dp))
                Column(modifier = Modifier.weight(1f)) {
                    Text(
                        text = "Automatic update checks",
                        color = TitanTextPrimary,
                        fontSize = 15.sp,
                        fontWeight = FontWeight.Medium
                    )
                    Text(
                        text = "Current ${updateState.currentVersion} • ${updateState.message}",
                        color = when (updateState.status) {
                            UpdateStatus.UpdateAvailable -> TitanPrimary
                            UpdateStatus.Error -> TitanAccentRed
                            else -> TitanTextSecondary
                        },
                        fontSize = 12.sp
                    )
                    Spacer(modifier = Modifier.height(8.dp))
                    Row(verticalAlignment = Alignment.CenterVertically) {
                        Button(
                            onClick = onCheckForUpdates,
                            enabled = updateState.status != UpdateStatus.Checking,
                            colors = ButtonDefaults.buttonColors(containerColor = TitanSurfaceVariant)
                        ) {
                            Icon(
                                imageVector = Icons.Default.Refresh,
                                contentDescription = null,
                                modifier = Modifier.size(16.dp)
                            )
                            Spacer(modifier = Modifier.width(6.dp))
                            Text(
                                text = if (updateState.status == UpdateStatus.Checking) "Checking" else "Check"
                            )
                        }
                        if (!updateState.releaseUrl.isNullOrBlank()) {
                            Spacer(modifier = Modifier.width(8.dp))
                            Button(
                                onClick = onOpenUpdate,
                                colors = ButtonDefaults.buttonColors(containerColor = TitanPrimary)
                            ) {
                                Text(
                                    text = if (updateState.status == UpdateStatus.UpdateAvailable) {
                                        "Get update"
                                    } else {
                                        "Release notes"
                                    }
                                )
                            }
                        }
                    }
                }
                Switch(
                    checked = settings.autoUpdateEnabled,
                    onCheckedChange = { onToggleAutoUpdate(it) },
                    colors = SwitchDefaults.colors(
                        checkedThumbColor = TitanPrimary,
                        checkedTrackColor = TitanSurfaceVariant,
                        uncheckedTrackColor = TitanBorder
                    )
                )
            }

            Spacer(modifier = Modifier.height(16.dp))

            // Section: Privacy & Data
            Text(
                text = "STORAGE",
                color = TitanPrimary,
                fontSize = 12.sp,
                fontWeight = FontWeight.Bold,
                modifier = Modifier.padding(vertical = 8.dp)
            )

            // Clear browsing data
            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .clickable {
                        WebStorage.getInstance().deleteAllData()
                        CookieManager.getInstance().removeAllCookies(null)
                        Toast.makeText(context, "Browsing data cleared", Toast.LENGTH_SHORT).show()
                    }
                    .padding(vertical = 12.dp),
                verticalAlignment = Alignment.CenterVertically
            ) {
                Icon(
                    imageVector = Icons.Default.Delete,
                    contentDescription = null,
                    tint = TitanAccentRed,
                    modifier = Modifier.size(24.dp)
                )
                Spacer(modifier = Modifier.width(16.dp))
                Column {
                    Text(
                        text = "Clear browsing data",
                        color = TitanTextPrimary,
                        fontSize = 15.sp,
                        fontWeight = FontWeight.Medium
                    )
                    Text(
                        text = "Clear cookies, cache, and local storage",
                        color = TitanTextSecondary,
                        fontSize = 13.sp
                    )
                }
            }

            Spacer(modifier = Modifier.height(16.dp))


            // Section: About
            Text(
                text = "ABOUT",
                color = TitanPrimary,
                fontSize = 12.sp,
                fontWeight = FontWeight.Bold,
                modifier = Modifier.padding(vertical = 8.dp)
            )

            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(vertical = 12.dp),
                verticalAlignment = Alignment.CenterVertically
            ) {
                Icon(
                    imageVector = Icons.Default.Info,
                    contentDescription = null,
                    tint = TitanTextSecondary,
                    modifier = Modifier.size(24.dp)
                )
                Spacer(modifier = Modifier.width(16.dp))
                Column {
                    Text(
                        text = "Titan Browser for Android",
                        color = TitanTextPrimary,
                        fontSize = 15.sp,
                        fontWeight = FontWeight.Medium
                    )
                    Text(
                        text = "Version ${updateState.currentVersion} • High-performance modern web browser",
                        color = TitanTextSecondary,
                        fontSize = 13.sp
                    )
                }
            }
        }
    }

    // Search Engine Selection Dialog
    if (showSearchEngineDialog) {
        Dialog(onDismissRequest = { showSearchEngineDialog = false }) {
            Card(
                shape = RoundedCornerShape(16.dp),
                colors = CardDefaults.cardColors(containerColor = TitanSurface),
                modifier = Modifier.fillMaxWidth()
            ) {
                Column(modifier = Modifier.padding(20.dp)) {
                    Text(
                        text = "Choose search engine",
                        color = TitanTextPrimary,
                        fontSize = 18.sp,
                        fontWeight = FontWeight.Bold
                    )
                    Spacer(modifier = Modifier.height(16.dp))

                    SearchEngine.entries.forEach { engine ->
                        val isSelected = engine.displayName == settings.searchEngine
                        Row(
                            modifier = Modifier
                                .fillMaxWidth()
                                .clickable {
                                    onUpdateSearchEngine(engine.displayName)
                                    showSearchEngineDialog = false
                                }
                                .padding(vertical = 12.dp),
                            verticalAlignment = Alignment.CenterVertically
                        ) {
                            Text(
                                text = engine.displayName,
                                color = if (isSelected) TitanPrimary else TitanTextPrimary,
                                fontSize = 15.sp,
                                fontWeight = if (isSelected) FontWeight.Bold else FontWeight.Normal,
                                modifier = Modifier.weight(1f)
                            )
                            if (isSelected) {
                                Icon(
                                    imageVector = Icons.Default.Check,
                                    contentDescription = null,
                                    tint = TitanPrimary,
                                    modifier = Modifier.size(20.dp)
                                )
                            }
                        }
                    }
                }
            }
        }
    }
}
