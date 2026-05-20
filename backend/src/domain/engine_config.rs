use crate::domain::entities::SettingsProfile;
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq)]
pub enum FuzzingIntensity {
    Low,
    Medium,
    High,
    Aggressive,
}

#[derive(Debug, Clone)]
pub struct EngineConfig {
    // Shared Rate Limiter configuration
    pub rate_limit_rps: u32,
    pub rate_limiter:
        std::sync::Arc<crate::infrastructure::safety::rate_limiter::SharedRateLimiter>,
    pub timeout_ms: u64,
    pub use_proxy: bool,
    pub proxy_url: Option<String>,
    pub user_agent: String,

    // Scanner / Crawler
    pub max_depth: u32,
    pub follow_redirects: bool,
    pub wordlist: Vec<String>,
    pub max_wordlist_items_per_scan: usize,

    // Collector / Services
    pub target_ports: Vec<u16>,
    pub max_concurrent_ports: usize,

    // Active Detection / Forms
    pub fuzzing_intensity: FuzzingIntensity,
    pub avoid_waf: bool,

    // Exploit
    pub active_exploit_enabled: bool,
}

impl EngineConfig {
    pub fn from_profile(profile: &SettingsProfile) -> Self {
        // 1. Rate Limiting (Safe Default)
        let rate_limit_rps = if profile.global.rate_limit_req_per_sec > 0 {
            profile.global.rate_limit_req_per_sec
        } else {
            10 // Safe default
        };

        let rate_limiter = std::sync::Arc::new(
            crate::infrastructure::safety::rate_limiter::SharedRateLimiter::new(rate_limit_rps),
        );

        // 2. Timeout
        let timeout_ms = if profile.global.timeout_ms > 0 {
            profile.global.timeout_ms
        } else {
            10000
        };

        // 3. Wordlist Discovery
        let mut wordlist = Vec::new();
        let max_wordlist_items = match profile.scanner.wordlist_size.as_str() {
            "small" => {
                wordlist = vec!["admin", "login", "api", "test", ".git"]
                    .into_iter()
                    .map(String::from)
                    .collect();
                5
            }
            "medium" => {
                wordlist = vec![
                    "admin", "login", "api", "test", ".git", "backup", "config", "staging", "v1",
                    "v2", "swagger",
                ]
                .into_iter()
                .map(String::from)
                .collect();
                20
            }
            "large" | "massive" => {
                wordlist = vec![
                    "admin",
                    "login",
                    "api",
                    "test",
                    ".git",
                    "backup",
                    "config",
                    "staging",
                    "v1",
                    "v2",
                    "swagger",
                    "phpmyadmin",
                    "graphql",
                    "metrics",
                    "health",
                    ".env",
                    "actuator",
                ]
                .into_iter()
                .map(String::from)
                .collect();
                50
            }
            _ => 0,
        };

        // 4. Port Scan Range Parser
        let mut target_ports = HashSet::new();
        let range_str = profile.services.port_scan_range.to_lowercase();

        if range_str == "top-100" || range_str == "common" {
            target_ports.extend(vec![
                21, 22, 25, 53, 80, 110, 143, 443, 465, 587, 993, 995, 1433, 1521, 2049, 3306,
                3389, 5432, 6379, 8080, 8443,
            ]);
        } else if range_str == "top-1000" {
            // A realistic sample of top 1000 for this exercise, we can extend it later
            target_ports.extend(vec![
                21, 22, 23, 25, 53, 80, 110, 111, 135, 139, 143, 443, 445, 993, 995, 1723, 3306,
                3389, 5900, 8080,
            ]);
        } else {
            // parse comma separated and dash separated
            for part in range_str.split(',') {
                let part = part.trim();
                if part.is_empty() {
                    continue;
                }
                if part.contains('-') {
                    let bounds: Vec<&str> = part.split('-').collect();
                    if bounds.len() == 2 {
                        if let (Ok(start), Ok(end)) =
                            (bounds[0].parse::<u16>(), bounds[1].parse::<u16>())
                        {
                            let s = std::cmp::min(start, end);
                            let e = std::cmp::max(start, end);
                            for p in s..=e {
                                target_ports.insert(p);
                            }
                        }
                    }
                } else if let Ok(p) = part.parse::<u16>() {
                    target_ports.insert(p);
                }
            }
        }

        // If empty, safe fallback
        if target_ports.is_empty() {
            target_ports.extend(vec![80, 443]);
        }

        let mut target_ports_vec: Vec<u16> = target_ports.into_iter().collect();
        target_ports_vec.sort();

        // 5. Fuzzing Intensity
        let fuzzing_intensity = match profile.forms.fuzzing_intensity.as_str() {
            "low" => FuzzingIntensity::Low,
            "medium" => FuzzingIntensity::Medium,
            "high" => FuzzingIntensity::High,
            "aggressive" => FuzzingIntensity::Aggressive,
            _ => FuzzingIntensity::Medium,
        };

        // 6. Aggressive mode checks
        let is_aggressive = fuzzing_intensity == FuzzingIntensity::High
            || fuzzing_intensity == FuzzingIntensity::Aggressive;

        // Port range restriction: if not aggressive, cap at 1000 ports
        if !is_aggressive && target_ports_vec.len() > 1000 {
            target_ports_vec.truncate(1000);
        }

        // Active Exploit constraints: only if profile clearly allows it AND sandbox + manual approval are on
        let active_exploit_enabled = if profile.exploit.mode == "authorized_active" {
            profile.exploit.sandbox_validation && profile.exploit.manual_approval_required
        } else {
            false
        };

        let proxy_url = if profile.global.use_proxy && !profile.global.proxy_url.is_empty() {
            Some(profile.global.proxy_url.clone())
        } else {
            None
        };

        Self {
            rate_limit_rps,
            rate_limiter,
            timeout_ms,
            use_proxy: profile.global.use_proxy,
            proxy_url,
            user_agent: profile.scanner.user_agent.clone(),
            max_depth: profile.scanner.max_depth,
            follow_redirects: profile.scanner.follow_redirects,
            wordlist,
            max_wordlist_items_per_scan: max_wordlist_items,
            target_ports: target_ports_vec,
            max_concurrent_ports: 100, // Mandatory default limit
            fuzzing_intensity,
            avoid_waf: profile.forms.avoid_waf,
            active_exploit_enabled,
        }
    }
}
