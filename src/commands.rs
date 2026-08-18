use crate::browser::BrowserManager;
use crate::ipc::IpcBrowserState;
use std::sync::{Arc, Mutex};
use tauri::State;

pub type SharedBrowser = Arc<Mutex<BrowserManager>>;

#[tauri::command]
pub fn handle_ipc(message: String, state: State<'_, SharedBrowser>) {
    if let Ok(mut browser) = state.lock() {
        browser.handle_incoming_ipc(&message);
    }
}

#[tauri::command]
pub fn get_state(state: State<'_, SharedBrowser>) -> Result<IpcBrowserState, String> {
    state
        .lock()
        .map(|b| b.get_state())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_tab(url: Option<String>, state: State<'_, SharedBrowser>) -> Result<u32, String> {
    let mut browser = state.lock().map_err(|e| e.to_string())?;
    let initial = url.as_deref().unwrap_or("https://www.youtube.com");
    Ok(browser.create_tab(initial))
}

#[tauri::command]
pub fn close_tab(tab_id: u32, state: State<'_, SharedBrowser>) -> Result<(), String> {
    let mut browser = state.lock().map_err(|e| e.to_string())?;
    browser.close_tab(tab_id);
    Ok(())
}

#[tauri::command]
pub fn switch_tab(tab_id: u32, state: State<'_, SharedBrowser>) -> Result<(), String> {
    let mut browser = state.lock().map_err(|e| e.to_string())?;
    browser.switch_tab(tab_id);
    Ok(())
}

#[tauri::command]
pub fn navigate(url: String, state: State<'_, SharedBrowser>) -> Result<(), String> {
    let mut browser = state.lock().map_err(|e| e.to_string())?;
    browser.navigate(&url);
    Ok(())
}

#[tauri::command]
pub fn go_back(state: State<'_, SharedBrowser>) -> Result<(), String> {
    let mut browser = state.lock().map_err(|e| e.to_string())?;
    browser.go_back();
    Ok(())
}

#[tauri::command]
pub fn go_forward(state: State<'_, SharedBrowser>) -> Result<(), String> {
    let mut browser = state.lock().map_err(|e| e.to_string())?;
    browser.go_forward();
    Ok(())
}

#[tauri::command]
pub fn reload(state: State<'_, SharedBrowser>) -> Result<(), String> {
    let mut browser = state.lock().map_err(|e| e.to_string())?;
    browser.reload();
    Ok(())
}

#[tauri::command]
pub fn go_home(state: State<'_, SharedBrowser>) -> Result<(), String> {
    let mut browser = state.lock().map_err(|e| e.to_string())?;
    browser.go_home();
    Ok(())
}

#[tauri::command]
pub fn set_zoom(zoom: f64, state: State<'_, SharedBrowser>) -> Result<(), String> {
    let mut browser = state.lock().map_err(|e| e.to_string())?;
    browser.set_zoom(zoom);
    Ok(())
}

#[tauri::command]
pub fn toggle_bookmark(title: String, url: String, state: State<'_, SharedBrowser>) -> Result<(), String> {
    let mut browser = state.lock().map_err(|e| e.to_string())?;
    browser.toggle_bookmark(title, url);
    Ok(())
}

#[tauri::command]
pub fn remove_bookmark(url: String, state: State<'_, SharedBrowser>) -> Result<(), String> {
    let mut browser = state.lock().map_err(|e| e.to_string())?;
    browser.remove_bookmark(url);
    Ok(())
}

#[tauri::command]
pub fn toggle_module(module_id: String, enabled: bool, state: State<'_, SharedBrowser>) -> Result<(), String> {
    let mut browser = state.lock().map_err(|e| e.to_string())?;
    browser.toggle_module(module_id, enabled);
    Ok(())
}

#[tauri::command]
pub fn set_theme(theme: String, state: State<'_, SharedBrowser>) -> Result<(), String> {
    let mut browser = state.lock().map_err(|e| e.to_string())?;
    browser.set_theme(theme);
    Ok(())
}

#[tauri::command]
pub fn set_accent_color(color: String, state: State<'_, SharedBrowser>) -> Result<(), String> {
    let mut browser = state.lock().map_err(|e| e.to_string())?;
    browser.set_accent_color(color);
    Ok(())
}

#[tauri::command]
pub fn set_search_engine(engine: String, state: State<'_, SharedBrowser>) -> Result<(), String> {
    let mut browser = state.lock().map_err(|e| e.to_string())?;
    browser.set_search_engine(engine);
    Ok(())
}

#[tauri::command]
pub fn set_show_bookmarks_bar(show: bool, state: State<'_, SharedBrowser>) -> Result<(), String> {
    let mut browser = state.lock().map_err(|e| e.to_string())?;
    browser.set_show_bookmarks_bar(show);
    Ok(())
}

#[tauri::command]
pub fn open_settings(state: State<'_, SharedBrowser>) -> Result<(), String> {
    let mut browser = state.lock().map_err(|e| e.to_string())?;
    browser.create_tab("titan://settings");
    Ok(())
}

#[tauri::command]
pub fn open_themes(state: State<'_, SharedBrowser>) -> Result<(), String> {
    let mut browser = state.lock().map_err(|e| e.to_string())?;
    browser.create_tab("titan://themes");
    Ok(())
}

#[tauri::command]
pub fn window_drag(#[allow(unused_variables)] state: State<'_, SharedBrowser>) -> Result<(), String> {
    #[cfg(desktop)]
    {
        let browser = state.lock().map_err(|e| e.to_string())?;
        let _ = browser.window.start_dragging();
    }
    Ok(())
}

#[tauri::command]
pub fn window_minimize(#[allow(unused_variables)] state: State<'_, SharedBrowser>) -> Result<(), String> {
    #[cfg(desktop)]
    {
        let browser = state.lock().map_err(|e| e.to_string())?;
        let _ = browser.window.minimize();
    }
    Ok(())
}

#[tauri::command]
pub fn window_toggle_maximize(#[allow(unused_variables)] state: State<'_, SharedBrowser>) -> Result<(), String> {
    #[cfg(desktop)]
    {
        let browser = state.lock().map_err(|e| e.to_string())?;
        if let Ok(is_max) = browser.window.is_maximized() {
            if is_max {
                let _ = browser.window.unmaximize();
            } else {
                let _ = browser.window.maximize();
            }
        }
    }
    Ok(())
}

#[tauri::command]
pub fn window_close(state: State<'_, SharedBrowser>) -> Result<(), String> {
    let browser = state.lock().map_err(|e| e.to_string())?;
    let _ = browser.window.close();
    Ok(())
}
