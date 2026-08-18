# 📱 Titan Browser for Android

High-performance, modern Android web browser built with **Kotlin**, **Jetpack Compose**, and hardware-accelerated **Android WebView**.

![Android](https://img.shields.io/badge/Platform-Android%208.0%2B%20(API%2026%2B)-brightgreen?logo=android)
![Kotlin](https://img.shields.io/badge/Kotlin-2.0.21-purple?logo=kotlin)
![Compose](https://img.shields.io/badge/UI-Jetpack%20Compose-blue?logo=jetpackcompose)

---

## ✨ Features

- **📺 Full YouTube & 4K 60fps Video**: Hardware-accelerated video playback with automatic fullscreen orientation handling.
- **⚡ Modern Jetpack Compose UI**: Fast 60fps UI following the Titan dark mode aesthetic (`#0F1015`, `#4E7CF6` accent, rounded cards).
- **🔍 Smart Omnibar**:
  - Automatically recognizes domains (`youtube.com`, `reddit.com`, `localhost:8080`).
  - Search engine shortcuts: `@yt` or `yt:` (YouTube), `@gh` or `gh:` (GitHub), `@ddg` (DuckDuckGo).
  - SSL security padlock indicator and animated loading indicator.
- **📑 Multi-Tab Grid Switcher**: Visual tab cards, tab badge counter, swipe to dismiss, and instant new tab creation.
- **🧭 Bottom Navigation Toolbar**: Back, Forward, Home, Bookmark Toggle, Tabs Switcher, and Overflow Menu.
- **★ Bookmarks & History**: Pre-loaded quick links with local persistent storage.
- **🖥️ Desktop Mode**: Toggle desktop user agent on any website with one tap.
- **🔍 Find in Page**: Search for text matches on any loaded web page.
- **🔒 Privacy & Storage**: Quick clear cookies, cache, and local storage.

---

## 🛠️ How to Build & Run

### 1. Open in Android Studio
1. Open **Android Studio** (Koala / Ladybug or newer).
2. Choose **Open** and select the `android/` directory:
   ```text
   c:\Users\quang\Documents\antigravity\focused-newton\android
   ```
3. Android Studio will automatically sync Gradle dependencies and build the project.
4. Connect an Android device (via USB or Wi-Fi debugging) or launch an Android Emulator.
5. Click **Run ▶ (Shift + F10)**.

### 2. Build via Command Line (Gradle)
From the `android/` directory:

```bash
# Debug APK
./gradlew assembleDebug

# Release APK
./gradlew assembleRelease
```
The output APK will be generated at:
```text
android/app/build/outputs/apk/debug/app-debug.apk
```

---

## 📂 Project Architecture

```text
android/
├── app/
│   ├── src/main/
│   │   ├── AndroidManifest.xml
│   │   ├── java/com/titan/browser/
│   │   │   ├── MainActivity.kt               # Entry point & intent handler
│   │   │   ├── TitanApp.kt                   # Application class
│   │   │   ├── model/                        # Tab, Bookmark, Settings, SearchEngine
│   │   │   ├── storage/                      # SharedPreferences & JSON storage
│   │   │   ├── viewmodel/                    # BrowserViewModel state manager
│   │   │   ├── web/                          # Custom WebView, WebChromeClient, WebViewClient, UrlUtils
│   │   │   └── ui/
│   │   │       ├── components/               # Omnibar, BottomToolbar, TabGrid, BookmarksSheet, MenuBottomSheet, FindInPageBar
│   │   │       ├── screens/                  # BrowserScreen, SettingsScreen
│   │   │       └── theme/                    # TitanTheme, Color, Type
│   │   └── res/                              # Drawables, mipmaps, strings, colors, themes
│   └── build.gradle.kts
├── build.gradle.kts
├── settings.gradle.kts
└── gradle/
```
