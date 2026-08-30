use crate::storage::get_app_data_dir;
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{Cursor, Read};
use std::path::{Path, PathBuf};
use zip::ZipArchive;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExtensionInfo {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub icon: Option<String>,
    pub enabled: bool,
    pub source: String,
    pub path: String,
    pub manifest_version: u32,
    pub options_page: Option<String>,
    pub popup_page: Option<String>,
    pub homepage_url: Option<String>,
}

pub fn get_extensions_dir() -> PathBuf {
    let dir = get_app_data_dir().join("extensions");
    let _ = fs::create_dir_all(&dir);
    dir
}

pub fn normalize_extension_id(input: &str) -> Option<(String, String)> {
    let input = input.trim();
    if input.is_empty() {
        return None;
    }

    // Direct Chrome Web Store URL: https://chromewebstore.google.com/detail/<name>/<id>
    if input.contains("chromewebstore.google.com") || input.contains("chrome.google.com/webstore") {
        if let Ok(url) = url::Url::parse(input) {
            let segments: Vec<&str> = url.path_segments()?.collect();
            if let Some(id) = segments.iter().rev().find(|s| is_valid_extension_id(s)) {
                return Some((id.to_string(), "chrome".to_string()));
            }
        }
    }

    // Direct Edge Add-ons URL: https://microsoftedge.microsoft.com/addons/detail/<name>/<id>
    if input.contains("microsoftedge.microsoft.com/addons") {
        if let Ok(url) = url::Url::parse(input) {
            let segments: Vec<&str> = url.path_segments()?.collect();
            if let Some(id) = segments.iter().rev().find(|s| is_valid_extension_id(s)) {
                return Some((id.to_string(), "edge".to_string()));
            }
        }
    }

    // Direct 32-character extension ID
    let clean_id = input.trim_matches('/').to_ascii_lowercase();
    if is_valid_extension_id(&clean_id) {
        return Some((clean_id, "chrome".to_string()));
    }

    None
}

pub fn is_valid_extension_id(id: &str) -> bool {
    let id = id.trim();
    id.len() == 32 && id.chars().all(|c| c.is_ascii_alphabetic())
}

pub fn extract_crx_zip_payload(data: &[u8]) -> Result<&[u8], String> {
    if data.len() < 4 {
        return Err("File too small to be a valid CRX or ZIP archive".into());
    }

    // Check if directly standard ZIP
    if data.starts_with(b"PK\x03\x04") {
        return Ok(data);
    }

    // Check if CRX format
    if data.starts_with(b"Cr24") {
        if data.len() < 12 {
            return Err("Invalid CRX header length".into());
        }
        let version = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);
        if version == 3 {
            let header_size = u32::from_le_bytes([data[8], data[9], data[10], data[11]]) as usize;
            let offset = 12 + header_size;
            if offset <= data.len() {
                return Ok(&data[offset..]);
            } else {
                return Err("CRX3 header length exceeds payload size".into());
            }
        } else if version == 2 {
            if data.len() < 16 {
                return Err("Invalid CRX2 header length".into());
            }
            let pub_key_len = u32::from_le_bytes([data[8], data[9], data[10], data[11]]) as usize;
            let sig_len = u32::from_le_bytes([data[12], data[13], data[14], data[15]]) as usize;
            let offset = 16 + pub_key_len + sig_len;
            if offset <= data.len() {
                return Ok(&data[offset..]);
            } else {
                return Err("CRX2 header length exceeds payload size".into());
            }
        }
    }

    // Search for ZIP magic signature in data
    if let Some(pos) = data.windows(4).position(|w| w == b"PK\x03\x04") {
        return Ok(&data[pos..]);
    }

    Err("Could not find ZIP payload in CRX package".into())
}

pub fn unpack_extension_zip(zip_bytes: &[u8], dest_dir: &Path) -> Result<(), String> {
    let cursor = Cursor::new(zip_bytes);
    let mut archive = ZipArchive::new(cursor).map_err(|e| format!("Failed to read ZIP: {e}"))?;

    let _ = fs::create_dir_all(dest_dir);

    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| format!("Failed to read zip entry {i}: {e}"))?;
        let raw_path = match file.enclosed_name() {
            Some(path) => path.to_owned(),
            None => continue, // Prevent ZipSlip directory traversal
        };

        let outpath = dest_dir.join(raw_path);

        if file.is_dir() {
            let _ = fs::create_dir_all(&outpath);
        } else {
            if let Some(parent) = outpath.parent() {
                let _ = fs::create_dir_all(parent);
            }
            let mut outfile =
                File::create(&outpath).map_err(|e| format!("Failed to create file: {e}"))?;
            std::io::copy(&mut file, &mut outfile)
                .map_err(|e| format!("Failed to extract file: {e}"))?;
        }
    }

    Ok(())
}

fn resolve_manifest_message(text: &str, dir: &Path) -> String {
    if !text.starts_with("__MSG_") || !text.ends_with("__") {
        return text.to_string();
    }

    let key = &text[6..text.len() - 2];
    let locales = ["en", "en_US", "en_GB", "_locales/en", "vi", "default"];

    for locale in locales {
        let locale_file = dir
            .join("_locales")
            .join(locale)
            .join("messages.json");
        if locale_file.exists() {
            if let Ok(content) = fs::read_to_string(&locale_file) {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(msg) = json.get(key).and_then(|v| v.get("message")).and_then(|m| m.as_str()) {
                        return msg.to_string();
                    }
                }
            }
        }
    }

    key.to_string()
}

pub fn parse_extension_manifest(
    dir: &Path,
    id: &str,
    source: &str,
    enabled: bool,
) -> Result<ExtensionInfo, String> {
    let manifest_path = dir.join("manifest.json");
    if !manifest_path.exists() {
        return Err(format!("manifest.json not found in {}", dir.display()));
    }

    let manifest_raw = fs::read_to_string(&manifest_path)
        .map_err(|e| format!("Failed to read manifest.json: {e}"))?;
    let json: serde_json::Value = serde_json::from_str(&manifest_raw)
        .map_err(|e| format!("Failed to parse manifest.json: {e}"))?;

    let raw_name = json["name"].as_str().unwrap_or("Unnamed Extension");
    let name = resolve_manifest_message(raw_name, dir);

    let raw_desc = json["description"].as_str().unwrap_or("");
    let description = resolve_manifest_message(raw_desc, dir);

    let version = json["version"].as_str().unwrap_or("1.0.0").to_string();
    let manifest_version = json["manifest_version"].as_u64().unwrap_or(3) as u32;
    let homepage_url = json["homepage_url"].as_str().map(String::from);

    // Options page
    let options_page = json["options_ui"]["page"]
        .as_str()
        .or_else(|| json["options_page"].as_str())
        .map(String::from);

    // Popup page
    let popup_page = json["action"]["default_popup"]
        .as_str()
        .or_else(|| json["browser_action"]["default_popup"].as_str())
        .map(String::from);

    // Icon (search 128, 48, 32, 16)
    let icon_rel_path = json["icons"]["128"]
        .as_str()
        .or_else(|| json["icons"]["48"].as_str())
        .or_else(|| json["icons"]["32"].as_str())
        .or_else(|| json["icons"]["16"].as_str())
        .or_else(|| json["action"]["default_icon"].as_str());

    let icon = icon_rel_path.and_then(|rel| {
        let full = dir.join(rel);
        if full.exists() {
            fs::read(&full).ok().map(|bytes| {
                let ext = full
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("png");
                let mime = match ext {
                    "svg" => "image/svg+xml",
                    "jpg" | "jpeg" => "image/jpeg",
                    "webp" => "image/webp",
                    _ => "image/png",
                };
                let b64 = format!("data:{mime};base64,{}", simple_base64_encode(&bytes));
                b64
            })
        } else {
            None
        }
    });

    Ok(ExtensionInfo {
        id: id.to_string(),
        name,
        version,
        description,
        icon,
        enabled,
        source: source.to_string(),
        path: dir.to_string_lossy().to_string(),
        manifest_version,
        options_page,
        popup_page,
        homepage_url,
    })
}

fn simple_base64_encode(bytes: &[u8]) -> String {
    const CHARSET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::with_capacity(bytes.len() * 4 / 3 + 4);
    for chunk in bytes.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let triple = (b0 << 16) | (b1 << 8) | b2;

        result.push(CHARSET[((triple >> 18) & 0x3F) as usize] as char);
        result.push(CHARSET[((triple >> 12) & 0x3F) as usize] as char);

        if chunk.len() > 1 {
            result.push(CHARSET[((triple >> 6) & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }

        if chunk.len() > 2 {
            result.push(CHARSET[(triple & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
    }
    result
}

pub fn download_and_install_extension(
    id_or_url: &str,
    source_hint: Option<&str>,
) -> Result<ExtensionInfo, String> {
    let (id, source) = match normalize_extension_id(id_or_url) {
        Some((id, src)) => (id, source_hint.unwrap_or(&src).to_string()),
        None => {
            if is_valid_extension_id(id_or_url) {
                (
                    id_or_url.trim().to_ascii_lowercase(),
                    source_hint.unwrap_or("chrome").to_string(),
                )
            } else {
                return Err(format!("Invalid extension ID or Store URL: {id_or_url}"));
            }
        }
    };

    let chrome_url = format!(
        "https://clients2.google.com/service/update2/crx?response=redirect&os=win&arch=x64&os_arch=x86-64&nacl_arch=x86-64&prod=chromecrx&prodchannel=&prodversion=128.0.0.0&lang=en-US&acceptformat=crx2,crx3&x=id%3D{id}%26installsource%3Dondemand%26uc"
    );
    let edge_url = format!(
        "https://edge.microsoft.com/extensionwebstorebase/v1/crx?response=redirect&prod=chromiumcrx&prodchannel=&x=id%3D{id}%26installsource%3Dondemand%26uc"
    );

    let (primary_url, fallback_url) = if source == "edge" {
        (edge_url, chrome_url)
    } else {
        (chrome_url, edge_url)
    };

    let fetch_crx = |url: &str| -> Result<Vec<u8>, String> {
        let response = ureq::get(url)
            .set("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/128.0.0.0 Safari/537.36")
            .timeout(std::time::Duration::from_secs(30))
            .call()
            .map_err(|e| format!("HTTP request error ({url}): {e}"))?;

        let mut bytes = Vec::new();
        response
            .into_reader()
            .read_to_end(&mut bytes)
            .map_err(|e| format!("Failed to read stream: {e}"))?;

        if bytes.is_empty() {
            return Err(format!("Empty payload from {url}"));
        }
        Ok(bytes)
    };

    let crx_bytes = match fetch_crx(&primary_url) {
        Ok(bytes) => bytes,
        Err(primary_err) => {
            eprintln!("Primary extension download failed ({primary_err}), trying fallback...");
            fetch_crx(&fallback_url).map_err(|fallback_err| {
                format!(
                    "Failed to download extension '{id}' from store endpoints. Primary error: {primary_err}; Fallback error: {fallback_err}"
                )
            })?
        }
    };

    let zip_payload = extract_crx_zip_payload(&crx_bytes)?;
    let dest_dir = get_extensions_dir().join(&id);

    if dest_dir.exists() {
        let _ = fs::remove_dir_all(&dest_dir);
    }

    unpack_extension_zip(zip_payload, &dest_dir)?;
    parse_extension_manifest(&dest_dir, &id, &source, true)
}

pub fn load_unpacked_extension(path_str: &str) -> Result<ExtensionInfo, String> {
    let path = PathBuf::from(path_str);
    if !path.exists() || !path.is_dir() {
        return Err(format!("Directory does not exist: {path_str}"));
    }

    let manifest_path = path.join("manifest.json");
    if !manifest_path.exists() {
        return Err(format!("No manifest.json found in {path_str}"));
    }

    // Generate a deterministic 32-character hex ID based on the folder path
    let hash = format!("{:x}", md5_simple(path_str.as_bytes()));
    let id = hash.chars().take(32).collect::<String>();

    parse_extension_manifest(&path, &id, "unpacked", true)
}

fn md5_simple(data: &[u8]) -> u128 {
    // Simple fast hash for deterministic local folder IDs
    let mut h: u128 = 0xcbf29ce484222325;
    for &b in data {
        h ^= b as u128;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

pub fn get_enabled_extension_paths() -> Vec<String> {
    let storage = crate::storage::StorageManager::new();
    let mut extensions = storage.load_extensions();
    let ext_dir = get_extensions_dir();

    // Auto-discover folders on disk if not in extensions.json
    if let Ok(entries) = fs::read_dir(&ext_dir) {
        let mut modified = false;
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() && path.join("manifest.json").exists() {
                let path_str = path.to_string_lossy().to_string();
                if !extensions.iter().any(|e| e.path == path_str) {
                    let id = path
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_default();
                    if let Ok(ext) = parse_extension_manifest(&path, &id, "local", true) {
                        extensions.push(ext);
                        modified = true;
                    }
                }
            }
        }
        if modified {
            storage.save_extensions(&extensions);
        }
    }

    extensions
        .into_iter()
        .filter(|e| e.enabled && Path::new(&e.path).exists())
        .map(|e| e.path)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_extension_ids() {
        assert!(is_valid_extension_id("cjpalhdlnbpafiamejdnhcphjbkeiagm"));
        assert!(!is_valid_extension_id("short_id"));
        assert!(!is_valid_extension_id("cjpalhdlnbpafiamejdnhcphjbkeiag1")); // contains digit
    }

    #[test]
    fn test_url_normalization() {
        let chrome_url = "https://chromewebstore.google.com/detail/ublock-origin/cjpalhdlnbpafiamejdnhcphjbkeiagm";
        let (id, source) = normalize_extension_id(chrome_url).expect("Should parse chrome webstore URL");
        assert_eq!(id, "cjpalhdlnbpafiamejdnhcphjbkeiagm");
        assert_eq!(source, "chrome");

        let edge_url = "https://microsoftedge.microsoft.com/addons/detail/ublock-origin/odfafepnkmbhccpbejgmiehpchacaeak";
        let (id, source) = normalize_extension_id(edge_url).expect("Should parse edge addons URL");
        assert_eq!(id, "odfafepnkmbhccpbejgmiehpchacaeak");
        assert_eq!(source, "edge");
    }

    #[test]
    fn test_crx_zip_payload_extraction() {
        // Mock ZIP archive starting with PK\x03\x04
        let zip_mock = b"PK\x03\x04mock_zip_content";
        let extracted = extract_crx_zip_payload(zip_mock).unwrap();
        assert_eq!(extracted, zip_mock);

        // Mock CRX3 file: Cr24 (4b) + version 3 (4b) + header_len 4 (4b) + 4b header + PK\x03\x04...
        let mut crx3_mock = Vec::new();
        crx3_mock.extend_from_slice(b"Cr24");
        crx3_mock.extend_from_slice(&3u32.to_le_bytes());
        crx3_mock.extend_from_slice(&4u32.to_le_bytes()); // header len = 4
        crx3_mock.extend_from_slice(b"head"); // header data
        crx3_mock.extend_from_slice(b"PK\x03\x04payload");

        let extracted_crx3 = extract_crx_zip_payload(&crx3_mock).unwrap();
        assert_eq!(extracted_crx3, b"PK\x03\x04payload");
    }

    #[test]
    fn test_download_real_extension() {
        // Test downloading dark reader from Chrome Web Store
        let info = download_and_install_extension("eimadpbcbfnmbkopoojfekhnkhdbieeh", Some("chrome"));
        assert!(info.is_ok(), "Failed to download Dark Reader: {:?}", info.err());
        let info = info.unwrap();
        assert_eq!(info.id, "eimadpbcbfnmbkopoojfekhnkhdbieeh");
        assert!(!info.name.is_empty());
    }

    #[test]
    fn test_download_and_verify_ublock_origin() {
        // uBlock Origin Lite
        let info = download_and_install_extension("ddkjiahejlhfcafbddmgiahcphecmpfh", Some("chrome"));
        assert!(info.is_ok(), "Failed to download uBlock Origin Lite: {:?}", info.err());
        let info = info.unwrap();
        assert_eq!(info.id, "ddkjiahejlhfcafbddmgiahcphecmpfh");
        assert!(info.name.contains("uBlock") || info.name.contains("uBO"));
        assert!(Path::new(&info.path).join("manifest.json").exists());
    }

    #[test]
    fn test_download_and_verify_bitwarden() {
        // Bitwarden Password Manager
        let info = download_and_install_extension("nngceckbapebfimnlniiiahkandclblb", Some("chrome"));
        assert!(info.is_ok(), "Failed to download Bitwarden: {:?}", info.err());
        let info = info.unwrap();
        assert_eq!(info.id, "nngceckbapebfimnlniiiahkandclblb");
        assert!(info.name.contains("Bitwarden"));
        assert!(Path::new(&info.path).join("manifest.json").exists());
    }
}
