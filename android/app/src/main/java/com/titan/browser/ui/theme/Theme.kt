package com.titan.browser.ui.theme

import android.app.Activity
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.darkColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.SideEffect
import androidx.compose.ui.graphics.toArgb
import androidx.compose.ui.platform.LocalView
import androidx.core.view.WindowCompat

private val DarkColorScheme = darkColorScheme(
    primary = TitanPrimary,
    onPrimary = TitanTextPrimary,
    primaryContainer = TitanSurfaceVariant,
    onPrimaryContainer = TitanTextPrimary,
    background = TitanBackground,
    onBackground = TitanTextPrimary,
    surface = TitanSurface,
    onSurface = TitanTextPrimary,
    surfaceVariant = TitanSurfaceVariant,
    onSurfaceVariant = TitanTextSecondary,
    outline = TitanBorder
)

@Composable
fun TitanBrowserTheme(
    content: @Composable () -> Unit
) {
    val view = LocalView.current
    if (!view.isInEditMode) {
        SideEffect {
            val window = (view.context as? Activity)?.window ?: return@SideEffect
            window.statusBarColor = TitanBackground.toArgb()
            window.navigationBarColor = TitanBackground.toArgb()
            WindowCompat.getInsetsController(window, view).apply {
                isAppearanceLightStatusBars = false
                isAppearanceLightNavigationBars = false
            }
        }
    }

    MaterialTheme(
        colorScheme = DarkColorScheme,
        typography = TitanTypography,
        content = content
    )
}
