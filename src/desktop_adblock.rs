use crate::adblock_engine::AdblockEngineManager;
use crate::browser::UserEvent;
use crate::ipc::BrowserSettings;
use std::rc::Rc;
use std::sync::{Arc, RwLock};
use tao::event_loop::EventLoopProxy;
use url::Url;
use webview2_com::{
    take_pwstr, Microsoft::Web::WebView2::Win32::*, WebResourceRequestedEventHandler,
};
use windows_core::{HSTRING, PWSTR};
use wry::{WebView, WebViewExtWindows};

#[derive(Clone, Debug)]
pub struct DesktopAdblockSettings {
    enabled: bool,
    aggressive_mode: bool,
    blocked_domains: Vec<String>,
    whitelisted_domains: Vec<String>,
}

impl From<&BrowserSettings> for DesktopAdblockSettings {
    fn from(settings: &BrowserSettings) -> Self {
        Self {
            enabled: settings.adblock_enabled,
            aggressive_mode: settings.adblock_aggressive_mode,
            blocked_domains: settings.adblock_blocked_domains.clone(),
            whitelisted_domains: settings.adblock_whitelisted_domains.clone(),
        }
    }
}

pub type SharedDesktopAdblockSettings = Arc<RwLock<DesktopAdblockSettings>>;

pub fn shared_settings(settings: &BrowserSettings) -> SharedDesktopAdblockSettings {
    Arc::new(RwLock::new(DesktopAdblockSettings::from(settings)))
}

pub fn update_shared_settings(shared: &SharedDesktopAdblockSettings, settings: &BrowserSettings) {
    if let Ok(mut current) = shared.write() {
        *current = DesktopAdblockSettings::from(settings);
    }
}

fn request_host(url: &str) -> Option<String> {
    Url::parse(url)
        .ok()
        .and_then(|parsed| parsed.host_str().map(str::to_ascii_lowercase))
}

fn domain_matches(host: &str, domain: &str) -> bool {
    let domain = domain.trim().trim_start_matches('.').to_ascii_lowercase();
    !domain.is_empty() && (host == domain || host.ends_with(&format!(".{domain}")))
}

fn is_bypassed_host(host: &str) -> bool {
    host == "localhost"
        || host == "127.0.0.1"
        || host == "::1"
        || host.contains("browserbench")
        || host.contains("speedometer")
}

fn should_block_request(
    engine: &AdblockEngineManager,
    settings: &DesktopAdblockSettings,
    url: &str,
    source_url: &str,
    request_type: &str,
) -> bool {
    if !settings.enabled || !(url.starts_with("http://") || url.starts_with("https://")) {
        return false;
    }

    let Some(host) = request_host(url) else {
        return false;
    };
    if is_bypassed_host(&host) {
        return false;
    }

    let source_host = request_host(source_url).unwrap_or_default();
    if settings
        .whitelisted_domains
        .iter()
        .any(|domain| domain_matches(&host, domain) || domain_matches(&source_host, domain))
    {
        return false;
    }

    if settings
        .blocked_domains
        .iter()
        .any(|domain| domain_matches(&host, domain))
    {
        return true;
    }

    if engine
        .check_network_request(url, source_url, request_type)
        .matched
    {
        return true;
    }

    if settings.aggressive_mode {
        let lower = url.to_ascii_lowercase();
        return [
            "adservice",
            "adserver",
            "telemetry",
            "tracking",
            "analytics",
            "pixel",
        ]
        .iter()
        .any(|token| lower.contains(token));
    }

    false
}

unsafe fn request_header(request: &ICoreWebView2WebResourceRequest, name: &str) -> Option<String> {
    let headers = request.Headers().ok()?;
    let mut value = PWSTR::null();
    let name = HSTRING::from(name);
    headers.GetHeader(&name, &mut value).ok()?;
    Some(take_pwstr(value))
}

fn request_type(
    context: COREWEBVIEW2_WEB_RESOURCE_CONTEXT,
    sec_fetch_dest: Option<&str>,
) -> &'static str {
    match context {
        COREWEBVIEW2_WEB_RESOURCE_CONTEXT_DOCUMENT => match sec_fetch_dest {
            Some("iframe") | Some("frame") | Some("object") | Some("embed") => "subdocument",
            _ => "document",
        },
        COREWEBVIEW2_WEB_RESOURCE_CONTEXT_STYLESHEET => "stylesheet",
        COREWEBVIEW2_WEB_RESOURCE_CONTEXT_IMAGE => "image",
        COREWEBVIEW2_WEB_RESOURCE_CONTEXT_MEDIA => "media",
        COREWEBVIEW2_WEB_RESOURCE_CONTEXT_FONT => "font",
        COREWEBVIEW2_WEB_RESOURCE_CONTEXT_SCRIPT => "script",
        COREWEBVIEW2_WEB_RESOURCE_CONTEXT_XML_HTTP_REQUEST
        | COREWEBVIEW2_WEB_RESOURCE_CONTEXT_FETCH => "xhr",
        COREWEBVIEW2_WEB_RESOURCE_CONTEXT_TEXT_TRACK => "other",
        COREWEBVIEW2_WEB_RESOURCE_CONTEXT_EVENT_SOURCE => "xhr",
        COREWEBVIEW2_WEB_RESOURCE_CONTEXT_WEBSOCKET => "websocket",
        COREWEBVIEW2_WEB_RESOURCE_CONTEXT_MANIFEST => "other",
        COREWEBVIEW2_WEB_RESOURCE_CONTEXT_SIGNED_EXCHANGE => "other",
        COREWEBVIEW2_WEB_RESOURCE_CONTEXT_PING => "ping",
        COREWEBVIEW2_WEB_RESOURCE_CONTEXT_CSP_VIOLATION_REPORT => "csp_report",
        _ => "other",
    }
}

pub fn attach_request_blocker(
    webview: &WebView,
    engine: Rc<AdblockEngineManager>,
    settings: SharedDesktopAdblockSettings,
    proxy: EventLoopProxy<UserEvent>,
) -> Result<(), String> {
    let controller = webview.controller();
    let core = unsafe { controller.CoreWebView2() }.map_err(|error| error.to_string())?;

    unsafe {
        core.AddWebResourceRequestedFilter(
            &HSTRING::from("*"),
            COREWEBVIEW2_WEB_RESOURCE_CONTEXT_ALL,
        )
        .map_err(|error| error.to_string())?;
    }

    let handler = WebResourceRequestedEventHandler::create(Box::new(move |sender, args| {
        let Some(args) = args else {
            return Ok(());
        };
        let request = unsafe { args.Request()? };

        let request_url = unsafe {
            let mut value = PWSTR::null();
            request.Uri(&mut value)?;
            take_pwstr(value)
        };

        let source_url = unsafe {
            request_header(&request, "Referer")
                .filter(|url| url.starts_with("http://") || url.starts_with("https://"))
                .or_else(|| {
                    sender.and_then(|webview| {
                        let mut value = PWSTR::null();
                        webview.Source(&mut value).ok()?;
                        let url = take_pwstr(value);
                        (url.starts_with("http://") || url.starts_with("https://")).then_some(url)
                    })
                })
                .unwrap_or_default()
        };

        let mut context = COREWEBVIEW2_WEB_RESOURCE_CONTEXT_OTHER;
        unsafe { args.ResourceContext(&mut context)? };
        let sec_fetch_dest = unsafe { request_header(&request, "Sec-Fetch-Dest") };
        let resource_type = request_type(context, sec_fetch_dest.as_deref());

        let blocked = settings
            .read()
            .map(|settings| {
                should_block_request(&engine, &settings, &request_url, &source_url, resource_type)
            })
            .unwrap_or(false);
        if !blocked {
            return Ok(());
        }

        // WebView2 has no cancellation flag for this event. A synthetic 403/204 is
        // still a successful opaque response to `fetch(..., { mode: "no-cors" })`.
        // Chromium rejects port 1 as unsafe before sending traffic, which gives all
        // resource types the same network-error semantics as a cancelled request.
        unsafe {
            request.SetUri(&HSTRING::from(
                "https://127.0.0.1:1/.titan-adblock-cancelled",
            ))?
        };

        let domain = request_host(&request_url).unwrap_or_default();
        let message = serde_json::json!({
            "type": "ReportBlockedAd",
            "domain": domain,
            "url": request_url.chars().take(300).collect::<String>(),
            "req_type": resource_type,
        })
        .to_string();
        let _ = proxy.send_event(UserEvent::Ipc(message));

        Ok(())
    }));

    let mut event_token = 0;
    unsafe { core.add_WebResourceRequested(&handler, &mut event_token) }
        .map(|_| ())
        .map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_settings() -> DesktopAdblockSettings {
        DesktopAdblockSettings {
            enabled: true,
            aggressive_mode: false,
            blocked_domains: vec!["ads.example".into()],
            whitelisted_domains: Vec::new(),
        }
    }

    #[test]
    fn native_layer_blocks_engine_and_user_domain_rules() {
        let engine = AdblockEngineManager::new();
        let settings = test_settings();

        assert!(should_block_request(
            &engine,
            &settings,
            "https://ad.doubleclick.net/banner.js",
            "https://example.com/",
            "script",
        ));
        assert!(should_block_request(
            &engine,
            &settings,
            "https://cdn.ads.example/banner.png",
            "https://example.com/",
            "image",
        ));
        assert!(!should_block_request(
            &engine,
            &settings,
            "https://en.wikipedia.org/static/site.js",
            "https://en.wikipedia.org/",
            "script",
        ));
    }

    #[test]
    fn native_layer_honors_source_and_request_whitelists() {
        let engine = AdblockEngineManager::new();
        let mut settings = test_settings();
        settings.whitelisted_domains.push("example.com".into());

        assert!(!should_block_request(
            &engine,
            &settings,
            "https://ad.doubleclick.net/banner.js",
            "https://www.example.com/",
            "script",
        ));
        settings.whitelisted_domains.push("ads.example".into());
        assert!(!should_block_request(
            &engine,
            &settings,
            "https://cdn.ads.example/banner.png",
            "https://another.example/",
            "image",
        ));
    }

    #[test]
    fn webview_context_maps_frames_without_misclassifying_navigation() {
        assert_eq!(
            request_type(COREWEBVIEW2_WEB_RESOURCE_CONTEXT_DOCUMENT, Some("iframe")),
            "subdocument"
        );
        assert_eq!(
            request_type(COREWEBVIEW2_WEB_RESOURCE_CONTEXT_DOCUMENT, Some("document")),
            "document"
        );
        assert_eq!(
            request_type(COREWEBVIEW2_WEB_RESOURCE_CONTEXT_FETCH, Some("empty")),
            "xhr"
        );
    }
}
