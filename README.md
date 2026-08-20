# 🚀 Titan Browser

A modern, high-performance web browser written in **Rust** using `tao` and `wry`.

![Rust](https://img.shields.io/badge/Rust-1.97%2B-orange?logo=rust)
![Platform](https://img.shields.io/badge/Platform-Windows-blue)
![Installer](https://img.shields.io/badge/Installer-MSI%20(327%20KB)-brightgreen)
![License](https://img.shields.io/badge/License-MIT-green)

---

## ✨ Features

- **📺 YouTube Ready**: Full 4K 60fps video playback, synchronized audio, fullscreen mode, comments, and logins out of the box.
- **⚡ Native Rust Architecture**: Clean multi-threaded architecture with asynchronous IPC event routing, zero GC overhead, and 734 KB binary footprint.
- **📑 Multi-Tab Engine**: Open, close, switch, and manage independent browser tabs with dynamic titles and loading spinners.
- **🔍 Smart Omnibar**:
  - Automatically recognizes domains (e.g. `youtube.com`, `crates.io`, `localhost:3000`).
  - Google search engine queries.
  - Search prefixes: `@yt` or `yt:` (YouTube search), `@gh` or `gh:` (GitHub search), `@ddg` (DuckDuckGo).
- **★ Bookmarks Bar**: Pre-loaded with quick links (YouTube, Google, Rust Docs, GitHub, Reddit, Wikipedia) + dynamic bookmark toggling.
- **🧭 Navigation Controls**: Back, Forward, Reload (animated spinner), Home.
- **🔍 Zoom Controls**: Dynamic zoom in/out (`0.5x` to `2.5x`).
- **⌨️ Keyboard Shortcuts**:
  - `Ctrl + T`: Open new tab
  - `Ctrl + W`: Close active tab
  - `Ctrl + L`: Focus and select address bar
  - `Ctrl + R`: Reload page
  - `Alt + Left`: Back
  - `Alt + Right`: Forward

---

## 📦 Windows MSI Installer

You can install Titan Browser directly on Windows using the bundled **MSI Installer**:

- **Installer Path**: [`target/TitanBrowserInstaller.msi`](file:///c:/Users/quang/Documents/antigravity/focused-newton/target/TitanBrowserInstaller.msi) (327 KB)

### To Install:
Double-click [`TitanBrowserInstaller.msi`](file:///c:/Users/quang/Documents/antigravity/focused-newton/target/TitanBrowserInstaller.msi) or run from PowerShell:
```powershell
msiexec /i .\target\TitanBrowserInstaller.msi
```
This automatically installs the browser to `C:\Program Files\Titan Browser` and creates Start Menu and Desktop shortcuts.

---

## 🛠️ How to Run from Source

### Development Mode
```powershell
cargo run
```

### Standalone Release Binary
```powershell
cargo build --release
.\target\release\titan-browser.exe
```

### Build MSI Package
```powershell
& "C:\Program Files\WiX Toolset v7.0\bin\wix.exe" build wix\main.wxs -o target\TitanBrowserInstaller.msi
```

---

## 📱 Android Version

Titan Browser also includes a native Android client built with **Kotlin** and **Jetpack Compose** in the [`android/`](android/README.md) directory.

### Quick Start for Android:
1. Open [`android/`](android/) in **Android Studio**.
2. Run on device or build via Gradle:
```bash
cd android
./gradlew assembleDebug
```
See the [Android README](android/README.md) for full architectural and feature details.

---

## Cross-platform Parity

Shared browser behavior should have one contract that both desktop and Android verify. Adblock URL decisions are covered by [`shared/adblock_contract.json`](shared/adblock_contract.json).

Run these checks before merging platform changes:

```powershell
cargo test
cd android
.\gradlew.bat testDebugUnitTest assembleDebug
```

GitHub Actions also runs the same desktop and Android checks on pushes and pull requests.

