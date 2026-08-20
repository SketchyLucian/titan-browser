use serde::{Deserialize, Serialize};

const RELEASE_API_URL: &str =
    "https://api.github.com/repos/SketchyLucian/titan-browser/releases/latest";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateInfo {
    pub version: String,
    pub release_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UpdateStatus {
    Idle,
    Checking,
    UpdateAvailable,
    UpToDate,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateState {
    pub current_version: String,
    pub latest_version: Option<String>,
    pub release_url: Option<String>,
    pub status: UpdateStatus,
    pub message: String,
}

impl Default for UpdateState {
    fn default() -> Self {
        Self {
            current_version: env!("CARGO_PKG_VERSION").to_string(),
            latest_version: None,
            release_url: None,
            status: UpdateStatus::Idle,
            message: "Automatic update checks are ready.".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UpdateCheckResult {
    Available(UpdateInfo),
    UpToDate(UpdateInfo),
    Failed(String),
}

#[derive(Debug, Deserialize)]
struct GitHubRelease {
    tag_name: String,
    html_url: String,
}

pub fn check_for_updates(current_version: &str) -> UpdateCheckResult {
    match fetch_latest_release() {
        Ok(info) => {
            if is_newer_version(&info.version, current_version) {
                UpdateCheckResult::Available(info)
            } else {
                UpdateCheckResult::UpToDate(info)
            }
        }
        Err(err) => UpdateCheckResult::Failed(err),
    }
}

fn fetch_latest_release() -> Result<UpdateInfo, String> {
    let release: GitHubRelease = ureq::get(RELEASE_API_URL)
        .set("Accept", "application/vnd.github+json")
        .set("User-Agent", "Titan-Browser-Updater")
        .call()
        .map_err(|err| format!("Could not reach update service: {err}"))?
        .into_json()
        .map_err(|err| format!("Could not read update metadata: {err}"))?;

    Ok(UpdateInfo {
        version: release.tag_name,
        release_url: release.html_url,
    })
}

fn is_newer_version(candidate: &str, current: &str) -> bool {
    let candidate_parts = parse_version(candidate);
    let current_parts = parse_version(current);

    for index in 0..candidate_parts.len().max(current_parts.len()) {
        let candidate_part = *candidate_parts.get(index).unwrap_or(&0);
        let current_part = *current_parts.get(index).unwrap_or(&0);
        if candidate_part > current_part {
            return true;
        }
        if candidate_part < current_part {
            return false;
        }
    }

    false
}

fn parse_version(version: &str) -> Vec<u64> {
    version
        .trim()
        .trim_start_matches('v')
        .split('.')
        .map(|part| {
            part.chars()
                .take_while(|ch| ch.is_ascii_digit())
                .collect::<String>()
                .parse::<u64>()
                .unwrap_or(0)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::is_newer_version;

    #[test]
    fn compares_semver_tags() {
        assert!(is_newer_version("v0.3.1", "0.3.0"));
        assert!(is_newer_version("1.0.0", "0.9.9"));
        assert!(!is_newer_version("v0.3.0", "0.3.0"));
        assert!(!is_newer_version("0.2.9", "0.3.0"));
    }
}
