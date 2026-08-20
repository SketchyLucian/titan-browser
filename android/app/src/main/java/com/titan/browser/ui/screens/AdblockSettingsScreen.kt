package com.titan.browser.ui.screens

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Refresh
import androidx.compose.material.icons.filled.Security
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.Icon
import androidx.compose.material3.Switch
import androidx.compose.material3.SwitchDefaults
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.titan.browser.model.BrowserSettings
import com.titan.browser.ui.theme.TitanBorder
import com.titan.browser.ui.theme.TitanPrimary
import com.titan.browser.ui.theme.TitanSurface
import com.titan.browser.ui.theme.TitanSurfaceVariant
import com.titan.browser.ui.theme.TitanTextPrimary
import com.titan.browser.ui.theme.TitanTextSecondary
import com.titan.browser.web.AdblockManager

@Composable
fun AdblockSettingsScreen(
    settings: BrowserSettings,
    onToggleAdblock: (Boolean) -> Unit,
    onToggleBlockVideoAds: (Boolean) -> Unit,
    onToggleCosmeticFiltering: (Boolean) -> Unit,
    onToggleBlockPopups: (Boolean) -> Unit,
    onToggleAggressiveAdblock: (Boolean) -> Unit,
    onToggleFilterList: (String, Boolean) -> Unit,
    onToggleAutoUpdateFilterLists: (Boolean) -> Unit,
    onRefreshFilterLists: () -> Unit,
    onBack: () -> Unit,
    modifier: Modifier = Modifier
) {
    val filterLists = AdblockManager.getFilterLists(settings)
    val activeRuleCount = filterLists.filter { it.enabled }.sumOf { it.count } + settings.adblockCustomRules.size

    SettingsPageScaffold(title = "AdBlock & shields", onBack = onBack, modifier = modifier) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .background(TitanSurface, RoundedCornerShape(14.dp))
                .padding(16.dp),
            verticalAlignment = Alignment.CenterVertically
        ) {
            Icon(
                imageVector = Icons.Default.Security,
                contentDescription = null,
                tint = TitanPrimary,
                modifier = Modifier.size(32.dp)
            )
            Spacer(modifier = Modifier.width(14.dp))
            Column(modifier = Modifier.weight(1f)) {
                Text("Titan Shield", color = TitanTextPrimary, fontSize = 16.sp, fontWeight = FontWeight.Bold)
                Text(
                    text = if (settings.adblockEnabled) "$activeRuleCount active rules" else "Protection is off",
                    color = TitanTextSecondary,
                    fontSize = 12.sp
                )
            }
            Switch(
                checked = settings.adblockEnabled,
                onCheckedChange = onToggleAdblock,
                colors = SwitchDefaults.colors(
                    checkedThumbColor = TitanPrimary,
                    checkedTrackColor = TitanSurfaceVariant,
                    uncheckedTrackColor = TitanBorder
                )
            )
        }

        SettingsSectionTitle("SHIELDS")
        SettingsToggleRow(
            icon = Icons.Default.Security,
            title = "Block video ads",
            description = "Skip or hide supported video ads.",
            checked = settings.blockVideoAds,
            enabled = settings.adblockEnabled,
            onCheckedChange = onToggleBlockVideoAds
        )
        SettingsToggleRow(
            icon = Icons.Default.Security,
            title = "Hide ad elements",
            description = "Remove blocked banners, overlays, and empty ad boxes.",
            checked = settings.cosmeticFiltering,
            enabled = settings.adblockEnabled,
            onCheckedChange = onToggleCosmeticFiltering
        )
        SettingsToggleRow(
            icon = Icons.Default.Security,
            title = "Block pop-ups",
            description = "Stop unrequested pop-ups and pop-under windows.",
            checked = settings.blockPopups,
            enabled = settings.adblockEnabled,
            onCheckedChange = onToggleBlockPopups
        )
        SettingsToggleRow(
            icon = Icons.Default.Security,
            title = "Aggressive filtering",
            description = "Use strict heuristics in addition to filter-list rules.",
            checked = settings.aggressiveMode,
            enabled = settings.adblockEnabled,
            onCheckedChange = onToggleAggressiveAdblock
        )

        SettingsSectionTitle("FILTER LISTS")
        filterLists.forEach { list ->
            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(vertical = 9.dp),
                verticalAlignment = Alignment.CenterVertically
            ) {
                Column(modifier = Modifier.weight(1f)) {
                    Text(list.name, color = TitanTextPrimary, fontSize = 14.sp, fontWeight = FontWeight.Medium)
                    Text("${list.count} rules", color = TitanTextSecondary, fontSize = 12.sp)
                }
                Switch(
                    checked = list.enabled,
                    enabled = settings.adblockEnabled,
                    onCheckedChange = { onToggleFilterList(list.id, it) },
                    colors = SwitchDefaults.colors(
                        checkedThumbColor = TitanPrimary,
                        checkedTrackColor = TitanSurfaceVariant,
                        uncheckedTrackColor = TitanBorder
                    )
                )
            }
        }

        SettingsSectionTitle("LIST UPDATES")
        SettingsToggleRow(
            icon = Icons.Default.Refresh,
            title = "Automatic filter-list updates",
            description = "Connect to the selected list providers when Titan starts.",
            checked = settings.autoUpdateFilterLists,
            onCheckedChange = onToggleAutoUpdateFilterLists
        )
        Button(
            onClick = onRefreshFilterLists,
            colors = ButtonDefaults.buttonColors(containerColor = TitanSurfaceVariant),
            modifier = Modifier.fillMaxWidth()
        ) {
            Icon(Icons.Default.Refresh, contentDescription = null, modifier = Modifier.size(18.dp))
            Spacer(modifier = Modifier.width(8.dp))
            Text("Update lists now")
        }
    }
}
