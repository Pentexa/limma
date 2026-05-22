use dashmap::DashMap;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;

/// Global monitor for Web Application Firewall (WAF) triggers.
/// Keeps track of suspicious HTTP status codes (403, 406, 429) per domain.
#[derive(Clone)]
pub struct WafMonitor {
    // Map of domain -> consecutive blocked requests
    blocked_counters: Arc<DashMap<String, AtomicU32>>,
    // Map of domain -> is WAF currently active/detected
    waf_detected: Arc<DashMap<String, AtomicBool>>,
    // Map of domain -> is Circuit Open (stop scanning)
    circuit_open: Arc<DashMap<String, AtomicBool>>,
}

impl WafMonitor {
    pub fn new() -> Self {
        Self {
            blocked_counters: Arc::new(DashMap::new()),
            waf_detected: Arc::new(DashMap::new()),
            circuit_open: Arc::new(DashMap::new()),
        }
    }

    /// Register an HTTP status code for a target URL.
    /// Returns true if a WAF tripwire has been activated (e.g. 3 consecutive blocks).
    pub fn register_response(&self, target_url: &str, status_code: u16) -> bool {
        let domain = extract_domain(target_url);

        let is_blocked = status_code == 403 || status_code == 406 || status_code == 429;

        let counter_entry = self
            .blocked_counters
            .entry(domain.clone())
            .or_insert_with(|| AtomicU32::new(0));
        let detected_entry = self
            .waf_detected
            .entry(domain.clone())
            .or_insert_with(|| AtomicBool::new(false));
        let circuit_entry = self
            .circuit_open
            .entry(domain.clone())
            .or_insert_with(|| AtomicBool::new(false));

        if is_blocked {
            let count = counter_entry.fetch_add(1, Ordering::SeqCst) + 1;
            if count >= 3 {
                detected_entry.store(true, Ordering::SeqCst);
            }
            if count >= 10 {
                circuit_entry.store(true, Ordering::SeqCst);
            }
        } else {
            // Reset counter on successful/normal response
            counter_entry.store(0, Ordering::SeqCst);
        }

        detected_entry.load(Ordering::SeqCst)
    }

    /// Fingerprints the WAF based on response headers or body.
    /// Updates the monitor state if a WAF is positively identified.
    pub fn fingerprint_waf(
        &self,
        target_url: &str,
        headers: &std::collections::HashMap<String, String>,
        _body: &str,
    ) -> Option<String> {
        let domain = extract_domain(target_url);
        let mut identified_waf = None;

        // Check common WAF headers
        if headers.contains_key("cf-ray")
            || headers
                .get("server")
                .map_or(false, |v| v.to_lowercase().contains("cloudflare"))
        {
            identified_waf = Some("Cloudflare".to_string());
        } else if headers.contains_key("x-amzn-requestid")
            || headers
                .get("server")
                .map_or(false, |v| v.to_lowercase().contains("awselb"))
        {
            identified_waf = Some("AWS WAF".to_string());
        } else if headers
            .get("server")
            .map_or(false, |v| v.to_lowercase().contains("akamai"))
        {
            identified_waf = Some("Akamai".to_string());
        } else if headers.contains_key("x-sucuri-id") {
            identified_waf = Some("Sucuri".to_string());
        }

        if let Some(waf_name) = &identified_waf {
            tracing::debug!(
                "[WafMonitor] Fingerprinted WAF: {} for domain {}",
                waf_name,
                domain
            );
            let detected_entry = self
                .waf_detected
                .entry(domain)
                .or_insert_with(|| AtomicBool::new(false));
            detected_entry.store(true, Ordering::SeqCst);
        }

        identified_waf
    }

    /// Check if WAF is currently detected for a given URL
    pub fn is_waf_detected(&self, target_url: &str) -> bool {
        let domain = extract_domain(target_url);
        if let Some(entry) = self.waf_detected.get(&domain) {
            entry.load(Ordering::SeqCst)
        } else {
            false
        }
    }

    /// Check if Circuit is Open (scanning should stop)
    pub fn is_circuit_open(&self, target_url: &str) -> bool {
        let domain = extract_domain(target_url);
        if let Some(entry) = self.circuit_open.get(&domain) {
            entry.load(Ordering::SeqCst)
        } else {
            false
        }
    }

    /// Get total blocked requests for a domain
    pub fn get_blocked_count(&self, target_url: &str) -> u32 {
        let domain = extract_domain(target_url);
        if let Some(entry) = self.blocked_counters.get(&domain) {
            entry.load(Ordering::SeqCst)
        } else {
            0
        }
    }
}

fn extract_domain(url: &str) -> String {
    url::Url::parse(url)
        .ok()
        .and_then(|u| u.host_str().map(|h| h.to_string()))
        .unwrap_or_else(|| url.to_string())
}
