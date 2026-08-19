#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod browser;
mod drag_util;
mod ipc;
mod menu_util;
mod storage;
mod url_utils;

use browser::{BrowserManager, UserEvent};
use storage::get_app_data_dir;
use std::sync::Arc;
use tao::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoopBuilder},
    window::WindowBuilder,
};

fn main() {
    // Crucial for Windows: Set WebView2 user data folder to %LOCALAPPDATA%\TitanBrowser\webview_profile
    let webview_data_dir = get_app_data_dir().join("webview_profile");
    let _ = std::fs::create_dir_all(&webview_data_dir);
    std::env::set_var("WEBVIEW2_USER_DATA_FOLDER", &webview_data_dir);

    // Hardened Chromium & WebView2 anti-telemetry and privacy flags:
    // Disables background networking, metrics uploads, crash reporting, domain reliability pings, tracking,
    // and maps known telemetry/diagnostic hostnames to 0.0.0.0 at the socket resolver layer.
    let privacy_browser_args = [
        "--disable-features=Translate,OptimizationHints,MediaRouter,InterestFeedContentSuggestions",
        "--disable-background-networking",
        "--disable-domain-reliability",
        "--disable-component-update",
        "--disable-sync",
        "--disable-breakpad",
        "--no-report-upload",
        "--disable-client-side-phishing-detection",
        "--disable-default-apps",
        "--no-pings",
        "--host-resolver-rules=MAP *.pipe.aria.microsoft.com 0.0.0.0, MAP *.events.data.microsoft.com 0.0.0.0, MAP telemetry.microsoft.com 0.0.0.0, MAP *.telemetry.microsoft.com 0.0.0.0, MAP watson.telemetry.microsoft.com 0.0.0.0, MAP mobile.pipe.aria.microsoft.com 0.0.0.0, MAP *.google-analytics.com 0.0.0.0, MAP *.googletagmanager.com 0.0.0.0, MAP stats.g.doubleclick.net 0.0.0.0, MAP *.sentry.io 0.0.0.0, MAP *.clarity.ms 0.0.0.0, MAP *.segment.io 0.0.0.0, MAP *.mixpanel.com 0.0.0.0, MAP *.doubleclick.net 0.0.0.0, MAP *.googleadservices.com 0.0.0.0, MAP *.googlesyndication.com 0.0.0.0, MAP *.adservice.google.com 0.0.0.0, MAP *.pagead2.googlesyndication.com 0.0.0.0, MAP *.adnxs.com 0.0.0.0, MAP *.criteo.com 0.0.0.0, MAP *.outbrain.com 0.0.0.0, MAP *.taboola.com 0.0.0.0, MAP *.popads.net 0.0.0.0, MAP *.popcash.net 0.0.0.0, MAP *.propellerads.com 0.0.0.0, MAP *.adcash.com 0.0.0.0",
    ].join(" ");

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
        .with_decorations(false) // Frameless modern browser top bar
        .build(&event_loop)
        .expect("Failed to create application window");

    let window = Arc::new(window);
    let mut browser = BrowserManager::new(window.clone(), proxy);
    browser.init();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::UserEvent(user_event) => match user_event {
                UserEvent::Ipc(msg) => {
                    browser.handle_incoming_ipc(&msg);
                }
                UserEvent::PageLoadStarted { tab_id, url } => {
                    browser.on_page_load_started(tab_id, url);
                }
                UserEvent::PageLoadFinished { tab_id, url } => {
                    browser.on_page_load_finished(tab_id, url);
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
                event: WindowEvent::ScaleFactorChanged {
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
