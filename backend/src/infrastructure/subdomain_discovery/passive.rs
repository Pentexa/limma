use std::collections::HashSet;

pub struct PassiveCollector;

impl PassiveCollector {
    pub async fn collect_from_crtsh(domain: &str, client: &reqwest::Client) -> HashSet<String> {
        let mut subdomains = HashSet::new();
        let url = format!("https://crt.sh/?q=%.{}&output=json", domain);
        
        // Fallback safely if it fails
        if let Ok(res) = client.get(&url).send().await {
            if let Ok(text) = res.text().await {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
                    if let Some(arr) = json.as_array() {
                        for item in arr {
                            if let Some(name_value) = item.get("name_value") {
                                if let Some(name_str) = name_value.as_str() {
                                    for sub in name_str.split('\n') {
                                        let cleaned = sub.trim().trim_start_matches("*.").to_lowercase();
                                        if cleaned.ends_with(domain) && cleaned != domain {
                                            subdomains.insert(cleaned);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        } else {
            tracing::warn!("[SubdomainDiscovery] crt.sh query failed for {}, gracefully falling back.", domain);
        }
        
        subdomains
    }
}
