package com.titan.browser.ui.components

import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Clear
import androidx.compose.material.icons.filled.Close
import androidx.compose.material.icons.filled.Lock
import androidx.compose.material.icons.filled.Refresh
import androidx.compose.material.icons.filled.Search
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.LinearProgressIndicator
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.focus.onFocusChanged
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.platform.LocalFocusManager
import androidx.compose.ui.text.TextRange
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.input.TextFieldValue
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.titan.browser.ui.theme.TitanAccentGreen
import com.titan.browser.ui.theme.TitanBorder
import com.titan.browser.ui.theme.TitanGlassBorder
import com.titan.browser.ui.theme.TitanPrimary
import com.titan.browser.ui.theme.TitanSurface
import com.titan.browser.ui.theme.TitanSurfaceVariant
import com.titan.browser.ui.theme.TitanTextPrimary
import com.titan.browser.ui.theme.TitanTextSecondary
import com.titan.browser.web.UrlUtils

@Composable
fun Omnibar(
    currentUrl: String,
    isLoading: Boolean,
    progress: Int,
    onNavigate: (String) -> Unit,
    onReloadOrStop: () -> Unit,
    modifier: Modifier = Modifier
) {
    val displayUrl = if (currentUrl == "titan://newtab" || currentUrl == "about:blank") "" else currentUrl
    var isFocused by remember { mutableStateOf(false) }
    var textState by remember { mutableStateOf(TextFieldValue(displayUrl)) }
    val focusRequester = remember { FocusRequester() }
    val focusManager = LocalFocusManager.current

    // Sync external URL changes when user isn't editing
    LaunchedEffect(currentUrl) {
        if (!isFocused) {
            val syncUrl = if (currentUrl == "titan://newtab" || currentUrl == "about:blank") "" else currentUrl
            textState = TextFieldValue(syncUrl)
        }
    }

    Column(modifier = modifier.fillMaxWidth()) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .padding(horizontal = 12.dp, vertical = 8.dp)
                .height(48.dp)
                .clip(RoundedCornerShape(24.dp))
                .background(if (isFocused) TitanSurfaceVariant else TitanSurface)
                .border(
                    width = 1.dp,
                    color = if (isFocused) TitanPrimary else TitanBorder,
                    shape = RoundedCornerShape(24.dp)
                )
                .padding(horizontal = 12.dp),
            verticalAlignment = Alignment.CenterVertically
        ) {
            // Security or Search Icon
            val isSecure = UrlUtils.isSecure(currentUrl) && currentUrl != "titan://newtab" && currentUrl != "about:blank"
            Icon(
                imageVector = if (isFocused || currentUrl == "titan://newtab" || currentUrl == "about:blank") Icons.Default.Search else if (isSecure) Icons.Default.Lock else Icons.Default.Search,
                contentDescription = null,
                tint = if (!isFocused && isSecure) TitanAccentGreen else TitanTextSecondary,
                modifier = Modifier.size(18.dp)
            )


            // Address Input Field
            Box(
                modifier = Modifier
                    .weight(1f)
                    .padding(horizontal = 8.dp),
                contentAlignment = Alignment.CenterStart
            ) {
                if (textState.text.isEmpty() && !isFocused) {
                    Text(
                        text = "Search or type URL",
                        color = TitanTextSecondary,
                        fontSize = 14.sp
                    )
                }

                BasicTextField(
                    value = textState,
                    onValueChange = { textState = it },
                    singleLine = true,
                    textStyle = TextStyle(
                        color = TitanTextPrimary,
                        fontSize = 14.sp
                    ),
                    cursorBrush = SolidColor(TitanPrimary),
                    keyboardOptions = KeyboardOptions(
                        keyboardType = KeyboardType.Uri,
                        imeAction = ImeAction.Go
                    ),
                    keyboardActions = KeyboardActions(
                        onGo = {
                            if (textState.text.isNotBlank()) {
                                onNavigate(textState.text)
                            }
                            focusManager.clearFocus()
                        }
                    ),
                    modifier = Modifier
                        .fillMaxWidth()
                        .focusRequester(focusRequester)
                        .onFocusChanged { focusState ->
                            isFocused = focusState.isFocused
                            if (focusState.isFocused) {
                                // Select all text on focus
                                textState = textState.copy(
                                    selection = TextRange(0, textState.text.length)
                                )
                            }
                        }
                )
            }

            // Trailing action: Clear text if focused, or Reload/Stop if unfocused
            if (isFocused && textState.text.isNotEmpty()) {
                IconButton(
                    onClick = { textState = TextFieldValue("") },
                    modifier = Modifier.size(28.dp)
                ) {
                    Icon(
                        imageVector = Icons.Default.Clear,
                        contentDescription = "Clear",
                        tint = TitanTextSecondary,
                        modifier = Modifier.size(18.dp)
                    )
                }
            } else {
                IconButton(
                    onClick = onReloadOrStop,
                    modifier = Modifier.size(28.dp)
                ) {
                    Icon(
                        imageVector = if (isLoading) Icons.Default.Close else Icons.Default.Refresh,
                        contentDescription = if (isLoading) "Stop" else "Reload",
                        tint = TitanTextSecondary,
                        modifier = Modifier.size(18.dp)
                    )
                }
            }
        }

        // Web Loading Progress Bar
        AnimatedVisibility(
            visible = isLoading && progress in 1..99,
            enter = fadeIn(),
            exit = fadeOut()
        ) {
            LinearProgressIndicator(
                progress = { progress / 100f },
                modifier = Modifier
                    .fillMaxWidth()
                    .height(2.dp),
                color = TitanPrimary,
                trackColor = TitanSurface
            )
        }
    }
}
