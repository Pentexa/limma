use crate::domain::entities::*;
use crate::infrastructure::rule_engine::{
    build_context_from_headers, DynamicRuleEngine, DynamicRuleFinding,
};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tokio::sync::broadcast;

/// In-memory Burp Bridge session manager.
///
/// Tracks active plugin connections and their imported traffic.
/// Phase 1: in-memory only. Phase 2+: optionally persisted to DB.
pub struct BurpBridgeManager {
    sessions: RwLock<HashMap<String, BurpBridgeSession>>,
    traffic_store: RwLock<HashMap<String, Vec<BurpTrafficItem>>>,
    findings_store: RwLock<HashMap<String, Vec<BurpNativeFinding>>>,
    event_channels: RwLock<HashMap<String, broadcast::Sender<BurpSseEvent>>>,
}

impl BurpBridgeManager {
    pub fn new() -> Self {
        Self {
            sessions: RwLock::new(HashMap::new()),
            traffic_store: RwLock::new(HashMap::new()),
            findings_store: RwLock::new(HashMap::new()),
            event_channels: RwLock::new(HashMap::new()),
        }
    }

    /// Create a new bridge session from a handshake request.
    pub fn create_session(&self, req: &BurpHandshakeRequest) -> BurpHandshakeResponse {
        let session_id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now();

        let session = BurpBridgeSession {
            session_id: session_id.clone(),
            target_url: req.target_url.clone(),
            burp_version: req.burp_version.clone(),
            plugin_version: req.plugin_version.clone(),
            connected_at: now,
            last_heartbeat: now,
            imported_traffic_count: 0,
            exported_findings_count: 0,
            status: BurpSessionStatus::Connected,
        };

        if let Ok(mut sessions) = self.sessions.write() {
            sessions.insert(session_id.clone(), session);
        }

        // Initialize empty traffic and findings stores for this session
        if let Ok(mut store) = self.traffic_store.write() {
            store.insert(session_id.clone(), Vec::new());
        }
        if let Ok(mut findings) = self.findings_store.write() {
            findings.insert(session_id.clone(), Vec::new());
        }
        // Initialize SSE broadcast channel
        let (tx, _rx) = broadcast::channel(100);
        if let Ok(mut channels) = self.event_channels.write() {
            channels.insert(session_id.clone(), tx);
        }

        tracing::info!(
            "[BurpBridge] New session created: {} for target: {}",
            session_id,
            req.target_url
        );

        BurpHandshakeResponse {
            session_id,
            status: BurpSessionStatus::Connected,
            server_version: "0.1.0".to_string(),
            capabilities: vec![
                "traffic-import".to_string(),
                "findings-export".to_string(),
                "handshake".to_string(),
            ],
        }
    }

    /// Import a batch of traffic items into a session.
    pub fn import_traffic(
        &self,
        session_id: &str,
        items: Vec<BurpTrafficItem>,
        rule_engine: &DynamicRuleEngine,
    ) -> Result<BurpImportTrafficResponse, String> {
        // Verify session exists
        let session_exists = self
            .sessions
            .read()
            .map(|s| s.contains_key(session_id))
            .unwrap_or(false);

        if !session_exists {
            return Err(format!("Session not found: {}", session_id));
        }

        let count = items.len();
        let mut new_findings_count = 0;
        let mut burp_findings = Vec::new();

        // 1. Process items through Rule Engine
        for item in &items {
            let ctx = build_context_from_headers(
                &item.url,
                item.response_status,
                &item.response_headers,
                item.response_body.as_deref(),
            );

            let dynamic_findings = rule_engine.evaluate(&ctx);
            new_findings_count += dynamic_findings.len();

            // Map DynamicRuleFinding to BurpNativeFinding
            let (host, port, protocol) = parse_url_parts(&item.url);

            for f in dynamic_findings {
                let detail = format!(
                    "<b>{}</b><br/>{}<br/><br/>Rule ID: {}<br/>Confidence: {}",
                    f.rule_name, f.description.key, f.rule_id, f.effective_confidence
                );

                let native_finding = BurpNativeFinding {
                    name: f.rule_name,
                    detail,
                    severity: f.severity, // DynamicRuleFinding severity string
                    confidence: f.effective_confidence,
                    url: item.url.clone(),
                    path: ctx.path.clone(),
                    host: host.clone(),
                    port,
                    protocol: protocol.clone(),
                    remediation:
                        "Please review the finding details and apply appropriate security controls."
                            .to_string(),
                    issue_type: 0x08000000,
                    cwe_id: None,
                };

                // Broadcast the finding in real-time
                if let Some(tx) = self
                    .event_channels
                    .read()
                    .ok()
                    .and_then(|c| c.get(session_id).cloned())
                {
                    let _ = tx.send(BurpSseEvent::FindingDetected(Box::new(native_finding.clone())));
                }

                burp_findings.push(native_finding);
            }
        }

        // 2. Store traffic items
        if let Ok(mut store) = self.traffic_store.write() {
            let entry = store.entry(session_id.to_string()).or_insert_with(Vec::new);
            entry.extend(items);
        }

        // 3. Store generated findings
        if !burp_findings.is_empty() {
            if let Ok(mut f_store) = self.findings_store.write() {
                let entry = f_store
                    .entry(session_id.to_string())
                    .or_insert_with(Vec::new);
                entry.extend(burp_findings);
            }
        }

        // 4. Update session counters
        if let Ok(mut sessions) = self.sessions.write() {
            if let Some(session) = sessions.get_mut(session_id) {
                session.imported_traffic_count += count;
                session.exported_findings_count += new_findings_count;
                session.last_heartbeat = chrono::Utc::now();
                session.status = BurpSessionStatus::Syncing;
            }
        }

        tracing::info!(
            "[BurpBridge] Imported {} traffic items for session: {}. Triggered {} findings.",
            count,
            session_id,
            new_findings_count
        );

        Ok(BurpImportTrafficResponse {
            imported_count: count,
            session_id: session_id.to_string(),
            new_findings_triggered: new_findings_count,
            enrichment_notes: vec![
                format!("{} HTTP exchanges stored and analyzed", count),
                format!("{} new security findings generated", new_findings_count),
            ],
        })
    }

    /// Subscribe to real-time events for a session.
    pub fn subscribe_to_events(
        &self,
        session_id: &str,
    ) -> Option<broadcast::Receiver<BurpSseEvent>> {
        self.event_channels
            .read()
            .ok()
            .and_then(|channels| channels.get(session_id).map(|tx| tx.subscribe()))
    }

    /// Get all active sessions.
    pub fn get_all_sessions(&self) -> Vec<BurpBridgeSession> {
        self.sessions
            .read()
            .map(|s| s.values().cloned().collect())
            .unwrap_or_default()
    }

    /// Get the current session info.
    pub fn get_session(&self, session_id: &str) -> Option<BurpBridgeSession> {
        self.sessions.read().ok()?.get(session_id).cloned()
    }

    /// Get traffic items for a session.
    pub fn get_traffic(&self, session_id: &str) -> Vec<BurpTrafficItem> {
        self.traffic_store
            .read()
            .ok()
            .and_then(|store| store.get(session_id).cloned())
            .unwrap_or_default()
    }

    /// Get traffic count for a session.
    pub fn get_traffic_count(&self, session_id: &str) -> usize {
        self.traffic_store
            .read()
            .ok()
            .and_then(|store| store.get(session_id).map(|v| v.len()))
            .unwrap_or(0)
    }

    /// Get all native findings for a session.
    pub fn get_session_findings(&self, session_id: &str) -> Vec<BurpNativeFinding> {
        self.findings_store
            .read()
            .ok()
            .and_then(|store| store.get(session_id).cloned())
            .unwrap_or_default()
    }

    /// Convert a NormalizedAuditReport's findings into Burp-native format.
    pub fn findings_to_burp_native(report: &NormalizedAuditReport) -> Vec<BurpNativeFinding> {
        let (host, port, protocol) = parse_url_parts(&report.target);

        report
            .canonical_findings
            .iter()
            .map(|cf| {
                let severity = match cf.severity {
                    SeverityLevel::Critical | SeverityLevel::High => "High".to_string(),
                    SeverityLevel::Medium => "Medium".to_string(),
                    SeverityLevel::Low => "Low".to_string(),
                    SeverityLevel::Informational => "Information".to_string(),
                };

                let confidence = match cf.confidence {
                    ConfidenceLevel::Certain => "Certain".to_string(),
                    ConfidenceLevel::Firm => "Firm".to_string(),
                    ConfidenceLevel::Tentative | ConfidenceLevel::Low => "Tentative".to_string(),
                };

                let path = cf
                    .affected_routes
                    .first()
                    .cloned()
                    .unwrap_or_else(|| "/".to_string());

                let detail = format!(
                    "<b>{}</b><br/><br/>Modules: {}<br/>Evidence count: {}<br/>{}",
                    cf.title,
                    cf.contributing_modules
                        .iter()
                        .map(|m| format!("{:?}", m))
                        .collect::<Vec<_>>()
                        .join(", "),
                    cf.merged_evidence_count,
                    cf.underlying_findings
                        .iter()
                        .flat_map(|f| &f.evidence)
                        .map(|e| format!("• {}", e.description))
                        .collect::<Vec<_>>()
                        .join("<br/>")
                );

                BurpNativeFinding {
                    name: cf.title.clone(),
                    detail,
                    severity,
                    confidence,
                    url: format!("{}://{}:{}{}", protocol, host, port, path),
                    path,
                    host: host.clone(),
                    port,
                    protocol: protocol.clone(),
                    remediation: String::new(), // Phase 2: populate from rule metadata
                    issue_type: 0x08000000,     // Extension-generated issue
                    cwe_id: None,               // Phase 2: CWE mapping
                }
            })
            .collect()
    }
}

/// Parse URL into (host, port, protocol) tuple.
fn parse_url_parts(url: &str) -> (String, i32, String) {
    let parsed = url::Url::parse(url).unwrap_or_else(|_| {
        url::Url::parse(&format!("https://{}", url))
            .unwrap_or_else(|_| url::Url::parse("https://unknown").unwrap())
    });
    let host = parsed.host_str().unwrap_or("unknown").to_string();
    let protocol = parsed.scheme().to_string();
    let port = parsed
        .port()
        .unwrap_or(if protocol == "https" { 443 } else { 80 }) as i32;
    (host, port, protocol)
}

pub type SharedBurpBridgeManager = Arc<BurpBridgeManager>;
