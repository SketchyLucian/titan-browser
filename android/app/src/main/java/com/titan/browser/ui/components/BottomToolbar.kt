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
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Clear
import androidx.compose.material.icons.filled.Close
import androidx.compose.material.icons.filled.Lock
import androidx.compose.material.icons.filled.MoreVert
import androidx.compose.material.icons.filled.Refresh
import androidx.compose.material.icons.filled.Search
import androidx.compose.material.icons.filled.Security
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
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.input.TextFieldValue
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.titan.browser.ui.theme.TitanAccentGreen
import com.titan.browser.ui.theme.TitanBorder
import com.titan.browser.ui.theme.TitanPrimary
import com.titan.browser.ui.theme.TitanSurface
import com.titan.browser.ui.theme.TitanSurfaceVariant
import com.titan.browser.ui.theme.TitanTextPrimary
import com.titan.browser.ui.theme.TitanTextSecondary
import com.titan.browser.web.UrlUtils

@Composable
fun BottomToolbar(
    currentUrl: String,
    isLoading: Boolean,
    progress: Int,
    tabCount: Int,
    onNavigate: (String) -> Unit,
    onReloadOrStop: () -> Unit,
    onTabsClick: () -> Unit,
    onMenuClick: () -> Unit,
    modifier: Modifier = Modifier
) {
    val isNewTab = currentUrl == "titan://newtab" || currentUrl == "about:blank"
    var isFocused by remember { mutableStateOf(false) }

    // When unfocused, show clean domain or empty placeholder; when focused, show full URL
    val displayUrl = if (isNewTab) "" else if (!isFocused) UrlUtils.getDomain(currentUrl) else currentUrl
    var textState by remember { mutableStateOf(TextFieldValue(displayUrl)) }
    val focusRequester = remember { FocusRequester() }
    val focusManager = LocalFocusManager.current

    LaunchedEffect(currentUrl, isFocused) {
        val nextText = if (isNewTab) "" else if (!isFocused) UrlUtils.getDomain(currentUrl) else currentUrl
        textState = TextFieldValue(
            text = nextText,
            selection = if (isFocused) TextRange(0, nextText.length) else TextRange(nextText.length)
        )
    }

    Column(
        modifier = modifier
            .fillMaxWidth()
            .background(TitanSurface)
            .border(width = 0.5.dp, color = TitanBorder)
            .navigationBarsPadding()
    ) {
        // Slim Loading Progress Indicator (directly above the Firefox bar)
        AnimatedVisibility(
            visible = isLoading && progress in 1..99,
            enter = fadeIn(),
            exit = fadeOut()
        ) {
            LinearProgressIndicator(
                progress = { progress / 100f },
                modifier = Modifier
                    .fillMaxWidth()
                    .height(2.5.dp),
                color = TitanPrimary,
                trackColor = TitanSurface
            )
        }

        // Firefox Android Bottom Bar Container
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .height(58.dp)
                .padding(horizontal = 10.dp, vertical = 6.dp),
            verticalAlignment = Alignment.CenterVertically
        ) {
            // Firefox Style Pill Search / Address Bar
            Row(
                modifier = Modifier
                    .weight(1f)
                    .height(46.dp)
                    .clip(RoundedCornerShape(23.dp))
                    .background(if (isFocused) TitanSurfaceVariant else TitanSurfaceVariant.copy(alpha = 0.7f))
                    .border(
                        width = 1.dp,
                        color = if (isFocused) TitanPrimary else TitanBorder,
                        shape = RoundedCornerShape(23.dp)
                    )
                    .padding(horizontal = 12.dp),
                verticalAlignment = Alignment.CenterVertically
            ) {
                // Leading Shield / Security Icon
                val isSecure = UrlUtils.isSecure(currentUrl) && !isNewTab
                Icon(
                    imageVector = if (isFocused || isNewTab) Icons.Default.Search else Icons.Default.Security,
                    contentDescription = "Shield",
                    tint = if (!isFocused && (isSecure || !isNewTab)) TitanAccentGreen else TitanTextSecondary,
                    modifier = Modifier.size(18.dp)
                )

                Spacer(modifier = Modifier.width(8.dp))

                // Address / Search Input Field
                Box(
                    modifier = Modifier.weight(1f),
                    contentAlignment = Alignment.CenterStart
                ) {
                    if (textState.text.isEmpty() && !isFocused) {
                        Text(
                            text = "Search or enter address",
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
                            fontSize = 14.sp,
                            fontWeight = if (!isFocused && !isNewTab) FontWeight.Medium else FontWeight.Normal
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
                                    val fullUrl = if (isNewTab) "" else currentUrl
                                    textState = TextFieldValue(
                                        text = fullUrl,
                                        selection = TextRange(0, fullUrl.length)
                                    )
                                }
                            }
                    )
                }

                // Trailing Action: Clear button if typing, or Reload/Stop if viewing a site
                if (isFocused && textState.text.isNotEmpty()) {
                    IconButton(
                        onClick = { textState = TextFieldValue("") },
                        modifier = Modifier.size(28.dp)
                    ) {
                        Icon(
                            imageVector = Icons.Default.Clear,
                            contentDescription = "Clear",
                            tint = TitanTextSecondary,
                            modifier = Modifier.size(16.dp)
                        )
                    }
                } else if (!isNewTab) {
                    IconButton(
                        onClick = onReloadOrStop,
                        modifier = Modifier.size(28.dp)
                    ) {
                        Icon(
                            imageVector = if (isLoading) Icons.Default.Close else Icons.Default.Refresh,
                            contentDescription = if (isLoading) "Stop" else "Reload",
                            tint = TitanTextSecondary,
                            modifier = Modifier.size(16.dp)
                        )
                    }
                }
            }

            Spacer(modifier = Modifier.width(8.dp))

            // Firefox Style Tab Counter Badge
            Box(
                modifier = Modifier
                    .size(34.dp)
                    .clip(RoundedCornerShape(8.dp))
                    .border(width = 1.5.dp, color = TitanTextPrimary, shape = RoundedCornerShape(8.dp))
                    .clickable {
                        focusManager.clearFocus()
                        onTabsClick()
                    },
                contentAlignment = Alignment.Center
            ) {
                Text(
                    text = tabCount.toString(),
                    color = TitanTextPrimary,
                    fontSize = 12.sp,
                    fontWeight = FontWeight.Bold
                )
            }

            Spacer(modifier = Modifier.width(4.dp))

            // Three-Dots Menu Button
            Box(
                modifier = Modifier
                    .size(48.dp)
                    .clickable(
                        interactionSource = remember { MutableInteractionSource() },
                        indication = null
                    ) {
                        focusManager.clearFocus()
                        onMenuClick()
                    },
                contentAlignment = Alignment.Center
            ) {
                Icon(
                    imageVector = Icons.Default.MoreVert,
                    contentDescription = "Menu",
                    tint = TitanTextPrimary,
                    modifier = Modifier.size(22.dp)
                )
            }
        }
    }
}
