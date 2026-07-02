use sqlx::PgPool;
use std::time::Instant;

use std::sync::Arc;
use tokio::sync::Mutex as TokioMutex;
use tokio::time::sleep;

/// PostgreSQL-backed, process-safe rate limiter for exploit operations.
/// Tracks request counts per target domain in fixed one-minute windows.
pub struct ExploitRateLimiter {
    max_per_minute: u32,
    pool: PgPool,
}

impl ExploitRateLimiter {
    pub fn new(pool: PgPool, max_per_minute: u32) -> Self {
        Self {
            max_per_minute,
            pool,
        }
    }

    /// Check if a request to the target is within rate limits.
    /// Records the request if allowed.
    pub async fn check(&self, target_url: &str) -> Result<(), String> {
        // Extract domain from URL for rate limiting
        let domain = extract_domain(target_url);

        let count = sqlx::query_scalar::<_, i32>(
            "WITH cleanup AS (
                 DELETE FROM exploit_rate_limits WHERE window_start < NOW() - INTERVAL '1 day'
             )
             INSERT INTO exploit_rate_limits (target_domain, window_start, request_count)
             VALUES ($1, date_trunc('minute', NOW()), 1)
             ON CONFLICT (target_domain, window_start) DO UPDATE
             SET request_count = exploit_rate_limits.request_count + 1
             WHERE exploit_rate_limits.request_count < $2
             RETURNING request_count",
        )
        .bind(&domain)
        .bind(self.max_per_minute as i32)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Distributed rate limiter unavailable: {e}"))?;

        if count.is_none() {
            Err(format!(
                "Rate limit exceeded for '{}': maximum {} requests/minute",
                domain, self.max_per_minute
            ))
        } else {
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
