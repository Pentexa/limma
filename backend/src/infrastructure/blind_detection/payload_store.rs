use std::collections::HashMap;
use std::sync::RwLock;
use uuid::Uuid;

/// Second-order payload tracking store.
///
/// Persists injected payloads in memory and tracks whether they were
/// later executed (detected via OOB callback or DOM observation).
/// This enables detection of stored/second-order vulnerabilities
/// where the payload is injected in one request but triggered in another.
pub struct PayloadStore {
    /// Map of payload_hash -> PayloadRecord
    records: RwLock<HashMap<String, PayloadRecord>>,
}

/// A tracked payload injection record.
#[derive(Debug, Clone)]
pub struct PayloadRecord {
    /// Unique identifier for this payload injection
    pub id: Uuid,
    /// The target URL where the payload was injected
    pub target_url: String,
    /// The parameter name used for injection
    pub parameter: String,
    /// The actual payload string that was sent
    pub payload: String,
    /// Hash of the payload for quick lookup
    pub payload_hash: String,
    /// The vulnerability type being tested
    pub vuln_type: String,
    /// Timestamp when the payload was injected
    pub injected_at: chrono::DateTime<chrono::Utc>,
    /// Whether this payload was detected as executed (triggered)
    pub executed: bool,
    /// Timestamp when execution was detected (if any)
    pub executed_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Callback ID used to correlate OOB interactions
    pub callback_id: Option<String>,
}

impl Default for PayloadStore {
    fn default() -> Self {
        Self::new()
    }
}

impl PayloadStore {
    pub fn new() -> Self {
        Self {
            records: RwLock::new(HashMap::new()),
        }
    }

    /// Save a new payload injection record for later tracking.
    pub fn save_payload(
        &self,
        target_url: &str,
        parameter: &str,
        payload: &str,
        vuln_type: &str,
        callback_id: Option<String>,
    ) -> String {
        let payload_hash = Self::hash_payload(payload);
        let record = PayloadRecord {
            id: Uuid::new_v4(),
            target_url: target_url.to_string(),
            parameter: parameter.to_string(),
            payload: payload.to_string(),
            payload_hash: payload_hash.clone(),
            vuln_type: vuln_type.to_string(),
            injected_at: chrono::Utc::now(),
            executed: false,
            executed_at: None,
            callback_id,
        };

        if let Ok(mut records) = self.records.write() {
            records.insert(payload_hash.clone(), record);
        }

        tracing::debug!(
            "[PayloadStore] Saved payload for {} param='{}' hash={}",
            target_url,
            parameter,
            payload_hash
        );

        payload_hash
    }

    /// Check if a payload with the given hash exists and return its record.
    pub fn track_payload(&self, payload_hash: &str) -> Option<PayloadRecord> {
        self.records
            .read()
            .ok()
            .and_then(|records| records.get(payload_hash).cloned())
    }

    /// Look up a payload by its callback ID (used when OOB interaction arrives).
    pub fn find_by_callback_id(&self, callback_id: &str) -> Option<PayloadRecord> {
        self.records.read().ok().and_then(|records| {
            records
                .values()
                .find(|r| r.callback_id.as_deref() == Some(callback_id))
                .cloned()
        })
    }

    /// Mark a payload as executed (triggered) — called when a second-order
    /// execution is detected via OOB callback, DOM observation, or timing anomaly.
    pub fn verify_execution(&self, payload_hash: &str) -> bool {
        if let Ok(mut records) = self.records.write() {
            if let Some(record) = records.get_mut(payload_hash) {
                record.executed = true;
                record.executed_at = Some(chrono::Utc::now());
                tracing::info!(
                    "[PayloadStore] Payload execution confirmed: hash={} target={} param={}",
                    payload_hash,
                    record.target_url,
                    record.parameter
                );
                return true;
            }
        }
        false
    }

    /// Mark a payload as executed by callback ID.
    pub fn verify_execution_by_callback(&self, callback_id: &str) -> bool {
        if let Ok(mut records) = self.records.write() {
            for record in records.values_mut() {
                if record.callback_id.as_deref() == Some(callback_id) {
                    record.executed = true;
                    record.executed_at = Some(chrono::Utc::now());
                    tracing::info!(
                        "[PayloadStore] Payload execution confirmed via callback: id={} target={}",
                        callback_id,
                        record.target_url
                    );
                    return true;
                }
            }
        }
        false
    }

    /// Get all payloads that were confirmed as executed.
    pub fn get_executed_payloads(&self) -> Vec<PayloadRecord> {
        self.records
            .read()
            .ok()
            .map(|records| records.values().filter(|r| r.executed).cloned().collect())
            .unwrap_or_default()
    }

    /// Get all pending (unconfirmed) payloads for a target.
    pub fn get_pending_for_target(&self, target_url: &str) -> Vec<PayloadRecord> {
        self.records
            .read()
            .ok()
            .map(|records| {
                records
                    .values()
                    .filter(|r| !r.executed && r.target_url == target_url)
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Purge expired records older than the specified duration.
    pub fn purge_older_than(&self, max_age: chrono::Duration) {
        let cutoff = chrono::Utc::now() - max_age;
        if let Ok(mut records) = self.records.write() {
            let before = records.len();
            records.retain(|_, r| r.injected_at > cutoff);
            let purged = before - records.len();
            if purged > 0 {
                tracing::info!("[PayloadStore] Purged {} expired payload records", purged);
            }
        }
    }

    /// Simple hash function for payload deduplication/lookup.
    fn hash_payload(payload: &str) -> String {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        payload.hash(&mut hasher);
        format!("{:016x}", hasher.finish())
    }
}
