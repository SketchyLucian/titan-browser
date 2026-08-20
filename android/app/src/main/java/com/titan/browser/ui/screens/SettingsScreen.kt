package com.titan.browser.ui.screens

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
import androidx.compose.material.icons.filled.Brightness4
import androidx.compose.material.icons.filled.Check
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
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.window.Dialog
import com.titan.browser.model.BrowserSettings
import com.titan.browser.model.SearchEngine
import com.titan.browser.model.UpdateState
import com.titan.browser.model.UpdateStatus
import com.titan.browser.ui.theme.TitanAccentRed
import com.titan.browser.ui.theme.TitanPrimary
import com.titan.browser.ui.theme.TitanSurface
import com.titan.browser.ui.theme.TitanSurfaceVariant
import com.titan.browser.ui.theme.TitanTextPrimary
import com.titan.browser.ui.theme.TitanTextSecondary
import com.titan.browser.web.AdblockManager

private enum class SettingsDestination { Home, Privacy, Adblock }

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
    onToggleAggressiveAdblock: (Boolean) -> Unit,
    onToggleFilterList: (String, Boolean) -> Unit,
    onUpdatePrivacySettings: (BrowserSettings) -> Unit,
    onToggleAutoUpdateFilterLists: (Boolean) -> Unit,
    onRefreshFilterLists: () -> Unit,
    onClearBrowsingData: () -> Unit,
    onToggleAutoUpdate: (Boolean) -> Unit,
    onCheckForUpdates: () -> Unit,
    onOpenUpdate: () -> Unit,
    onClose: () -> Unit,
    modifier: Modifier = Modifier
) {
    var destination by rememberSaveable { mutableStateOf(SettingsDestination.Home) }
    var showSearchEngineDialog by rememberSaveable { mutableStateOf(false) }

    when (destination) {
        SettingsDestination.Privacy -> PrivacySettingsScreen(
            settings = settings,
            onSettingsChange = onUpdatePrivacySettings,
            onClearBrowsingData = onClearBrowsingData,
            onBack = { destination = SettingsDestination.Home },
            modifier = modifier
        )
        SettingsDestination.Adblock -> AdblockSettingsScreen(
            settings = settings,
            onToggleAdblock = onToggleAdblock,
            onToggleBlockVideoAds = onToggleBlockVideoAds,
            onToggleCosmeticFiltering = onToggleCosmeticFiltering,
            onToggleBlockPopups = onToggleBlockPopups,
            onToggleAggressiveAdblock = onToggleAggressiveAdblock,
            onToggleFilterList = onToggleFilterList,
            onToggleAutoUpdateFilterLists = onToggleAutoUpdateFilterLists,
            onRefreshFilterLists = onRefreshFilterLists,
            onBack = { destination = SettingsDestination.Home },
            modifier = modifier
        )
        SettingsDestination.Home -> SettingsPageScaffold(
            title = "Settings",
            onBack = onClose,
            modifier = modifier
        ) {
            SettingsSectionTitle("PRIVACY & SECURITY")
            val privacyProtectionCount = listOf(
                settings.stripTrackingParameters,
                settings.blockThirdPartyCookies,
                settings.blockWebRtc,
                settings.reduceFingerprinting,
                settings.blockHyperlinkAuditing,
                settings.globalPrivacyControlEnabled,
                settings.doNotTrackEnabled
            ).count { it } + 1
            SettingsLinkRow(
                icon = Icons.Default.Lock,
                title = "Privacy & security",
                description = "Tracker firewall, cookies, WebRTC, fingerprinting, and browsing data.",
                trailingText = "$privacyProtectionCount on",
                onClick = { destination = SettingsDestination.Privacy }
            )

            SettingsSectionTitle("CONTENT BLOCKING")
            val activeRuleCount = AdblockManager.getFilterLists(settings)
                .filter { it.enabled }
                .sumOf { it.count } + settings.adblockCustomRules.size
            SettingsLinkRow(
                icon = Icons.Default.Security,
                title = "AdBlock & shields",
                description = "Ads, pop-ups, cosmetic filters, and filter lists.",
                trailingText = if (settings.adblockEnabled) "$activeRuleCount rules" else "Off",
                onClick = { destination = SettingsDestination.Adblock }
            )

            SettingsSectionTitle("GENERAL")
            SettingsLinkRow(
                icon = Icons.Default.Search,
                title = "Search engine",
                description = settings.searchEngine,
                onClick = { showSearchEngineDialog = true }
            )
            SettingsToggleRow(
                icon = Icons.Default.Brightness4,
                title = "Dark theme",
                description = "Use Titan's dark interface.",
                checked = settings.darkTheme,
                onCheckedChange = onToggleDarkTheme
            )

            SettingsSectionTitle("UPDATES")
            SettingsToggleRow(
                icon = Icons.Default.SystemUpdate,
                title = "Automatic update checks",
                description = "Connect to GitHub when Titan starts. This is off by default.",
                checked = settings.autoUpdateEnabled,
                onCheckedChange = onToggleAutoUpdate
            )
            Text(
                text = "Version ${updateState.currentVersion} • ${updateState.message}",
                color = if (updateState.status == UpdateStatus.Error) TitanAccentRed else TitanTextSecondary,
                fontSize = 12.sp,
                modifier = Modifier.padding(vertical = 6.dp)
            )
            Row(verticalAlignment = Alignment.CenterVertically) {
                Button(
                    onClick = onCheckForUpdates,
                    enabled = updateState.status != UpdateStatus.Checking,
                    colors = ButtonDefaults.buttonColors(containerColor = TitanSurfaceVariant)
                ) {
                    Icon(Icons.Default.Refresh, contentDescription = null, modifier = Modifier.size(16.dp))
                    Spacer(modifier = Modifier.width(6.dp))
                    Text(if (updateState.status == UpdateStatus.Checking) "Checking" else "Check now")
                }
                if (!updateState.releaseUrl.isNullOrBlank()) {
                    Spacer(modifier = Modifier.width(8.dp))
                    Button(
                        onClick = onOpenUpdate,
                        colors = ButtonDefaults.buttonColors(containerColor = TitanPrimary)
                    ) {
                        Text(if (updateState.status == UpdateStatus.UpdateAvailable) "Get update" else "Release notes")
                    }
                }
            }

            SettingsSectionTitle("ABOUT")
            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(vertical = 12.dp),
                verticalAlignment = Alignment.CenterVertically
            ) {
                Icon(Icons.Default.Info, contentDescription = null, tint = TitanTextSecondary)
                Spacer(modifier = Modifier.width(16.dp))
                Column {
                    Text("Titan Browser for Android", color = TitanTextPrimary, fontSize = 15.sp, fontWeight = FontWeight.Medium)
                    Text("Version ${updateState.currentVersion}", color = TitanTextSecondary, fontSize = 13.sp)
                }
            }
            Spacer(modifier = Modifier.height(16.dp))
        }
    }

    if (showSearchEngineDialog) {
        Dialog(onDismissRequest = { showSearchEngineDialog = false }) {
            Card(
                shape = RoundedCornerShape(16.dp),
                colors = CardDefaults.cardColors(containerColor = TitanSurface),
                modifier = Modifier.fillMaxWidth()
            ) {
                Column(modifier = Modifier.padding(20.dp)) {
                    Text("Choose a search engine", color = TitanTextPrimary, fontSize = 18.sp, fontWeight = FontWeight.Bold)
                    Spacer(modifier = Modifier.height(12.dp))
                    SearchEngine.entries.forEach { engine ->
                        SettingsLinkRow(
                            icon = if (engine.displayName == settings.searchEngine) Icons.Default.Check else Icons.Default.Search,
                            title = engine.displayName,
                            description = engine.searchUrl,
                            onClick = {
                                onUpdateSearchEngine(engine.displayName)
                                showSearchEngineDialog = false
                            }
                        )
                    }
                }
            }
        }
    }
}
