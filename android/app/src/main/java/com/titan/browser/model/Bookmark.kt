package com.titan.browser.model

import kotlinx.serialization.Serializable

@Serializable
data class Bookmark(
    val title: String,
    val url: String,
    val timestamp: Long = System.currentTimeMillis()
)
