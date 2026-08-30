use webview2_com::{ClearBrowsingDataCompletedHandler, Microsoft::Web::WebView2::Win32::*};
use windows_core::Interface;
use wry::{WebView, WebViewExtWindows};

fn selected_data_kinds(
    cookies: bool,
    cache: bool,
    local_storage: bool,
) -> COREWEBVIEW2_BROWSING_DATA_KINDS {
    let mut kinds = COREWEBVIEW2_BROWSING_DATA_KINDS(0);
    if cookies {
        kinds |= COREWEBVIEW2_BROWSING_DATA_KINDS_COOKIES;
    }
    if cache {
        kinds |= COREWEBVIEW2_BROWSING_DATA_KINDS_DISK_CACHE;
    }
    if local_storage {
        kinds |= COREWEBVIEW2_BROWSING_DATA_KINDS_ALL_DOM_STORAGE;
        kinds |= COREWEBVIEW2_BROWSING_DATA_KINDS_SERVICE_WORKERS;
    }
    kinds
}

pub fn clear_browsing_data(
    webview: &WebView,
    cookies: bool,
    cache: bool,
    local_storage: bool,
) -> Result<(), String> {
    let kinds = selected_data_kinds(cookies, cache, local_storage);
    if kinds.0 == 0 {
        return Ok(());
    }

    let controller = webview.controller();
    let core = unsafe { controller.CoreWebView2() }.map_err(|error| error.to_string())?;
    let profile = core
        .cast::<ICoreWebView2_13>()
        .and_then(|core| unsafe { core.Profile() })
        .and_then(|profile| profile.cast::<ICoreWebView2Profile2>())
        .map_err(|error| error.to_string())?;
    let handler = ClearBrowsingDataCompletedHandler::create(Box::new(|_| Ok(())));
    unsafe { profile.ClearBrowsingData(kinds, &handler) }.map_err(|error| error.to_string())
}

pub fn install_browser_extension(
    webview: &WebView,
    extension_folder_path: &str,
) -> Result<(), String> {
    let controller = webview.controller();
    let core = unsafe { controller.CoreWebView2() }.map_err(|error| error.to_string())?;
    let profile = core
        .cast::<ICoreWebView2_13>()
        .and_then(|core| unsafe { core.Profile() })
        .and_then(|profile| profile.cast::<ICoreWebView2Profile7>())
        .map_err(|error| error.to_string())?;

    let path_wide = windows_core::HSTRING::from(extension_folder_path);
    let handler = webview2_com::ProfileAddBrowserExtensionCompletedHandler::create(Box::new(
        |result, _ext| {
            if let Err(e) = result {
                eprintln!("AddBrowserExtension error: {e}");
            } else {
                println!("Extension successfully added to WebView2 profile dynamically!");
            }
            Ok(())
        },
    ));

    unsafe {
        profile.AddBrowserExtension(
            windows_core::PCWSTR(path_wide.as_ptr()),
            &handler,
        )
    }
    .map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::selected_data_kinds;
    use webview2_com::Microsoft::Web::WebView2::Win32::{
        COREWEBVIEW2_BROWSING_DATA_KINDS_ALL_DOM_STORAGE, COREWEBVIEW2_BROWSING_DATA_KINDS_COOKIES,
        COREWEBVIEW2_BROWSING_DATA_KINDS_DISK_CACHE,
        COREWEBVIEW2_BROWSING_DATA_KINDS_SERVICE_WORKERS,
    };

    #[test]
    fn maps_user_choices_to_profile_data_kinds() {
        assert_eq!(selected_data_kinds(false, false, false).0, 0);
        assert_eq!(
            selected_data_kinds(true, true, true).0,
            COREWEBVIEW2_BROWSING_DATA_KINDS_COOKIES.0
                | COREWEBVIEW2_BROWSING_DATA_KINDS_DISK_CACHE.0
                | COREWEBVIEW2_BROWSING_DATA_KINDS_ALL_DOM_STORAGE.0
                | COREWEBVIEW2_BROWSING_DATA_KINDS_SERVICE_WORKERS.0
        );
    }
}
