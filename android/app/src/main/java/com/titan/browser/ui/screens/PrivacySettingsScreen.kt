package com.titan.browser.ui.screens

import android.widget.Toast
import androidx.compose.foundation.background
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
import androidx.compose.material.icons.filled.Delete
import androidx.compose.material.icons.filled.Lock
import androidx.compose.material.icons.filled.Security
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.titan.browser.model.BrowserSettings
import com.titan.browser.ui.theme.TitanAccentRed
import com.titan.browser.ui.theme.TitanPrimary
import com.titan.browser.ui.theme.TitanSurface
import com.titan.browser.ui.theme.TitanTextPrimary
import com.titan.browser.ui.theme.TitanTextSecondary

@Composable
fun PrivacySettingsScreen(
    settings: BrowserSettings,
    onSettingsChange: (BrowserSettings) -> Unit,
    onClearBrowsingData: () -> Unit,
    onBack: () -> Unit,
    modifier: Modifier = Modifier
) {
    val context = LocalContext.current
    SettingsPageScaffold(title = "Privacy & security", onBack = onBack, modifier = modifier) {
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
                Text(
                    text = "Tracker request firewall",
                    color = TitanTextPrimary,
                    fontSize = 16.sp,
                    fontWeight = FontWeight.Bold
                )
                Text(
                    text = "Titan blocks known analytics, crash-reporting, and telemetry hosts before the page can connect.",
                    color = TitanTextSecondary,
                    fontSize = 12.sp
                )
            }
            Text(
                text = "Always on",
                color = TitanPrimary,
                fontSize = 12.sp,
                fontWeight = FontWeight.Bold
            )
        }

        SettingsSectionTitle("TRACKING PROTECTION")
        SettingsToggleRow(
            icon = Icons.Default.Lock,
            title = "Strip tracking parameters",
            description = "Remove UTM, fbclid, gclid, and similar tags from links.",
            checked = settings.stripTrackingParameters,
            onCheckedChange = { onSettingsChange(settings.copy(stripTrackingParameters = it)) }
        )
        SettingsToggleRow(
            icon = Icons.Default.Lock,
            title = "Block third-party cookies",
            description = "Sites can use first-party cookies. Embedded third parties cannot set cookies.",
            checked = settings.blockThirdPartyCookies,
            onCheckedChange = { onSettingsChange(settings.copy(blockThirdPartyCookies = it)) }
        )
        SettingsToggleRow(
            icon = Icons.Default.Lock,
            title = "Block WebRTC",
            description = "Stop peer connections that can reveal network addresses.",
            checked = settings.blockWebRtc,
            onCheckedChange = { onSettingsChange(settings.copy(blockWebRtc = it)) }
        )
        SettingsToggleRow(
            icon = Icons.Default.Lock,
            title = "Reduce fingerprinting",
            description = "Use a generic user agent and reduce exposed device details.",
            checked = settings.reduceFingerprinting,
            onCheckedChange = { onSettingsChange(settings.copy(reduceFingerprinting = it)) }
        )
        SettingsToggleRow(
            icon = Icons.Default.Lock,
            title = "Block link auditing",
            description = "Remove hidden ping targets from links before you open them.",
            checked = settings.blockHyperlinkAuditing,
            onCheckedChange = { onSettingsChange(settings.copy(blockHyperlinkAuditing = it)) }
        )

        SettingsSectionTitle("PRIVACY SIGNALS")
        SettingsToggleRow(
            icon = Icons.Default.Security,
            title = "Global Privacy Control",
            description = "Send the Sec-GPC: 1 signal and expose it to page scripts.",
            checked = settings.globalPrivacyControlEnabled,
            onCheckedChange = { onSettingsChange(settings.copy(globalPrivacyControlEnabled = it)) }
        )
        SettingsToggleRow(
            icon = Icons.Default.Security,
            title = "Do Not Track",
            description = "Send the DNT: 1 signal and expose it to page scripts.",
            checked = settings.doNotTrackEnabled,
            onCheckedChange = { onSettingsChange(settings.copy(doNotTrackEnabled = it)) }
        )

        SettingsSectionTitle("SITE PERMISSIONS")
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .padding(vertical = 10.dp),
            verticalAlignment = Alignment.CenterVertically
        ) {
            Icon(Icons.Default.Lock, contentDescription = null, tint = TitanPrimary)
            Spacer(modifier = Modifier.width(16.dp))
            Column(modifier = Modifier.weight(1f)) {
                Text(
                    text = "Location, camera, and microphone",
                    color = TitanTextPrimary,
                    fontSize = 15.sp,
                    fontWeight = FontWeight.Medium
                )
                Text(
                    text = "Titan denies these WebView permission requests.",
                    color = TitanTextSecondary,
                    fontSize = 12.sp
                )
            }
            Text("Blocked", color = TitanPrimary, fontSize = 12.sp, fontWeight = FontWeight.Bold)
        }

        SettingsSectionTitle("BROWSING DATA")
        Text(
            text = "Delete cookies, cache, form data, history, and web storage from this app.",
            color = TitanTextSecondary,
            fontSize = 13.sp
        )
        Spacer(modifier = Modifier.height(10.dp))
        Button(
            onClick = {
                onClearBrowsingData()
                Toast.makeText(context, "Browsing data cleared", Toast.LENGTH_SHORT).show()
            },
            colors = ButtonDefaults.buttonColors(containerColor = TitanAccentRed),
            modifier = Modifier.fillMaxWidth()
        ) {
            Icon(Icons.Default.Delete, contentDescription = null, modifier = Modifier.size(18.dp))
            Spacer(modifier = Modifier.width(8.dp))
            Text("Clear browsing data")
        }
        Spacer(modifier = Modifier.height(16.dp))
    }
}
