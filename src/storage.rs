use crate::ipc::{Bookmark, BrowserModule, BrowserSettings};
use std::fs;
use std::path::PathBuf;

pub fn get_app_data_dir() -> PathBuf {
    if let Some(local_app_data) = std::env::var_os("LOCALAPPDATA") {
        PathBuf::from(local_app_data).join("TitanBrowser")
    } else {
        PathBuf::from(".").join("TitanBrowser")
    }
}

pub struct StorageManager {
    bookmarks_file: PathBuf,
    modules_file: PathBuf,
    settings_file: PathBuf,
}

impl StorageManager {
    pub fn new() -> Self {
        let dir = get_app_data_dir();
        let _ = fs::create_dir_all(&dir);
        Self {
            bookmarks_file: dir.join("bookmarks.json"),
            modules_file: dir.join("modules.json"),
            settings_file: dir.join("settings.json"),
        }
    }

    pub fn load_bookmarks(&self) -> Vec<Bookmark> {
        if let Ok(data) = fs::read_to_string(&self.bookmarks_file) {
            if let Ok(bookmarks) = serde_json::from_str::<Vec<Bookmark>>(&data) {
                return bookmarks;
            }
        }
        vec![]
    }

    pub fn save_bookmarks(&self, bookmarks: &[Bookmark]) {
        if let Ok(json) = serde_json::to_string_pretty(bookmarks) {
            let _ = fs::write(&self.bookmarks_file, json);
        }
    }

    pub fn load_modules(&self) -> Vec<BrowserModule> {
        if let Ok(data) = fs::read_to_string(&self.modules_file) {
            if let Ok(modules) = serde_json::from_str::<Vec<BrowserModule>>(&data) {
                let filtered: Vec<BrowserModule> = modules
                    .into_iter()
                    .filter(|m| m.id == "dark_reader")
                    .collect();
                if !filtered.is_empty() {
                    return filtered;
                }
            }
        }

        vec![BrowserModule {
            id: "dark_reader".into(),
            name: "Universal Dark Mode".into(),
            description: "Forces modern dark contrast theme on all bright websites.".into(),
            icon: "moon".into(),
            enabled: false,
            stats: None,
        }]
    }

    pub fn save_modules(&self, modules: &[BrowserModule]) {
        let filtered: Vec<BrowserModule> = modules
            .iter()
            .filter(|m| m.id == "dark_reader")
            .cloned()
            .collect();
        if let Ok(json) = serde_json::to_string_pretty(&filtered) {
            let _ = fs::write(&self.modules_file, json);
        }
    }

    pub fn load_settings(&self) -> BrowserSettings {
        if let Ok(data) = fs::read_to_string(&self.settings_file) {
            if let Ok(mut settings) = serde_json::from_str::<BrowserSettings>(&data) {
                let mut settings_changed = false;
                if settings.privacy_migration_version < 1 {
                    settings.auto_update_enabled = false;
                    settings.privacy_migration_version = 1;
                    settings_changed = true;
                }
                if !settings.telemetry_disabled {
                    settings_changed = true;
                }
                settings.telemetry_disabled = true;
                for domain in crate::ipc::default_blocked_domains() {
                    if !settings.blocked_domains.contains(&domain) {
                        settings.blocked_domains.push(domain);
                        settings_changed = true;
                    }
                }
                if !settings
                    .adblock_filter_lists
                    .iter()
                    .any(|list_id| list_id == "turtlecute_test")
                {
                    settings
                        .adblock_filter_lists
                        .push("turtlecute_test".to_string());
                    settings_changed = true;
                }
                if settings_changed {
                    self.save_settings(&settings);
                }
                return settings;
            }
        }
        BrowserSettings::default()
    }

    pub fn save_settings(&self, settings: &BrowserSettings) {
        if let Ok(json) = serde_json::to_string_pretty(settings) {
            let _ = fs::write(&self.settings_file, json);
        }
    }
}
