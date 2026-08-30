pub const BLOCKED_TELEMETRY_DOMAINS: &[&str] = &[
    "pipe.aria.microsoft.com",
    "events.data.microsoft.com",
    "telemetry.microsoft.com",
    "watson.telemetry.microsoft.com",
    "mobile.pipe.aria.microsoft.com",
    "google-analytics.com",
    "analytics.google.com",
    "googletagmanager.com",
    "stats.g.doubleclick.net",
    "app-measurement.com",
    "crashlyticsreports-pa.googleapis.com",
    "crashlytics.com",
    "firebase-logging.googleapis.com",
    "firebaselogging-pa.googleapis.com",
    "sentry.io",
    "browser.sentry-cdn.com",
    "js.sentry-cdn.com",
    "sentry-cdn.com",
    "bugsnag.com",
    "sessions.bugsnag.com",
    "segment.io",
    "segment.com",
    "api.segment.io",
    "cdn.segment.com",
    "mixpanel.com",
    "api.mixpanel.com",
    "amplitude.com",
    "api2.amplitude.com",
    "api.amplitude.com",
    "clarity.ms",
    "hotjar.com",
    "hotjar.io",
    "static.hotjar.com",
    "fullstory.com",
    "mouseflow.com",
    "heapanalytics.com",
    "heap.io",
    "datadoghq.com",
    "browser-intake-datadoghq.com",
    "browser-intake-datadoghq.eu",
    "newrelic.com",
    "nr-data.net",
    "bam.nr-data.net",
    "loggly.com",
    "scorecardresearch.com",
    "quantserve.com",
    "bat.bing.com",
    "snap.licdn.com",
    "px.ads.linkedin.com",
    "analytics.twitter.com",
    "ads-twitter.com",
    "analytics.tiktok.com",
    "analytics.yahoo.com",
    "plausible.io",
];

pub fn webview2_browser_args() -> String {
    [
        "--disable-features=Translate,OptimizationHints,MediaRouter,InterestFeedContentSuggestions,AttributionReporting,PrivacySandboxAdsAPIs,TopicsAPI,InterestGroupStorage,Fledge,SharedStorageAPI,PrivateAggregationApi,ReportingAPI".to_string(),
        "--disable-domain-reliability".into(),
        "--disable-sync".into(),
        "--disable-breakpad".into(),
        "--no-report-upload".into(),
        "--metrics-recording-only".into(),
        "--no-pings".into(),
    ]
    .join(" ")
}

pub fn sanitize_local_log_url(raw_url: &str) -> String {
    let Ok(mut parsed) = url::Url::parse(raw_url) else {
        return String::new();
    };
    if parsed.scheme() != "http" && parsed.scheme() != "https" {
        return String::new();
    }
    parsed.set_query(None);
    parsed.set_fragment(None);
    parsed.to_string()
}

pub fn normalize_domain_rule(raw_domain: &str) -> Option<String> {
    let candidate = raw_domain.trim();
    if candidate.is_empty() || candidate.len() > 2048 {
        return None;
    }
    let url = if candidate.starts_with("http://") || candidate.starts_with("https://") {
        url::Url::parse(candidate).ok()?
    } else {
        url::Url::parse(&format!("https://{candidate}")).ok()?
    };
    let host = url.host_str()?.trim_end_matches('.').to_ascii_lowercase();
    if host.is_empty() || host.len() > 253 {
        return None;
    }
    let valid = host == "localhost"
        || host.parse::<std::net::IpAddr>().is_ok()
        || host.split('.').all(|label| {
            !label.is_empty()
                && label.len() <= 63
                && !label.starts_with('-')
                && !label.ends_with('-')
                && label
                    .chars()
                    .all(|ch| ch.is_ascii_alphanumeric() || ch == '-')
        });
    valid.then_some(host)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn browser_args_contain_privacy_flags() {
        let args = webview2_browser_args();
        assert!(args.contains("--no-report-upload"));
        assert!(args.contains("--no-pings"));
    }

    #[test]
    fn local_log_urls_drop_identifiers() {
        assert_eq!(
            sanitize_local_log_url("https://tracker.example/collect?user=42#token"),
            "https://tracker.example/collect"
        );
        assert_eq!(sanitize_local_log_url("not a url"), "");
    }

    #[test]
    fn domain_rules_are_normalized_and_validated() {
        assert_eq!(
            normalize_domain_rule("https://Analytics.Example.com/path"),
            Some("analytics.example.com".into())
        );
        assert_eq!(normalize_domain_rule("\"><script>"), None);
        assert_eq!(normalize_domain_rule("bad host.example"), None);
    }
}
