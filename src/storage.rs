use crate::ipc::{
    Bookmark, BrowserModule, BrowserSession, BrowserSettings, DownloadRecord, HistoryEntry,
};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fs;
use std::fs::OpenOptions;
use std::io::{self, Write};
use std::path::PathBuf;

pub fn get_app_data_dir() -> PathBuf {
    if let Some(override_dir) = std::env::var_os("TITAN_APP_DATA_DIR") {
        PathBuf::from(override_dir)
    } else if let Some(local_app_data) = std::env::var_os("LOCALAPPDATA") {
        PathBuf::from(local_app_data).join("TitanBrowser")
    } else {
        PathBuf::from(".").join("TitanBrowser")
    }
}

pub struct StorageManager {
    bookmarks_file: PathBuf,
    modules_file: PathBuf,
    settings_file: PathBuf,
    history_file: PathBuf,
    session_file: PathBuf,
    downloads_file: PathBuf,
}

impl StorageManager {
    pub fn new() -> Self {
        let dir = get_app_data_dir();
        let _ = fs::create_dir_all(&dir);
        Self::for_directory(dir)
    }

    fn for_directory(dir: PathBuf) -> Self {
        Self {
            bookmarks_file: dir.join("bookmarks.json"),
            modules_file: dir.join("modules.json"),
            settings_file: dir.join("settings.json"),
            history_file: dir.join("history.json"),
            session_file: dir.join("session.json"),
            downloads_file: dir.join("downloads.json"),
        }
    }

    fn load_json<T: DeserializeOwned>(&self, path: &PathBuf) -> Option<T> {
        let candidates = [
            path.clone(),
            path.with_extension("json.tmp"),
            path.with_extension("json.bak"),
        ];

        for candidate in candidates {
            match fs::read_to_string(&candidate) {
                Ok(data) => match serde_json::from_str(&data) {
                    Ok(value) => {
                        if candidate != *path {
                            eprintln!("Recovered {} from {}", path.display(), candidate.display());
                        }
                        return Some(value);
                    }
                    Err(error) => {
                        eprintln!("Could not parse {}: {error}", candidate.display());
                    }
                },
                Err(error) if error.kind() == io::ErrorKind::NotFound => {}
                Err(error) => {
                    eprintln!("Could not read {}: {error}", candidate.display());
                }
            }
        }

        None
    }

    fn save_json<T: Serialize + ?Sized>(&self, path: &PathBuf, value: &T) {
        let result = (|| -> io::Result<()> {
            let json = serde_json::to_vec_pretty(value).map_err(io::Error::other)?;
            let temporary = path.with_extension("json.tmp");
            let backup = path.with_extension("json.bak");
            let mut output = OpenOptions::new()
                .create(true)
                .truncate(true)
                .write(true)
                .open(&temporary)?;
            output.write_all(&json)?;
            output.sync_all()?;
            drop(output);

            if path.exists() {
                if backup.exists() {
                    fs::remove_file(&backup)?;
                }
                fs::rename(path, &backup)?;
            }

            if let Err(error) = fs::rename(&temporary, path) {
                if backup.exists() && !path.exists() {
                    let _ = fs::rename(&backup, path);
                }
                return Err(error);
            }

            if backup.exists() {
                fs::remove_file(backup)?;
            }
            Ok(())
        })();

        if let Err(error) = result {
            eprintln!("Could not save {}: {error}", path.display());
        }
    }

    pub fn load_bookmarks(&self) -> Vec<Bookmark> {
        self.load_json(&self.bookmarks_file).unwrap_or_default()
    }

    pub fn save_bookmarks(&self, bookmarks: &[Bookmark]) {
        self.save_json(&self.bookmarks_file, bookmarks);
    }

    pub fn load_modules(&self) -> Vec<BrowserModule> {
        if let Some(modules) = self.load_json::<Vec<BrowserModule>>(&self.modules_file) {
            let filtered: Vec<BrowserModule> = modules
                .into_iter()
                .filter(|m| m.id == "dark_reader")
                .collect();
            if !filtered.is_empty() {
                return filtered;
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
        self.save_json(&self.modules_file, &filtered);
    }

    pub fn load_settings(&self) -> BrowserSettings {
        if let Some(mut settings) = self.load_json::<BrowserSettings>(&self.settings_file) {
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
        BrowserSettings::default()
    }

    pub fn save_settings(&self, settings: &BrowserSettings) {
        self.save_json(&self.settings_file, settings);
    }

    pub fn load_history(&self) -> Vec<HistoryEntry> {
        self.load_json(&self.history_file).unwrap_or_default()
    }

    pub fn save_history(&self, history: &[HistoryEntry]) {
        self.save_json(&self.history_file, history);
    }

    pub fn load_session(&self) -> BrowserSession {
        self.load_json(&self.session_file).unwrap_or_default()
    }

    pub fn save_session(&self, session: &BrowserSession) {
        self.save_json(&self.session_file, session);
    }

    pub fn load_downloads(&self) -> Vec<DownloadRecord> {
        self.load_json(&self.downloads_file).unwrap_or_default()
    }

    pub fn save_downloads(&self, downloads: &[DownloadRecord]) {
        self.save_json(&self.downloads_file, downloads);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn isolated_storage() -> (StorageManager, PathBuf) {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock")
            .as_nanos();
        let directory = std::env::temp_dir().join(format!(
            "titan-storage-test-{}-{unique}",
            std::process::id()
        ));
        fs::create_dir_all(&directory).expect("create test directory");
        (StorageManager::for_directory(directory.clone()), directory)
    }

    #[test]
    fn replaces_json_without_leaving_a_backup() {
        let (storage, directory) = isolated_storage();
        storage.save_history(&[HistoryEntry {
            title: "First".into(),
            url: "https://example.com/first".into(),
            last_visited_ms: 1,
            visit_count: 1,
        }]);
        storage.save_history(&[HistoryEntry {
            title: "Second".into(),
            url: "https://example.com/second".into(),
            last_visited_ms: 2,
            visit_count: 1,
        }]);

        let history = storage.load_history();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].title, "Second");
        assert!(!storage.history_file.with_extension("json.bak").exists());
        fs::remove_dir_all(directory).expect("remove test directory");
    }

    #[test]
    fn recovers_json_from_an_interrupted_replacement() {
        let (storage, directory) = isolated_storage();
        storage.save_history(&[HistoryEntry {
            title: "Recover me".into(),
            url: "https://example.com/recover".into(),
            last_visited_ms: 3,
            visit_count: 1,
        }]);
        fs::rename(
            &storage.history_file,
            storage.history_file.with_extension("json.bak"),
        )
        .expect("simulate interrupted replacement");

        let history = storage.load_history();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].title, "Recover me");
        fs::remove_dir_all(directory).expect("remove test directory");
    }
}
