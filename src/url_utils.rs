pub fn search_url_for_engine(engine: &str, query: &str) -> String {
    let encoded = urlencoding(query);
    match engine {
        "DuckDuckGo" => format!("https://duckduckgo.com/?q={}", encoded),
        "Bing" => format!("https://www.bing.com/search?q={}", encoded),
        "Brave" => format!("https://search.brave.com/search?q={}", encoded),
        "YouTube" => format!("https://www.youtube.com/results?search_query={}", encoded),
        _ => format!("https://www.google.com/search?q={}", encoded),
    }
}

#[allow(dead_code)]
pub fn normalize_or_search_url(input: &str) -> String {
    normalize_or_search_url_with_engine(input, "Google")
}

pub fn normalize_or_search_url_with_engine(input: &str, engine: &str) -> String {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return search_url_for_engine(engine, "");
    }

    // Direct protocol checks
    if trimmed.starts_with("http://")
        || trimmed.starts_with("https://")
        || trimmed.starts_with("file://")
        || trimmed.starts_with("about:")
        || trimmed.starts_with("titan://")
        || trimmed.starts_with("chrome://")
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

    // Default search engine
    search_url_for_engine(engine, trimmed)
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

pub fn strip_tracking_parameters(raw_url: &str) -> String {
    if !raw_url.starts_with("http://") && !raw_url.starts_with("https://") {
        return raw_url.to_string();
    }

    if let Ok(mut parsed) = url::Url::parse(raw_url) {
        if parsed.query().is_none() {
            return raw_url.to_string();
        }

        let tracking_keys = [
            "utm_source", "utm_medium", "utm_campaign", "utm_term", "utm_content", "utm_id",
            "utm_source_platform", "utm_creative_format", "utm_marketing_tactic",
            "fbclid", "gclid", "gclsrc", "dclid", "msclkid", "mc_eid", "igshid",
            "yclid", "_hsenc", "_hsmi", "wbraid", "gbraid", "twclid", "ref_src", "ref_url"
        ];

        let filtered_pairs: Vec<(String, String)> = parsed
            .query_pairs()
            .filter(|(k, _)| !tracking_keys.contains(&k.to_lowercase().as_str()))
            .map(|(k, v)| (k.into_owned(), v.into_owned()))
            .collect();

        if filtered_pairs.is_empty() {
            parsed.set_query(None);
        } else {
            let mut serializer = url::form_urlencoded::Serializer::new(String::new());
            for (k, v) in filtered_pairs {
                serializer.append_pair(&k, &v);
            }
            parsed.set_query(Some(&serializer.finish()));
        }

        parsed.to_string()
    } else {
        raw_url.to_string()
    }
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

    #[test]
    fn test_strip_tracking_parameters() {
        assert_eq!(
            strip_tracking_parameters("https://example.com/article?utm_source=twitter&utm_medium=social&page=2"),
            "https://example.com/article?page=2"
        );
        assert_eq!(
            strip_tracking_parameters("https://example.com/item?fbclid=IwAR123456"),
            "https://example.com/item"
        );
        assert_eq!(
            strip_tracking_parameters("https://example.com/?gclid=abc&q=rust"),
            "https://example.com/?q=rust"
        );
    }
}
