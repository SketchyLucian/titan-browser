package com.titan.browser.model

import kotlinx.serialization.Serializable

@Serializable
data class HistoryEntry(
    val title: String,
    val url: String,
    val lastVisitedMs: Long = System.currentTimeMillis(),
    val visitCount: Int = 1
)
