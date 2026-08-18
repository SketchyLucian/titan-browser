pub fn normalize_or_search_url(input: &str) -> String {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return "https://www.google.com".to_string();
    }

    // Direct protocol checks
    if trimmed.starts_with("http://")
        || trimmed.starts_with("https://")
        || trimmed.starts_with("file://")
        || trimmed.starts_with("about:")
    {
        return trimmed.to_string();
    }

    // YouTube quick search prefix
    if let Some(query) = trimmed.strip_prefix("@yt ")
        .or_else(|| trimmed.strip_prefix("yt:"))
        .or_else(|| trimmed.strip_prefix("youtube "))
    {
        return format!(
            "https://www.youtube.com/results?search_query={}",
            urlencoding(query.trim())
        );
    }

    // GitHub quick search prefix
    if let Some(query) = trimmed.strip_prefix("@gh ")
        .or_else(|| trimmed.strip_prefix("gh:"))
        .or_else(|| trimmed.strip_prefix("github "))
    {
        return format!(
            "https://github.com/search?q={}",
            urlencoding(query.trim())
        );
    }

    // DuckDuckGo quick search prefix
    if let Some(query) = trimmed.strip_prefix("@ddg ").or_else(|| trimmed.strip_prefix("ddg:")) {
        return format!(
            "https://duckduckgo.com/?q={}",
            urlencoding(query.trim())
        );
    }

    // Check if it looks like a domain name (e.g. "youtube.com", "crates.io", "localhost:8080")
    if is_likely_domain_or_ip(trimmed) {
        return format!("https://{trimmed}");
    }

    // Default to Google search
    format!("https://www.google.com/search?q={}", urlencoding(trimmed))
}

fn is_likely_domain_or_ip(text: &str) -> bool {
    // If it contains whitespace, it's definitely a search query
    if text.contains(' ') || text.contains('\t') || text.contains('\n') {
        return false;
    }

    // Check for localhost or IP patterns
    if text.starts_with("localhost") || text.starts_with("127.0.0.1") {
        return true;
    }

    // Look for top-level domains or domain formatting
    let parts: Vec<&str> = text.split('/').collect();
    let host_part = parts[0];

    if let Some(dot_idx) = host_part.rfind('.') {
        let tld = &host_part[dot_idx + 1..];
        let tld_clean = tld.split(':').next().unwrap_or(tld);
        if tld_clean.len() >= 2 && tld_clean.chars().all(|c| c.is_alphabetic()) {
            return true;
        }
    }

    false
}

fn urlencoding(input: &str) -> String {
    let mut encoded = String::new();
    for byte in input.bytes() {
        match byte {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(byte as char);
            }
            b' ' => encoded.push('+'),
            _ => {
                encoded.push_str(&format!("%{:02X}", byte));
            }
        }
    }
    encoded
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_detection() {
        assert_eq!(
            normalize_or_search_url("https://youtube.com"),
            "https://youtube.com"
        );
        assert_eq!(
            normalize_or_search_url("youtube.com"),
            "https://youtube.com"
        );
        assert_eq!(
            normalize_or_search_url("rust-lang.org/learn"),
            "https://rust-lang.org/learn"
        );
        assert_eq!(
            normalize_or_search_url("localhost:3000"),
            "https://localhost:3000"
        );
    }

    #[test]
    fn test_search_detection() {
        assert_eq!(
            normalize_or_search_url("rust programming tutorial"),
            "https://www.google.com/search?q=rust+programming+tutorial"
        );
        assert_eq!(
            normalize_or_search_url("@yt lofi chill"),
            "https://www.youtube.com/results?search_query=lofi+chill"
        );
    }
}
