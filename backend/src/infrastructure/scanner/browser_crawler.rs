use crate::domain::fuzzing::{EndpointContext, ScanTarget, InsertionPoint};
use headless_chrome::{Browser, LaunchOptionsBuilder};
use std::collections::{HashSet, HashMap};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use url::Url;

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
        let base_host = base_url_parsed.host_str().unwrap_or_default().to_string();

        // Instead of directly calling CDP, we can evaluate JS to intercept fetch/XHR
        // or just rely on DOM for now. The CDP bindings are unstable across versions.
        let js_intercept = r#"
            window.__captured_endpoints = [];
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
        "#;

        // Navigate and wait for idle
        tab.navigate_to(&self.base_url).map_err(|e| e.to_string())?;
        let _ = tab.evaluate(js_intercept, false);
        tab.wait_until_navigated().map_err(|e| e.to_string())?;
        
        // Wait an extra 3 seconds for SPA initial requests to fire
        std::thread::sleep(Duration::from_secs(3));

        // Retrieve captured endpoints from window
        let endpoints_extracted: Vec<EndpointContext> = {
            let mut extracted = Vec::new();
            if let Ok(res) = tab.evaluate("JSON.stringify(window.__captured_endpoints)", false) {
                if let Some(val) = res.value {
                    if let Some(val_str) = val.as_str() {
                        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(val_str) {
                            if let Some(arr) = parsed.as_array() {
                                for item in arr {
                                    if let (Some(m), Some(u)) = (item.get("method"), item.get("url")) {
                                        extracted.push(EndpointContext::new(m.as_str().unwrap_or("GET"), u.as_str().unwrap_or("")));
                                    }
                                }
                            }
                        }
                    }
                }
            }
            extracted
        };

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

        Ok(BrowserCrawlResult {
            links: links.into_iter().collect(),
            endpoints,
        })
    }
}
