use crate::domain::entities::{RedirectChainEntry, ScanEvent, ScannedPage};
use crate::infrastructure::scanner::{fingerprint, security};
use chrono::Utc;
use reqwest::Client;
use scraper::{Html, Selector};
use std::collections::{HashMap, HashSet, VecDeque};
use std::time::Instant;
use url::Url;

pub struct CrawlResult {
    pub pages: Vec<ScannedPage>,
    pub events: Vec<ScanEvent>,
    pub final_url: String, // Final URL of the starting page
    pub main_chain: Vec<RedirectChainEntry>,
}

#[derive(Debug)]
struct QueueItem {
    url: String,
    depth: u32,
}

pub async fn crawl(
    client: &Client,
    fingerprinter: &fingerprint::FingerprintEngine,
    start_url: &str,
    config: &crate::domain::engine_config::EngineConfig,
    tx: Option<tokio::sync::mpsc::UnboundedSender<ScanEvent>>,
) -> Result<CrawlResult, String> {
    let mut pages = Vec::new();
    let mut events = Vec::new();

    let mut emit_event =
        |event_type: &str, level: &str, message: String, payload: Option<serde_json::Value>| {
            let ev = ScanEvent {
                timestamp: Utc::now(),
                event_type: event_type.to_string(),
                level: level.to_string(),
                message,
                payload,
            };
            events.push(ev.clone());
            if let Some(ref t) = tx {
                let _ = t.send(ev);
            }
        };

    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();

    let base_parsed = Url::parse(start_url).map_err(|e| e.to_string())?;

    queue.push_back(QueueItem {
        url: start_url.to_string(),
        depth: 0,
    });

    let mut main_final_url = start_url.to_string();
    let mut main_chain = Vec::new();

    // Wordlist discovery injection
    let mut added_wordlists = 0;
    for word in &config.wordlist {
        if added_wordlists >= config.max_wordlist_items_per_scan { break; }
        let mut w_url = base_parsed.clone();
        // Append wordlist to path cleanly
            let current_path = w_url.path();
            let new_path = if current_path.ends_with('/') {
                format!("{}{}", current_path, word)
            } else {
                format!("{}/{}", current_path, word)
            };
            w_url.set_path(&new_path);
            let w_url_str = w_url.to_string();
            queue.push_back(QueueItem {
                url: w_url_str,
                depth: 1, // Start injected wordlists at depth 1
            });
            added_wordlists += 1;
    }

    emit_event(
        "SCAN_STARTED",
        "INFO",
        format!("Initiating Phase 4 Crawl starting at {}", start_url),
        None,
    );

    while let Some(item) = queue.pop_front() {
        if pages.len() as u32 >= (config.max_depth * 10).max(10) { // Limit total pages reasonably based on depth
            emit_event(
                "CRAWL_LIMIT_REACHED",
                "WARN",
                "Hit crawler page limit constraint".to_string(),
                None,
            );
            break;
        }

        if visited.contains(&item.url) {
            continue;
        }
        visited.insert(item.url.clone());

        emit_event(
            "CRAWLING_PAGE",
            "INFO",
            format!("Crawling page (depth: {}): {}", item.depth, item.url),
            None,
        );

        if !pages.is_empty() {
            config.rate_limiter.wait().await;
        }

        // Fetch process similar to scanner.rs
        let fetch_res = fetch_manual_redirects(client, &item.url).await;

        match fetch_res {
            Ok((final_url, status, headers, body, latency, chain)) => {
                if pages.is_empty() {
                    main_final_url = final_url.clone();
                    main_chain = chain.clone();
                }

                // If final url is different from requested, add to visited
                visited.insert(final_url.clone());

                let mut ct = None;
                for (k, v) in &headers {
                    if k.as_str() == "content-type" {
                        ct = Some(v.clone())
                    }
                }

                let scan_ctx = fingerprint::ScanContext::new(&body, &headers);
                let detected = fingerprinter.analyze(&scan_ctx);

                let security_headers = security::audit_headers(&headers);
                let risk_insights =
                    security::generate_insights(&headers, &detected, &final_url, &body, &chain);

                let sp = ScannedPage {
                    url: item.url.clone(),
                    status_code: status,
                    latency_ms: latency,
                    headers: headers.clone(),
                    content_type: ct.clone(),
                    detected_technologies: detected.clone(),
                    security_headers,
                    risk_insights,
                };
                pages.push(sp.clone());

                emit_event(
                    "PAGE_CRAWLED",
                    "INFO",
                    format!("Successfully fetched {}", item.url),
                    Some(serde_json::to_value(&sp).unwrap_or(serde_json::Value::Null)),
                );

                if !detected.is_empty() {
                    emit_event(
                        "TECH_DETECTED",
                        "INFO",
                        format!(
                            "Fingerprinted {} technologies on {}",
                            detected.len(),
                            item.url
                        ),
                        Some(serde_json::to_value(&detected).unwrap_or(serde_json::Value::Null)),
                    );
                }

                // Link extraction if depth is within limits
                if item.depth < config.max_depth && ct.unwrap_or_default().contains("text/html") {
                    let extracted = extract_same_domain_links(&body, &final_url, &base_parsed);

                    // Prioritize links
                    let mut prioritized = Vec::new();
                    let mut normal = Vec::new();
                    for link in extracted {
                        if !visited.contains(&link) {
                            let link_l = link.to_lowercase();
                            if link_l.contains("login")
                                || link_l.contains("about")
                                || link_l.contains("contact")
                                || link_l.contains("dashboard")
                                || link_l.contains("admin")
                            {
                                prioritized.push(link);
                            } else {
                                normal.push(link);
                            }
                        }
                    }

                    // Add prioritized first, then normal
                    for link in prioritized {
                        queue.push_back(QueueItem {
                            url: link,
                            depth: item.depth + 1,
                        });
                    }
                    for link in normal {
                        queue.push_back(QueueItem {
                            url: link,
                            depth: item.depth + 1,
                        });
                    }
                }
            }
            Err(e) => {
                emit_event(
                    "FETCH_ERROR",
                    "ERROR",
                    format!("Failed to fetch {}: {}", item.url, e),
                    None,
                );
            }
        }
    }

    emit_event(
        "CRAWL_COMPLETE",
        "INFO",
        format!("Crawl complete. Scanned {} pages.", pages.len()),
        None,
    );

    Ok(CrawlResult {
        pages,
        events,
        final_url: main_final_url,
        main_chain,
    })
}

// Helper to manually track redirects and return final results
async fn fetch_manual_redirects(
    client: &Client,
    start_url: &str,
) -> Result<
    (
        String,
        u16,
        HashMap<String, String>,
        String,
        u64,
        Vec<RedirectChainEntry>,
    ),
    String,
> {
    let mut current_url = start_url.to_string();
    let mut redirect_chain = Vec::new();
    let mut redirect_count = 0;
    let max_redirects = 5;

    let total_start = Instant::now();

    loop {
        let resp_result = client.get(&current_url).send().await;

        let resp = match resp_result {
            Ok(r) => r,
            Err(e) => {
                redirect_chain.push(RedirectChainEntry {
                    url: current_url.clone(),
                    status_code: 0,
                });
                return Err(e.to_string());
            }
        };

        let status = resp.status();
        let status_code = status.as_u16();

        redirect_chain.push(RedirectChainEntry {
            url: current_url.clone(),
            status_code,
        });

        if status.is_redirection() {
            if redirect_count >= max_redirects {
                return Err("Too many redirects".to_string());
            }
            if let Some(loc) = resp.headers().get(reqwest::header::LOCATION) {
                if let Ok(loc_str) = loc.to_str() {
                    let mut next_url = loc_str.to_string();
                    if !next_url.starts_with("http") {
                        if let Ok(p_cur) = Url::parse(&current_url) {
                            if let Ok(p_next) = p_cur.join(&next_url) {
                                next_url = p_next.to_string();
                            }
                        }
                    }
                    current_url = next_url;
                    redirect_count += 1;
                    continue;
                }
            }
            return Err("Redirect status but no location".to_string());
        } else {
            let mut headers_map = HashMap::new();
            for (name, value) in resp.headers() {
                headers_map.insert(
                    name.as_str().to_lowercase(),
                    value.to_str().unwrap_or("").to_string(),
                );
            }
            let body = resp.text().await.unwrap_or_default();
            let latency_ms = total_start.elapsed().as_millis() as u64;

            return Ok((
                current_url,
                status_code,
                headers_map,
                body,
                latency_ms,
                redirect_chain,
            ));
        }
    }
}

fn extract_same_domain_links(html: &str, current_url: &str, base_parsed: &Url) -> Vec<String> {
    let mut links = HashSet::new();
    let document = Html::parse_document(html);
    if let Ok(selector) = Selector::parse("a[href]") {
        for element in document.select(&selector) {
            if let Some(href) = element.value().attr("href") {
                if let Ok(parsed_current) = Url::parse(current_url) {
                    if let Ok(joined) = parsed_current.join(href) {
                        // Check if same domain
                        if joined.host_str() == base_parsed.host_str() {
                            let mut clean_url = joined.clone();
                            clean_url.set_fragment(None);
                            links.insert(clean_url.to_string());
                        }
                    }
                }
            }
        }
    }
    links.into_iter().collect()
}
