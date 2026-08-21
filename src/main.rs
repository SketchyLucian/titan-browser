#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod adblock_engine;
mod browser;
#[cfg(target_os = "windows")]
mod desktop_adblock;
#[cfg(target_os = "windows")]
mod desktop_data;
mod drag_util;
mod ipc;
mod menu_util;
mod privacy;
mod storage;
mod updater;
mod url_utils;

use browser::{BrowserManager, UserEvent};
use std::sync::Arc;
use storage::get_app_data_dir;
use tao::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoopBuilder},
    window::WindowBuilder,
};

fn main() {
    let initial_url = requested_launch_url(std::env::args().skip(1));
    // Crucial for Windows: Set WebView2 user data folder to %LOCALAPPDATA%\TitanBrowser\webview_profile
    let webview_data_dir = std::env::var_os("TITAN_WEBVIEW_DATA_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| get_app_data_dir().join("webview_profile"));
    let _ = std::fs::create_dir_all(&webview_data_dir);
    std::env::set_var("WEBVIEW2_USER_DATA_FOLDER", &webview_data_dir);

    let mut privacy_browser_args = privacy::webview2_browser_args();
    if let Ok(port) = std::env::var("TITAN_PROFILE_PORT") {
        if port.parse::<u16>().is_ok() {
            privacy_browser_args.push_str(&format!(" --remote-debugging-port={port}"));
        }
    }

    std::env::set_var(
        "WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS",
        &privacy_browser_args,
    );

    let event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();
    let proxy = event_loop.create_proxy();

    let window = WindowBuilder::new()
        .with_title("Titan Browser")
        .with_inner_size(LogicalSize::new(1300.0, 850.0))
        .with_min_inner_size(LogicalSize::new(600.0, 400.0))
        .with_visible(false)
        .with_decorations(false) // Frameless modern browser top bar
        .build(&event_loop)
        .expect("Failed to create application window");

    let window = Arc::new(window);
    let mut browser = BrowserManager::new(window.clone(), proxy);
    browser.init(initial_url.as_deref());
    window.set_visible(true);

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::UserEvent(user_event) => match user_event {
                UserEvent::Ipc(msg) => {
                    browser.handle_incoming_ipc(&msg);
                }
                UserEvent::UpdateCheckFinished(result) => {
                    browser.on_update_check_finished(result);
                }
                UserEvent::PageLoadStarted { tab_id, url } => {
                    browser.on_page_load_started(tab_id, url);
                }
                UserEvent::PageLoadFinished { tab_id, url } => {
                    browser.on_page_load_finished(tab_id, url);
                }
                UserEvent::OpenPopup { url } => {
                    browser.open_popup(&url);
                }
                UserEvent::DownloadStarted { url, path } => {
                    browser.on_download_started(url, path);
                }
                UserEvent::DownloadCompleted { url, path, success } => {
                    browser.on_download_completed(url, path, success);
                }
                UserEvent::AdoptPopup { tab_id } => {
                    browser.adopt_popup(tab_id);
                }
                UserEvent::Exit => {
                    *control_flow = ControlFlow::Exit;
                }
            },
            Event::WindowEvent {
                event: WindowEvent::Resized(physical_size),
                ..
            } => {
                let scale = window.scale_factor();
                let logical = physical_size.to_logical::<f64>(scale);
                browser.resize(logical.width, logical.height);
            }
            Event::WindowEvent {
                event:
                    WindowEvent::ScaleFactorChanged {
                        scale_factor,
                        new_inner_size,
                    },
                ..
            } => {
                let logical = new_inner_size.to_logical::<f64>(scale_factor);
                browser.resize(logical.width, logical.height);
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }
            _ => (),
        }
    });
}

fn requested_launch_url(args: impl IntoIterator<Item = String>) -> Option<String> {
    args.into_iter().find_map(|argument| {
        let parsed = url::Url::parse(argument.trim()).ok()?;
        matches!(parsed.scheme(), "http" | "https").then(|| parsed.into())
    })
}

#[cfg(test)]
mod tests {
    use super::requested_launch_url;

    #[test]
    fn accepts_only_web_urls_from_the_command_line() {
        assert_eq!(
            requested_launch_url(["--flag".into(), "https://example.test/path".into()]),
            Some("https://example.test/path".into())
        );
        assert_eq!(requested_launch_url(["file:///C:/secret".into()]), None);
        assert_eq!(requested_launch_url(["javascript:alert(1)".into()]), None);
    }
}
