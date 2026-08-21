package com.titan.browser

import android.Manifest
import android.app.AlertDialog
import android.app.DownloadManager
import android.app.role.RoleManager
import android.content.ActivityNotFoundException
import android.content.pm.PackageManager
import android.content.Intent
import android.net.Uri
import android.os.Build
import android.os.Bundle
import android.provider.Settings
import android.webkit.GeolocationPermissions
import android.webkit.PermissionRequest
import android.webkit.ValueCallback
import android.webkit.WebChromeClient
import android.widget.Toast
import androidx.activity.ComponentActivity
import androidx.activity.result.contract.ActivityResultContracts
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.activity.viewModels
import androidx.core.content.ContextCompat
import androidx.core.net.toUri
import com.titan.browser.ui.screens.BrowserScreen
import com.titan.browser.ui.theme.TitanBrowserTheme
import com.titan.browser.viewmodel.BrowserViewModel
import com.titan.browser.web.BrowserHostDelegate
import com.titan.browser.web.DownloadHandler
import com.titan.browser.web.DownloadRequestSpec

class MainActivity : ComponentActivity(), BrowserHostDelegate {

    private val viewModel: BrowserViewModel by viewModels()
    private var fileChooserCallback: ValueCallback<Array<Uri>>? = null
    private var pendingDownload: DownloadRequestSpec? = null
    private var pendingWebPermission: PermissionRequest? = null
    private var pendingGeolocation: Pair<String, GeolocationPermissions.Callback>? = null

    private val fileChooserLauncher = registerForActivityResult(
        ActivityResultContracts.StartActivityForResult()
    ) { result ->
        val callback = fileChooserCallback ?: return@registerForActivityResult
        fileChooserCallback = null
        callback.onReceiveValue(
            WebChromeClient.FileChooserParams.parseResult(result.resultCode, result.data)
        )
    }

    private val downloadPermissionLauncher = registerForActivityResult(
        ActivityResultContracts.RequestPermission()
    ) { granted ->
        val download = pendingDownload
        pendingDownload = null
        if (granted && download != null) {
            enqueueDownload(download)
        } else if (download != null) {
            showMessage("Storage permission is required to save this download")
        }
    }

    private val webPermissionLauncher = registerForActivityResult(
        ActivityResultContracts.RequestMultiplePermissions()
    ) { grants ->
        val request = pendingWebPermission ?: return@registerForActivityResult
        pendingWebPermission = null
        val allowedResources = request.resources.filter { resource ->
            when (resource) {
                PermissionRequest.RESOURCE_VIDEO_CAPTURE -> grants[Manifest.permission.CAMERA] == true
                PermissionRequest.RESOURCE_AUDIO_CAPTURE -> grants[Manifest.permission.RECORD_AUDIO] == true
                else -> false
            }
        }
        if (allowedResources.isEmpty()) request.deny()
        else request.grant(allowedResources.toTypedArray())
    }

    private val locationPermissionLauncher = registerForActivityResult(
        ActivityResultContracts.RequestMultiplePermissions()
    ) { grants ->
        val pending = pendingGeolocation ?: return@registerForActivityResult
        pendingGeolocation = null
        val allowed = grants[Manifest.permission.ACCESS_FINE_LOCATION] == true ||
            grants[Manifest.permission.ACCESS_COARSE_LOCATION] == true
        pending.second.invoke(pending.first, allowed, false)
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        enableEdgeToEdge()
        super.onCreate(savedInstanceState)

        viewModel.attachBrowserHost(this)
        handleIntent(intent)

        setContent {
            TitanBrowserTheme {
                BrowserScreen(viewModel = viewModel)
            }
        }
    }

    override fun onResume() {
        super.onResume()
        viewModel.onHostResume()
    }

    override fun onPause() {
        viewModel.onHostPause()
        super.onPause()
    }

    override fun onTrimMemory(level: Int) {
        super.onTrimMemory(level)
        viewModel.onTrimMemory(level)
    }

    override fun onNewIntent(intent: Intent) {
        super.onNewIntent(intent)
        setIntent(intent)
        handleIntent(intent)
    }

    override fun onDestroy() {
        fileChooserCallback?.onReceiveValue(null)
        fileChooserCallback = null
        pendingWebPermission?.deny()
        pendingWebPermission = null
        pendingGeolocation?.let { (origin, callback) -> callback.invoke(origin, false, false) }
        pendingGeolocation = null
        viewModel.detachBrowserHost(this)
        super.onDestroy()
    }

    override fun onDownloadRequested(request: DownloadRequestSpec) {
        if (request.validatedUrl() == null) {
            showMessage("Titan blocked an unsafe download address")
            return
        }

        if (Build.VERSION.SDK_INT <= Build.VERSION_CODES.P &&
            ContextCompat.checkSelfPermission(
                this,
                Manifest.permission.WRITE_EXTERNAL_STORAGE
            ) != PackageManager.PERMISSION_GRANTED
        ) {
            pendingDownload = request
            downloadPermissionLauncher.launch(Manifest.permission.WRITE_EXTERNAL_STORAGE)
            return
        }
        enqueueDownload(request)
    }

    override fun onShowBrowserMessage(message: String) {
        showMessage(message)
    }

    override fun onOpenDownloads() {
        runCatching { startActivity(Intent(DownloadManager.ACTION_VIEW_DOWNLOADS)) }
            .onFailure { showMessage("The system Downloads screen is unavailable") }
    }

    override fun onOpenDefaultBrowserSettings() {
        val intent = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
            getSystemService(RoleManager::class.java)
                .createRequestRoleIntent(RoleManager.ROLE_BROWSER)
        } else {
            Intent(Settings.ACTION_MANAGE_DEFAULT_APPS_SETTINGS)
        }
        runCatching { startActivity(intent) }
            .onFailure { showMessage("The Default Apps screen is unavailable") }
    }

    override fun onShowFileChooser(
        callback: ValueCallback<Array<Uri>>,
        params: WebChromeClient.FileChooserParams
    ): Boolean {
        fileChooserCallback?.onReceiveValue(null)
        fileChooserCallback = callback
        return try {
            fileChooserLauncher.launch(params.createIntent())
            true
        } catch (_: ActivityNotFoundException) {
            fileChooserCallback = null
            callback.onReceiveValue(null)
            showMessage("No app can select this file")
            false
        }
    }

    override fun onGeolocationPermissionRequest(
        origin: String,
        callback: GeolocationPermissions.Callback
    ) {
        val site = origin.toUri().host
        if (site.isNullOrBlank() || pendingGeolocation != null) {
            callback.invoke(origin, false, false)
            return
        }

        AlertDialog.Builder(this)
            .setTitle("Allow location?")
            .setMessage("$site wants to use your location.")
            .setPositiveButton("Allow") { _, _ ->
                val permissions = arrayOf(
                    Manifest.permission.ACCESS_FINE_LOCATION,
                    Manifest.permission.ACCESS_COARSE_LOCATION
                )
                if (permissions.any { hasPermission(it) }) {
                    callback.invoke(origin, true, false)
                } else {
                    pendingGeolocation = origin to callback
                    locationPermissionLauncher.launch(permissions)
                }
            }
            .setNegativeButton("Block") { _, _ -> callback.invoke(origin, false, false) }
            .setOnCancelListener { callback.invoke(origin, false, false) }
            .show()
    }

    override fun onWebPermissionRequest(request: PermissionRequest) {
        val site = request.origin?.host
        val supportedResources = request.resources.filter {
            it == PermissionRequest.RESOURCE_VIDEO_CAPTURE ||
                it == PermissionRequest.RESOURCE_AUDIO_CAPTURE
        }
        if (site.isNullOrBlank() || supportedResources.isEmpty() || pendingWebPermission != null) {
            request.deny()
            return
        }

        val capabilities = supportedResources.map {
            if (it == PermissionRequest.RESOURCE_VIDEO_CAPTURE) "camera" else "microphone"
        }.distinct().joinToString(" and ")
        AlertDialog.Builder(this)
            .setTitle("Allow $capabilities?")
            .setMessage("$site wants to use your $capabilities.")
            .setPositiveButton("Allow") { _, _ -> requestRuntimeWebPermissions(request) }
            .setNegativeButton("Block") { _, _ -> request.deny() }
            .setOnCancelListener { request.deny() }
            .show()
    }

    private fun requestRuntimeWebPermissions(request: PermissionRequest) {
        val runtimePermissions = request.resources.mapNotNull {
            when (it) {
                PermissionRequest.RESOURCE_VIDEO_CAPTURE -> Manifest.permission.CAMERA
                PermissionRequest.RESOURCE_AUDIO_CAPTURE -> Manifest.permission.RECORD_AUDIO
                else -> null
            }
        }.distinct()
        val missing = runtimePermissions.filterNot(::hasPermission)
        if (missing.isEmpty()) {
            request.grant(request.resources.filter {
                it == PermissionRequest.RESOURCE_VIDEO_CAPTURE ||
                    it == PermissionRequest.RESOURCE_AUDIO_CAPTURE
            }.toTypedArray())
        } else {
            pendingWebPermission = request
            webPermissionLauncher.launch(missing.toTypedArray())
        }
    }

    private fun enqueueDownload(request: DownloadRequestSpec) {
        DownloadHandler.enqueue(applicationContext, request)
            .onSuccess { showMessage("Download started") }
            .onFailure { showMessage("Download failed: ${it.message ?: "unknown error"}") }
    }

    private fun hasPermission(permission: String): Boolean =
        ContextCompat.checkSelfPermission(this, permission) == PackageManager.PERMISSION_GRANTED

    private fun showMessage(message: String) {
        Toast.makeText(this, message, Toast.LENGTH_LONG).show()
    }

    private fun handleIntent(intent: Intent?) {
        val data = intent?.dataString
        if (!data.isNullOrBlank() && (data.startsWith("http://") || data.startsWith("https://"))) {
            viewModel.openUrlFromExternalIntent(data)
        }
    }
}
