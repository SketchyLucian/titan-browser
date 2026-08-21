package com.titan.browser.model

import kotlinx.serialization.Serializable

@Serializable
data class PersistedTab(
    val url: String,
    val title: String,
    val isDesktopMode: Boolean = false
)

@Serializable
data class BrowserSession(
    val tabs: List<PersistedTab> = emptyList(),
    val activeIndex: Int = 0
)
