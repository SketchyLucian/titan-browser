# Titan Browser

Titan is a multi-tab browser for Windows and Android. The Windows app uses Rust, Wry, and WebView2. The Android app uses Kotlin, Jetpack Compose, and Android WebView.

## Browser features

- Persistent tabs, active-tab recovery, history, bookmarks, and download records
- Direct downloads and file uploads
- OAuth/payment popups with opener support on desktop and user-gesture popup handling on Android
- Private tabs with isolated profiles
- Address-bar search, back/forward/reload, desktop-mode toggle on Android, and find in page
- Native profile-wide browsing-data deletion
- Per-site camera, microphone, and location prompts on Android
- Ad and tracker blocking with shared contract tests
- Windows and Android default-browser registration

Desktop shortcuts include `Ctrl+T`, `Ctrl+Shift+N`, `Ctrl+W`, `Ctrl+L`, `Ctrl+R`, `Ctrl+H`, `Ctrl+J`, `Alt+Left`, and `Alt+Right`. They work while page content has focus.

## Build and test

Install Rust, Node.js 22 or newer, and the platform toolchain. Then run:

```powershell
npm ci
npm run build
cargo fmt --all -- --check
cargo test
cargo clippy --all-targets -- -D warnings
cargo build --release
```

The Windows executable is `target/release/titan-browser.exe`.

Run the desktop browser-level checks after building the debug executable:

```powershell
cargo build
npm run verify:desktop-popups
npm run verify:desktop-download
npm run verify:desktop-session
```

These checks use isolated temporary profiles. They verify popup opener messaging, private-cookie isolation, native data clearing, download contents and status, content-focused shortcuts, history, and restart recovery.

Build the Windows installer with WiX 7:

```powershell
& "C:\Program Files\WiX Toolset v7.0\bin\wix.exe" build -arch x64 wix\main.wxs -o target\TitanBrowserInstaller.msi
```

The MSI registers Titan with Windows Default Apps for HTTP and HTTPS. The user must still confirm the default-browser choice in Windows Settings.

## Android

From the `android` directory:

```powershell
.\gradlew.bat testDebugUnitTest lintDebug lintRelease assembleDebug assembleRelease
```

The debug APK is `android/app/build/outputs/apk/debug/app-debug.apk`. For a one-person personal install, the debug-signed APK is acceptable. For distribution, use a signed release build.

With a phone connected through USB or Wi-Fi ADB, run:

```powershell
npm run verify:android-device
npm run verify:android-popups
```

The device verifier installs the debug APK, opens Titan through an Android `VIEW` intent, and checks login-cookie persistence, DownloadManager output, upload chooser handoff, camera/microphone/location prompts, session restore, and runtime crash logs. See [android/README.md](android/README.md) for release-signing inputs and device checks.

## Release signing

Do not distribute unsigned or debug-signed builds. A debug-signed APK is only appropriate for a personal device install.

Android release signing reads these Gradle properties or environment variables:

- `TITAN_RELEASE_STORE_FILE`
- `TITAN_RELEASE_STORE_PASSWORD`
- `TITAN_RELEASE_KEY_ALIAS`
- `TITAN_RELEASE_KEY_PASSWORD`

If they are absent, Gradle deliberately emits `app-release-unsigned.apk`.

Install an organization-owned code-signing certificate (including its private key) in the Windows `CurrentUser\My` certificate store, set `TITAN_WINDOWS_CERT_SHA1`, and run:

```powershell
.\scripts\sign-windows.ps1
```

The script signs and verifies both `titan-browser.exe` and `TitanBrowserInstaller.msi` with SHA-256 and an RFC 3161 timestamp. Set `TITAN_WINDOWS_CERT_STORE=LocalMachine` for the machine certificate store, `TITAN_WINDOWS_SIGNTOOL` for a non-standard SignTool path, or `TITAN_WINDOWS_TIMESTAMP_URL` for a different timestamp service. Certificates and private keys must remain outside the repository.
