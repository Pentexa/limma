use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Instant;

use std::sync::Arc;
use tokio::sync::Mutex as TokioMutex;
use tokio::time::sleep;

/// Simple in-memory rate limiter for exploit operations.
///
/// Tracks request counts per target domain within a sliding window.
pub struct ExploitRateLimiter {
    max_per_minute: u32,
    counters: Mutex<HashMap<String, Vec<Instant>>>,
}

impl ExploitRateLimiter {
    pub fn new(max_per_minute: u32) -> Self {
        Self {
            max_per_minute,
            counters: Mutex::new(HashMap::new()),
        }
    }

    /// Check if a request to the target is within rate limits.
    /// Records the request if allowed.
    pub fn check(&self, target_url: &str) -> Result<(), String> {
        // Extract domain from URL for rate limiting
        let domain = extract_domain(target_url);

        let mut counters = self.counters.lock().map_err(|e| e.to_string())?;
        let now = Instant::now();
        let one_minute_ago = now - std::time::Duration::from_secs(60);

        let timestamps = counters.entry(domain.clone()).or_default();

        // Purge old entries
        timestamps.retain(|t| *t > one_minute_ago);

        if timestamps.len() as u32 >= self.max_per_minute {
            Err(format!(
                "Rate limit exceeded for '{}': {} requests/minute (max: {})",
                domain,
                timestamps.len(),
                self.max_per_minute
            ))
        } else {
            timestamps.push(now);
            Ok(())
        }
    }
}

/// Extract domain from a URL for rate limiting grouping
fn extract_domain(url: &str) -> String {
    url::Url::parse(url)
        .ok()
        .and_then(|u| u.host_str().map(|h| h.to_string()))
        .unwrap_or_else(|| url.to_string())
}

#[derive(Debug, Clone)]
pub struct SharedRateLimiter {
    delay_ms: u64,
    next_allowed: Arc<TokioMutex<Instant>>,
}

impl SharedRateLimiter {
    pub fn new(rate_limit_rps: u32) -> Self {
        let delay_ms = if rate_limit_rps > 0 {
            1000 / (rate_limit_rps as u64)
        } else {
            100 // fallback
        };

        Self {
            delay_ms,
            next_allowed: Arc::new(TokioMutex::new(Instant::now())),
        }
    }

    /// Acquires permission to send a request, sleeping if necessary to maintain the rate limit.
    pub async fn wait(&self) {
        if self.delay_ms == 0 {
            return;
        }

        let mut next = self.next_allowed.lock().await;
        let now = Instant::now();

        if *next > now {
            let sleep_duration = *next - now;
            sleep(sleep_duration).await;
        }

        *next = Instant::now() + std::time::Duration::from_millis(self.delay_ms);
    }
}
