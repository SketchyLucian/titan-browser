pub mod browser;
pub mod commands;
pub mod drag_util;
pub mod ipc;
pub mod menu_util;
pub mod storage;
pub mod url_utils;

use browser::BrowserManager;
use commands::SharedBrowser;
use std::sync::{Arc, Mutex};
use tauri::{window::WindowBuilder, Listener, Manager};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let handle = app.handle().clone();

            let builder = WindowBuilder::new(app, "main").title("Titan Browser");

            #[cfg(desktop)]
            let builder = builder
                .inner_size(1300.0, 850.0)
                .min_inner_size(600.0, 400.0)
                .decorations(false);

            let window = builder.build()?;

            let mut browser = BrowserManager::new(handle.clone(), window.clone());
            browser.init();

            let shared_browser: SharedBrowser = Arc::new(Mutex::new(browser));
            app.manage(shared_browser.clone());

            // Handle window resize events (desktop)
            #[cfg(desktop)]
            {
                let browser_for_resize = shared_browser.clone();
                let win_clone = window.clone();
                window.on_window_event(move |event| {
                    if let tauri::WindowEvent::Resized(size) = event {
                        let scale = win_clone.scale_factor().unwrap_or(1.0);
                        let logical_w = size.width as f64 / scale;
                        let logical_h = size.height as f64 / scale;
                        if let Ok(mut b) = browser_for_resize.lock() {
                            b.resize(logical_w, logical_h);
                        }
                    }
                });
            }

            // Handle page load events emitted from child webviews
            let browser_for_loads = shared_browser.clone();
            app.listen("page-load-event", move |event| {
                if let Ok(payload) = serde_json::from_str::<serde_json::Value>(event.payload()) {
                    if let (Some(tab_id), Some(url), Some(finished)) = (
                        payload.get("tab_id").and_then(|v| v.as_u64()).map(|u| u as u32),
                        payload.get("url").and_then(|v| v.as_str()),
                        payload.get("finished").and_then(|v| v.as_bool()),
                    ) {
                        if let Ok(mut b) = browser_for_loads.lock() {
                            b.on_page_load_event(tab_id, url.to_string(), finished);
                        }
                    }
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::handle_ipc,
            commands::get_state,
            commands::create_tab,
            commands::close_tab,
            commands::switch_tab,
            commands::navigate,
            commands::go_back,
            commands::go_forward,
            commands::reload,
            commands::go_home,
            commands::set_zoom,
            commands::toggle_bookmark,
            commands::remove_bookmark,
            commands::toggle_module,
            commands::set_theme,
            commands::set_accent_color,
            commands::set_search_engine,
            commands::set_show_bookmarks_bar,
            commands::open_settings,
            commands::open_themes,
            commands::window_drag,
            commands::window_minimize,
            commands::window_toggle_maximize,
            commands::window_close,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
