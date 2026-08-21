# Titan Browser for Android

The Android client supports Android 8.0 (API 26) and newer. It uses Kotlin, Jetpack Compose, and the installed Android System WebView.

## Supported browser flows

- Persistent tabs, active-tab restore, bookmarks, and browsing history
- System downloads with cookies, user agent, referrer, and completion notifications
- File upload through the Android document picker
- User-initiated popup tabs for sign-in and payment flows
- Per-site camera, microphone, and location prompts
- Private tabs through Android WebView multi-profile support
- Default-browser role request and HTTP/HTTPS intent handling
- Fullscreen media, desktop mode, find in page, sharing, and system Downloads access

If the installed System WebView does not support multiple profiles, Titan does not offer a false private mode. It asks the user to update System WebView instead.

## Build and verify

Use Java 17 and an Android SDK with API 36:

```powershell
.\gradlew.bat testDebugUnitTest lintDebug lintRelease assembleDebug assembleRelease
```

The debug APK is `app/build/outputs/apk/debug/app-debug.apk`.

For runtime acceptance, connect a device or emulator and run:

```powershell
adb devices -l
npm --prefix .. run verify:android-device
npm --prefix .. run verify:android-popups
```

The device verifier installs the debug APK and checks Android `VIEW` intent handling, login-cookie persistence across restart, system downloads, `<input type="file">` handoff to the document picker, camera/microphone/location prompts, session restore, and fatal runtime logs. `verify:android-popups` covers user-gesture popup tabs and automatic popup blocking through WebView devtools.

For a one-person personal install, the debug-signed APK is acceptable. Do not distribute it. Before distributing to other users, verify a signed release APK, private-cookie isolation, and the browser-role prompt on the target Android versions.

## Release signing

Set all four values as Gradle properties or environment variables:

- `TITAN_RELEASE_STORE_FILE`
- `TITAN_RELEASE_STORE_PASSWORD`
- `TITAN_RELEASE_KEY_ALIAS`
- `TITAN_RELEASE_KEY_PASSWORD`

Then run:

```powershell
.\gradlew.bat lintRelease clean assembleRelease
```

Without those values, the release build is intentionally unsigned. Never distribute the debug APK or `app-release-unsigned.apk` as a production release.
