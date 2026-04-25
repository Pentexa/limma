use crate::domain::entities::*;
use crate::infrastructure::rule_engine::{build_context_from_headers, DynamicRuleEngine};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tokio::sync::broadcast;

/// Persistent Burp Bridge session manager.
///
/// Maintains an in-memory write-through cache backed by PostgreSQL.
/// Sessions, traffic, and findings survive server restarts.
/// SSE broadcast channels are always in-memory (transient by nature).
pub struct BurpBridgeManager {
    sessions: RwLock<HashMap<String, BurpBridgeSession>>,
    traffic_store: RwLock<HashMap<String, Vec<BurpTrafficItem>>>,
    findings_store: RwLock<HashMap<String, Vec<BurpNativeFinding>>>,
    event_channels: RwLock<HashMap<String, broadcast::Sender<BurpSseEvent>>>,
    pool: sqlx::PgPool,
}

impl BurpBridgeManager {
    /// Create manager and hydrate in-memory caches from the database.
    pub async fn new(pool: sqlx::PgPool) -> Self {
        let manager = Self {
            sessions: RwLock::new(HashMap::new()),
            traffic_store: RwLock::new(HashMap::new()),
            findings_store: RwLock::new(HashMap::new()),
            event_channels: RwLock::new(HashMap::new()),
            pool,
        };

        // Restore persisted state on startup
        if let Err(e) = manager.hydrate_from_db().await {
            tracing::warn!("[BurpBridge] Failed to hydrate from database: {}", e);
        }

        manager
    }

    /// Hydrate in-memory caches from the database on startup.
    async fn hydrate_from_db(&self) -> Result<(), String> {
        // 1. Restore sessions
        #[derive(sqlx::FromRow)]
        struct SessionRow {
            session_id: String,
            target_url: String,
            burp_version: Option<String>,
            plugin_version: Option<String>,
            connected_at: chrono::DateTime<chrono::Utc>,
            last_heartbeat: chrono::DateTime<chrono::Utc>,
            imported_traffic_count: i32,
            exported_findings_count: i32,
            status: String,
        }

        let session_rows: Vec<SessionRow> = sqlx::query_as(
            "SELECT session_id, target_url, burp_version, plugin_version, connected_at, last_heartbeat, imported_traffic_count, exported_findings_count, status FROM burp_sessions"
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to load sessions: {}", e))?;

        let session_count = session_rows.len();

        if let Ok(mut sessions) = self.sessions.write() {
            for row in &session_rows {
                let status = match row.status.as_str() {
                    "connected" => BurpSessionStatus::Connected,
                    "syncing" => BurpSessionStatus::Syncing,
                    "idle" => BurpSessionStatus::Idle,
                    _ => BurpSessionStatus::Disconnected,
                };

                sessions.insert(
                    row.session_id.clone(),
                    BurpBridgeSession {
                        session_id: row.session_id.clone(),
                        target_url: row.target_url.clone(),
                        burp_version: row.burp_version.clone(),
                        plugin_version: row.plugin_version.clone(),
                        connected_at: row.connected_at,
                        last_heartbeat: row.last_heartbeat,
                        imported_traffic_count: row.imported_traffic_count as usize,
                        exported_findings_count: row.exported_findings_count as usize,
                        status,
                    },
                );
            }
        }

        // 2. Restore traffic items per session
        #[derive(sqlx::FromRow)]
        struct TrafficRow {
            session_id: String,
            url: String,
            method: String,
            request_headers: serde_json::Value,
            request_body: Option<String>,
            response_status: i32,
            response_headers: serde_json::Value,
            response_body: Option<String>,
            timestamp: i64,
            tool_source: String,
        }

        let traffic_rows: Vec<TrafficRow> = sqlx::query_as(
            "SELECT session_id, url, method, request_headers, request_body, response_status, response_headers, response_body, timestamp, tool_source FROM burp_traffic_items ORDER BY id ASC"
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to load traffic: {}", e))?;

        let traffic_count = traffic_rows.len();

        if let Ok(mut store) = self.traffic_store.write() {
            for row in traffic_rows {
                let req_headers: HashMap<String, String> =
                    serde_json::from_value(row.request_headers).unwrap_or_default();
                let resp_headers: HashMap<String, String> =
                    serde_json::from_value(row.response_headers).unwrap_or_default();

                let item = BurpTrafficItem {
                    url: row.url,
                    method: row.method,
                    request_headers: req_headers,
                    request_body: row.request_body,
                    response_status: row.response_status as u16,
                    response_headers: resp_headers,
                    response_body: row.response_body,
                    timestamp: row.timestamp,
                    tool_source: row.tool_source,
                };

                store
                    .entry(row.session_id)
                    .or_insert_with(Vec::new)
                    .push(item);
            }
        }

        // 3. Restore findings per session
        #[derive(sqlx::FromRow)]
        struct FindingRow {
            session_id: String,
            name: String,
            detail: String,
            severity: String,
            confidence: String,
            url: String,
            path: String,
            host: String,
            port: i32,
            protocol: String,
            remediation: String,
            issue_type: i32,
            cwe_id: Option<i32>,
        }

        let finding_rows: Vec<FindingRow> = sqlx::query_as(
            "SELECT session_id, name, detail, severity, confidence, url, path, host, port, protocol, remediation, issue_type, cwe_id FROM burp_findings ORDER BY id ASC"
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to load findings: {}", e))?;

        let findings_count = finding_rows.len();

        if let Ok(mut f_store) = self.findings_store.write() {
            for row in finding_rows {
                let finding = BurpNativeFinding {
                    name: row.name,
                    detail: row.detail,
                    severity: row.severity,
                    confidence: row.confidence,
                    url: row.url,
                    path: row.path,
                    host: row.host,
                    port: row.port,
                    protocol: row.protocol,
                    remediation: row.remediation,
                    issue_type: row.issue_type as u32,
                    cwe_id: row.cwe_id.map(|v| v as u32),
                };

                f_store
                    .entry(row.session_id)
                    .or_insert_with(Vec::new)
                    .push(finding);
            }
        }

        // 4. Initialize SSE channels for restored sessions
        if let Ok(mut channels) = self.event_channels.write() {
            for row in &session_rows {
                let (tx, _rx) = broadcast::channel(100);
                channels.insert(row.session_id.clone(), tx);
            }
        }

        tracing::info!(
            "[BurpBridge] Hydrated from DB: {} sessions, {} traffic items, {} findings",
            session_count,
            traffic_count,
            findings_count
        );

        Ok(())
    }

    /// Create a new bridge session from a handshake request.
    /// Persists session to the database immediately.
    pub async fn create_session(&self, req: &BurpHandshakeRequest) -> BurpHandshakeResponse {
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

        // Write to in-memory cache
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

        // Persist to database
        let status_str = "connected";
        if let Err(e) = sqlx::query(
            r#"INSERT INTO burp_sessions
               (session_id, target_url, burp_version, plugin_version, connected_at, last_heartbeat, imported_traffic_count, exported_findings_count, status)
               VALUES ($1, $2, $3, $4, $5, $6, 0, 0, $7)"#,
        )
        .bind(&session_id)
        .bind(&req.target_url)
        .bind(&req.burp_version)
        .bind(&req.plugin_version)
        .bind(now)
        .bind(now)
        .bind(status_str)
        .execute(&self.pool)
        .await
        {
            tracing::error!("[BurpBridge] Failed to persist session {}: {}", session_id, e);
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
    /// Persists traffic and generated findings to the database.
    pub async fn import_traffic(
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
                    let _ = tx.send(BurpSseEvent::FindingDetected(Box::new(
                        native_finding.clone(),
                    )));
                }

                burp_findings.push(native_finding);
            }
        }

        // 2. Store traffic items (in-memory)
        if let Ok(mut store) = self.traffic_store.write() {
            let entry = store.entry(session_id.to_string()).or_insert_with(Vec::new);
            entry.extend(items.clone());
        }

        // 3. Persist traffic items to database
        for item in &items {
            let req_headers_json = serde_json::to_value(&item.request_headers).unwrap_or_default();
            let resp_headers_json =
                serde_json::to_value(&item.response_headers).unwrap_or_default();

            if let Err(e) = sqlx::query(
                r#"INSERT INTO burp_traffic_items
                   (session_id, url, method, request_headers, request_body, response_status, response_headers, response_body, timestamp, tool_source)
                   VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)"#,
            )
            .bind(session_id)
            .bind(&item.url)
            .bind(&item.method)
            .bind(&req_headers_json)
            .bind(&item.request_body)
            .bind(item.response_status as i32)
            .bind(&resp_headers_json)
            .bind(&item.response_body)
            .bind(item.timestamp)
            .bind(&item.tool_source)
            .execute(&self.pool)
            .await
            {
                tracing::error!(
                    "[BurpBridge] Failed to persist traffic item for session {}: {}",
                    session_id,
                    e
                );
            }
        }

        // 4. Store generated findings (in-memory)
        if !burp_findings.is_empty() {
            if let Ok(mut f_store) = self.findings_store.write() {
                let entry = f_store
                    .entry(session_id.to_string())
                    .or_insert_with(Vec::new);
                entry.extend(burp_findings.clone());
            }
        }

        // 5. Persist findings to database
        for finding in &burp_findings {
            if let Err(e) = sqlx::query(
                r#"INSERT INTO burp_findings
                   (session_id, name, detail, severity, confidence, url, path, host, port, protocol, remediation, issue_type, cwe_id)
                   VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)"#,
            )
            .bind(session_id)
            .bind(&finding.name)
            .bind(&finding.detail)
            .bind(&finding.severity)
            .bind(&finding.confidence)
            .bind(&finding.url)
            .bind(&finding.path)
            .bind(&finding.host)
            .bind(finding.port)
            .bind(&finding.protocol)
            .bind(&finding.remediation)
            .bind(finding.issue_type as i32)
            .bind(finding.cwe_id.map(|v| v as i32))
            .execute(&self.pool)
            .await
            {
                tracing::error!(
                    "[BurpBridge] Failed to persist finding for session {}: {}",
                    session_id,
                    e
                );
            }
        }

        // 6. Update session counters (in-memory + DB)
        let status_str;
        if let Ok(mut sessions) = self.sessions.write() {
            if let Some(session) = sessions.get_mut(session_id) {
                session.imported_traffic_count += count;
                session.exported_findings_count += new_findings_count;
                session.last_heartbeat = chrono::Utc::now();
                session.status = BurpSessionStatus::Syncing;
            }
        }
        status_str = "syncing";

        if let Err(e) = sqlx::query(
            r#"UPDATE burp_sessions
               SET imported_traffic_count = imported_traffic_count + $1,
                   exported_findings_count = exported_findings_count + $2,
                   last_heartbeat = $3,
                   status = $4
               WHERE session_id = $5"#,
        )
        .bind(count as i32)
        .bind(new_findings_count as i32)
        .bind(chrono::Utc::now())
        .bind(status_str)
        .bind(session_id)
        .execute(&self.pool)
        .await
        {
            tracing::error!(
                "[BurpBridge] Failed to update session counters for {}: {}",
                session_id,
                e
            );
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
    #[allow(dead_code)]
    pub fn get_traffic(&self, session_id: &str) -> Vec<BurpTrafficItem> {
        self.traffic_store
            .read()
            .ok()
            .and_then(|store| store.get(session_id).cloned())
            .unwrap_or_default()
    }

    /// Get traffic count for a session.
    #[allow(dead_code)]
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
    #[allow(dead_code)]
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
            .unwrap_or_else(|_| url::Url::parse("https://unknown").expect("static URL parse"))
    });
    let host = parsed.host_str().unwrap_or("unknown").to_string();
    let protocol = parsed.scheme().to_string();
    let port = parsed
        .port()
        .unwrap_or(if protocol == "https" { 443 } else { 80 }) as i32;
    (host, port, protocol)
}

pub type SharedBurpBridgeManager = Arc<BurpBridgeManager>;
