package com.titan.browser.model

enum class UpdateStatus {
    Idle,
    Checking,
    UpdateAvailable,
    UpToDate,
    Error
}

data class UpdateState(
    val currentVersion: String,
    val latestVersion: String? = null,
    val releaseUrl: String? = null,
    val status: UpdateStatus = UpdateStatus.Idle,
    val message: String = "Automatic update checks are ready."
)
