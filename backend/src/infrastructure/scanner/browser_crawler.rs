use crate::domain::fuzzing::{EndpointContext, InsertionPoint, ScanTarget};
use base64::{engine::general_purpose, Engine as _};
use headless_chrome::{
    protocol::cdp::{Network, Page::AddScriptToEvaluateOnNewDocument},
    Browser, LaunchOptionsBuilder,
};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use url::Url;

const RESPONSE_BODY_PREVIEW_LIMIT: usize = 4096;

pub struct BrowserCrawlResult {
    pub links: Vec<String>,
    pub endpoints: Vec<ScanTarget>,
}

pub struct BrowserCrawler {
    base_url: String,
    max_tabs: usize,
}

impl BrowserCrawler {
    pub fn new(base_url: &str, max_tabs: Option<usize>) -> Self {
        let max_tabs = max_tabs.unwrap_or(2).min(3); // Default 2, max 3
        Self {
            base_url: base_url.to_string(),
            max_tabs,
        }
    }

    pub fn crawl(&self) -> Result<BrowserCrawlResult, String> {
        let options = LaunchOptionsBuilder::default()
            .headless(true)
            .idle_browser_timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| e.to_string())?;

        let browser = Browser::new(options).map_err(|e| e.to_string())?;
        let tab = browser.new_tab().map_err(|e| e.to_string())?;

        let base_url_parsed = Url::parse(&self.base_url).map_err(|e| e.to_string())?;
        let base_host = base_url_parsed.host_str();
        let cdp_endpoints: Arc<Mutex<HashMap<String, EndpointContext>>> =
            Arc::new(Mutex::new(HashMap::new()));
        let cdp_endpoints_listener = cdp_endpoints.clone();
        let cdp_endpoints_response = cdp_endpoints.clone();

        let _ = tab.call_method(Network::Enable {
            max_total_buffer_size: None,
            max_resource_buffer_size: None,
            max_post_data_size: Some(64 * 1024),
            report_direct_socket_traffic: None,
            enable_durable_messages: None,
        });
        let _ = tab.add_event_listener(Arc::new(
            move |event: &headless_chrome::protocol::cdp::types::Event| {
                if let headless_chrome::protocol::cdp::types::Event::NetworkRequestWillBeSent(
                    event,
                ) = event
                {
                    let request = &event.params.request;
                    if request.url.starts_with("http://") || request.url.starts_with("https://") {
                        let mut ctx = EndpointContext::new(&request.method, &request.url);
                        ctx.headers = replayable_request_headers(&request.headers);
                        ctx.body = request.post_data.clone();
                        if let Ok(mut lock) = cdp_endpoints_listener.lock() {
                            lock.insert(event.params.request_id.clone(), ctx);
                        }
                    }
                }
            },
        ));
        let _ = tab.register_response_handling(
            "limma-browser-crawler",
            Box::new(move |params, fetch_body| {
                if let Ok(mut lock) = cdp_endpoints_response.lock() {
                    if let Some(ctx) = lock.get_mut(&params.request_id) {
                        ctx.response_status =
                            Some(params.response.status.min(u16::MAX as u32) as u16);
                        ctx.response_headers = cdp_headers_to_map(&params.response.headers);

                        if is_previewable_mime(&params.response.mime_type) {
                            ctx.response_body_preview = fetch_body().ok().and_then(|body| {
                                response_body_preview(
                                    &body.body,
                                    body.base_64_encoded,
                                    RESPONSE_BODY_PREVIEW_LIMIT,
                                )
                            });
                        }
                    }
                }
            }),
        );

        // Instead of directly calling CDP, we can evaluate JS to intercept fetch/XHR
        // or just rely on DOM for now. The CDP bindings are unstable across versions.
        let js_intercept = r#"
            if (!window.__limma_capture_installed) {
            window.__limma_capture_installed = true;
            window.__captured_endpoints = window.__captured_endpoints || [];
            let oldFetch = window.fetch;
            window.fetch = function() {
                window.__captured_endpoints.push({ method: arguments[1]?.method || 'GET', url: arguments[0] });
                return oldFetch.apply(this, arguments);
            };
            let oldOpen = XMLHttpRequest.prototype.open;
            XMLHttpRequest.prototype.open = function(method, url) {
                window.__captured_endpoints.push({ method: method, url: url });
                return oldOpen.apply(this, arguments);
            };
            }
        "#;

        // Navigate and wait for idle
        let _ = tab.call_method(AddScriptToEvaluateOnNewDocument {
            source: js_intercept.to_string(),
            world_name: None,
            include_command_line_api: None,
            run_immediately: Some(true),
        });
        tab.navigate_to(&self.base_url).map_err(|e| e.to_string())?;
        let _ = tab.evaluate(js_intercept, false);
        tab.wait_until_navigated().map_err(|e| e.to_string())?;

        // Wait an extra 3 seconds for SPA initial requests to fire
        std::thread::sleep(Duration::from_secs(3));

        // Retrieve captured endpoints from window
        let mut endpoints_extracted: Vec<EndpointContext> = {
            let mut extracted = Vec::new();
            if let Ok(res) = tab.evaluate("JSON.stringify(window.__captured_endpoints)", false) {
                if let Some(val) = res.value {
                    if let Some(val_str) = val.as_str() {
                        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(val_str) {
                            if let Some(arr) = parsed.as_array() {
                                for item in arr {
                                    if let (Some(m), Some(u)) =
                                        (item.get("method"), item.get("url"))
                                    {
                                        let raw_url = u.as_str().unwrap_or("");
                                        if let Ok(joined) = base_url_parsed.join(raw_url) {
                                            if joined.host_str() == base_host {
                                                extracted.push(EndpointContext::new(
                                                    m.as_str().unwrap_or("GET"),
                                                    joined.as_str(),
                                                ));
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            extracted
        };

        if let Ok(lock) = cdp_endpoints.lock() {
            for ctx in lock.values() {
                if let Ok(joined) = base_url_parsed.join(&ctx.url) {
                    if joined.host_str() == base_host {
                        let mut normalized = ctx.clone();
                        normalized.url = joined.to_string();
                        endpoints_extracted.push(normalized);
                    }
                }
            }
        }

        // Extract DOM links
        let mut links = HashSet::new();
        if let Ok(elements) = tab.find_elements("a[href]") {
            for el in elements {
                if let Ok(Some(href)) = el.get_attribute_value("href") {
                    if let Ok(joined) = base_url_parsed.join(&href) {
                        if joined.host_str() == base_url_parsed.host_str() {
                            let mut clean = joined.clone();
                            clean.set_fragment(None);
                            links.insert(clean.to_string());
                        }
                    }
                }
            }
        }

        // Convert captured contexts into ScanTargets by inferring insertion points
        let mut endpoints = Vec::new();
        let mut seen_urls = HashSet::new();

        for ctx in endpoints_extracted {
            let mut insertion_points = Vec::new();

            // Add query params
            if let Ok(parsed) = Url::parse(&ctx.url) {
                for (k, _) in parsed.query_pairs() {
                    insertion_points.push(InsertionPoint::QueryParam(k.to_string()));
                }

                // Deduplicate by URL without query + method
                let dedup_key = format!("{}|{}|{}", ctx.method, parsed.path(), ctx.body.is_some());
                if seen_urls.contains(&dedup_key) {
                    continue;
                }
                seen_urls.insert(dedup_key);
            }

            // Add JSON body mutation points
            if let Some(ref body) = ctx.body {
                let paths = crate::infrastructure::active_detection::fuzzing::json_mutator::JsonMutator::extract_mutation_paths(body);
                for p in paths {
                    insertion_points.push(InsertionPoint::JsonBodyPath(p));
                }
            }

            endpoints.push(ScanTarget {
                endpoint: ctx,
                insertion_points,
            });
        }

        let mut links: Vec<_> = links.into_iter().collect();
        links.sort();
        links.truncate(self.max_tabs.saturating_mul(25).max(25));

        Ok(BrowserCrawlResult { links, endpoints })
    }
}

fn replayable_request_headers(headers: &Network::Headers) -> HashMap<String, String> {
    cdp_headers_to_map(headers)
        .into_iter()
        .filter(|(name, _)| is_replayable_request_header(name))
        .collect()
}

fn is_replayable_request_header(name: &str) -> bool {
    !matches!(
        name.to_ascii_lowercase().as_str(),
        "host"
            | "content-length"
            | "connection"
            | "keep-alive"
            | "proxy-authenticate"
            | "proxy-authorization"
            | "te"
            | "trailer"
            | "transfer-encoding"
            | "upgrade"
    )
}

fn cdp_headers_to_map(headers: &Network::Headers) -> HashMap<String, String> {
    headers
        .0
        .as_ref()
        .and_then(serde_json::Value::as_object)
        .map(|raw_headers| {
            raw_headers
                .iter()
                .filter_map(|(name, value)| {
                    header_value_to_string(value).map(|value| (name.to_ascii_lowercase(), value))
                })
                .collect()
        })
        .unwrap_or_default()
}

fn header_value_to_string(value: &serde_json::Value) -> Option<String> {
    match value {
        serde_json::Value::String(value) => Some(value.clone()),
        serde_json::Value::Number(_) | serde_json::Value::Bool(_) => Some(value.to_string()),
        serde_json::Value::Array(values) => {
            let joined = values
                .iter()
                .filter_map(header_value_to_string)
                .collect::<Vec<_>>()
                .join(", ");
            (!joined.is_empty()).then_some(joined)
        }
        _ => None,
    }
}

fn is_previewable_mime(mime_type: &str) -> bool {
    let mime_type = mime_type.to_ascii_lowercase();
    mime_type.starts_with("text/")
        || mime_type.contains("json")
        || mime_type.contains("xml")
        || mime_type.contains("javascript")
        || mime_type.contains("x-www-form-urlencoded")
        || mime_type.contains("graphql")
}

fn response_body_preview(body: &str, base64_encoded: bool, limit: usize) -> Option<String> {
    let decoded = if base64_encoded {
        let bytes = general_purpose::STANDARD.decode(body).ok()?;
        String::from_utf8_lossy(&bytes).into_owned()
    } else {
        body.to_string()
    };

    if decoded.is_empty() {
        return None;
    }

    Some(decoded.chars().take(limit).collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn cdp_headers_are_normalized_and_sanitized_for_replay() {
        let headers = Network::Headers(Some(json!({
            "Content-Type": "application/json",
            "X-Trace": ["a", "b"],
            "Content-Length": "123",
            "Host": "example.com"
        })));

        let replay_headers = replayable_request_headers(&headers);

        assert_eq!(
            replay_headers.get("content-type").map(String::as_str),
            Some("application/json")
        );
        assert_eq!(
            replay_headers.get("x-trace").map(String::as_str),
            Some("a, b")
        );
        assert!(!replay_headers.contains_key("content-length"));
        assert!(!replay_headers.contains_key("host"));
    }

    #[test]
    fn response_preview_decodes_base64_and_applies_limit() {
        let encoded = general_purpose::STANDARD.encode("abcdef");

        assert_eq!(
            response_body_preview(&encoded, true, 4).as_deref(),
            Some("abcd")
        );
        assert_eq!(
            response_body_preview("plain text", false, 5).as_deref(),
            Some("plain")
        );
        assert_eq!(response_body_preview("", false, 5), None);
    }
}
