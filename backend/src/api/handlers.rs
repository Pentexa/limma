use crate::api::models::{
    AnalysisRequest, AuthResponse, CreateUserRequest, FeedbackRequest, LoginRequest, ProxyRequest,
    UserPublic, VerifyPortRequest, VerifyPortResponse,
};
use crate::application::use_cases::{
    AnalyzeWebsite, AuditSecurity, CollectExternalServices, DiscoverApis, GenerateMasterReport,
    InvestigateServer, LoginUser, MapForms, RegisterUser,
};
use crate::error::AppError;
use crate::infrastructure::auditor::HttpSecurityAuditor;
use crate::infrastructure::burp_bridge::SharedBurpBridgeManager;
use crate::infrastructure::collector::HttpServiceCollector;
use crate::infrastructure::discoverer::HttpApiDiscoverer;
use crate::infrastructure::investigator::HttpInvestigator;
use crate::infrastructure::mapper::HttpFormMapper;
use crate::infrastructure::persistence::PgUserRepository;
use crate::infrastructure::rule_engine::SharedDynamicRuleEngine;
use crate::infrastructure::scanner::HttpWebsiteScanner;
use axum::{extract::State, http::StatusCode, Json};
use std::sync::Arc;

pub struct AppState {
    pub user_repo: Arc<PgUserRepository>,
    pub website_scanner: Arc<HttpWebsiteScanner>,
    pub server_investigator: Arc<HttpInvestigator>,
    pub api_discoverer: Arc<HttpApiDiscoverer>,
    pub service_collector: Arc<HttpServiceCollector>,
    pub security_auditor: Arc<HttpSecurityAuditor>,
    pub form_mapper: Arc<HttpFormMapper>,
    pub dynamic_rule_engine: SharedDynamicRuleEngine,
    pub burp_bridge: SharedBurpBridgeManager,
    pub delta_engine: Arc<crate::infrastructure::delta_engine::DeltaEngine>,
    pub db_pool: sqlx::PgPool,
    pub jwt_secret: String,
}

pub async fn register_user(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateUserRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    let use_case = RegisterUser {
        repo: &*state.user_repo,
    };

    match use_case
        .execute(payload.name, payload.email, payload.password)
        .await
    {
        Ok(user) => {
            // Auto-login after registration: create token
            let token = crate::infrastructure::auth::create_token(user.id, &state.jwt_secret)
                .map_err(AppError::Internal)?;

            let response = AuthResponse {
                token,
                user: UserPublic {
                    id: user.id.to_string(),
                    name: user.name,
                    email: user.email,
                },
            };
            let value = serde_json::to_value(response)?;
            Ok((StatusCode::CREATED, Json(value)))
        }
        Err(e) => Err(AppError::BadRequest(e)),
    }
}

pub async fn login_user(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let use_case = LoginUser {
        repo: &*state.user_repo,
        jwt_secret: &state.jwt_secret,
    };

    match use_case.execute(payload.email, payload.password).await {
        Ok((user, token)) => {
            let response = AuthResponse {
                token,
                user: UserPublic {
                    id: user.id.to_string(),
                    name: user.name,
                    email: user.email,
                },
            };
            let value = serde_json::to_value(response)?;
            Ok(Json(value))
        }
        Err(e) => Err(AppError::BadRequest(e)),
    }
}

pub async fn get_me(
    State(state): State<Arc<AppState>>,
    req: axum::http::Request<axum::body::Body>,
) -> Result<Json<serde_json::Value>, AppError> {
    // Extract token from Authorization header
    let auth_header = req
        .headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| AppError::BadRequest("Missing Authorization header".to_string()))?;

    let token = auth_header.strip_prefix("Bearer ").ok_or_else(|| {
        AppError::BadRequest("Invalid Authorization format. Use: Bearer <token>".to_string())
    })?;

    let claims = crate::infrastructure::auth::verify_token(token, &state.jwt_secret)
        .map_err(AppError::BadRequest)?;

    let user_id =
        crate::infrastructure::auth::user_id_from_claims(&claims).map_err(AppError::BadRequest)?;

    // Find user by iterating — simple approach using email lookup from token sub (user id)
    // We query by ID directly
    let row = sqlx::query("SELECT id, name, email, created_at FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_optional(&state.db_pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    match row {
        Some(r) => {
            use sqlx::Row;
            let user = UserPublic {
                id: r
                    .try_get::<uuid::Uuid, _>("id")
                    .unwrap_or_default()
                    .to_string(),
                name: r.try_get("name").unwrap_or_default(),
                email: r.try_get("email").unwrap_or_default(),
            };
            let value = serde_json::to_value(user)?;
            Ok(Json(value))
        }
        None => Err(AppError::BadRequest("User not found".to_string())),
    }
}

pub async fn analyze_website(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<AnalysisRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let use_case = AnalyzeWebsite {
        scanner: &*state.website_scanner,
    };

    let analysis = use_case
        .execute(payload.url)
        .await
        .map_err(AppError::Internal)?;
    let value = serde_json::to_value(analysis)?;
    Ok(Json(value))
}

use axum::extract::Query;
use axum::response::sse::{Event, Sse};
use serde::Deserialize;
use std::convert::Infallible;
use tokio_stream::StreamExt;

use crate::domain::repositories::{ServerInvestigator, WebsiteScanner};

#[derive(Deserialize)]
pub struct StreamQuery {
    url: String,
}

pub async fn analyze_website_stream(
    State(state): State<Arc<AppState>>,
    Query(query): Query<StreamQuery>,
) -> Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>> {
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
    let scanner = state.website_scanner.clone();
    let url = query.url.clone();

    tokio::spawn(async move {
        if let Err(e) = scanner.scan_stream(&url, tx).await {
            tracing::error!("Scan stream failed for {}: {}", url, e);
        }
    });

    let stream = tokio_stream::wrappers::UnboundedReceiverStream::new(rx).map(|evt| {
        let json = match serde_json::to_string(&evt) {
            Ok(j) => j,
            Err(e) => {
                tracing::warn!("SSE serialization error: {}", e);
                serde_json::json!({"error": e.to_string()}).to_string()
            }
        };
        Ok(Event::default().data(json).event(evt.event_type.clone()))
    });

    Sse::new(stream).keep_alive(axum::response::sse::KeepAlive::new())
}

pub async fn investigate_server(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<AnalysisRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let use_case = InvestigateServer {
        investigator: &*state.server_investigator,
    };

    let info = use_case
        .execute(payload.url)
        .await
        .map_err(AppError::Internal)?;
    let value = serde_json::to_value(info)?;
    Ok(Json(value))
}

pub async fn investigate_server_stream(
    State(state): State<Arc<AppState>>,
    Query(query): Query<StreamQuery>,
) -> Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>> {
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
    let investigator = state.server_investigator.clone();
    let url = query.url.clone();

    tokio::spawn(async move {
        if let Err(e) = investigator.investigate_stream(&url, tx).await {
            tracing::error!("Investigation stream failed for {}: {}", url, e);
        }
    });

    let stream = tokio_stream::wrappers::UnboundedReceiverStream::new(rx).map(|evt| {
        let json = match serde_json::to_string(&evt) {
            Ok(j) => j,
            Err(e) => {
                tracing::warn!("SSE serialization error: {}", e);
                serde_json::json!({"error": e.to_string()}).to_string()
            }
        };
        Ok(Event::default().data(json).event(evt.event_type.clone()))
    });

    Sse::new(stream).keep_alive(axum::response::sse::KeepAlive::new())
}

pub async fn discover_apis(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<AnalysisRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let use_case = DiscoverApis {
        discoverer: &*state.api_discoverer,
    };

    let res = use_case
        .execute(payload.url)
        .await
        .map_err(AppError::Internal)?;
    let value = serde_json::to_value(res)?;
    Ok(Json(value))
}

pub async fn collect_services(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<AnalysisRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let use_case = CollectExternalServices {
        collector: &*state.service_collector,
    };

    let res = use_case
        .execute(payload.url)
        .await
        .map_err(AppError::Internal)?;
    let value = serde_json::to_value(res)?;
    Ok(Json(value))
}

pub async fn audit_security(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<AnalysisRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let use_case = AuditSecurity {
        auditor: &*state.security_auditor,
    };

    let res = use_case
        .execute(payload.url)
        .await
        .map_err(AppError::Internal)?;
    let value = serde_json::to_value(res)?;
    Ok(Json(value))
}

pub async fn map_forms(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<AnalysisRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let use_case = MapForms {
        mapper: &*state.form_mapper,
    };

    let res = use_case
        .execute(payload.url)
        .await
        .map_err(AppError::Internal)?;
    let value = serde_json::to_value(res)?;
    Ok(Json(value))
}

pub async fn generate_master_report(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<AnalysisRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let use_case = GenerateMasterReport {
        scanner: &*state.website_scanner,
        investigator: &*state.server_investigator,
        discoverer: &*state.api_discoverer,
        collector: &*state.service_collector,
        auditor: &*state.security_auditor,
        mapper: &*state.form_mapper,
        dynamic_rule_engine: Some(&*state.dynamic_rule_engine),
        db_pool: state.db_pool.clone(),
    };

    let res = use_case
        .execute(payload.url)
        .await
        .map_err(AppError::Internal)?;

    // Save scan to database for historical tracking
    if let Err(e) = state.delta_engine.save_scan(&res).await {
        tracing::error!("Failed to save scan to delta engine: {}", e);
    }

    let value = serde_json::to_value(res)?;
    Ok(Json(value))
}

pub async fn proxy_request(
    Json(payload): Json<ProxyRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    let client = reqwest::Client::new();
    let req = match payload.method.to_uppercase().as_str() {
        "POST" => client.post(&payload.url),
        _ => client.get(&payload.url),
    };

    let req = if let Some(body) = payload.body {
        if payload.method.to_uppercase() == "POST" {
            req.header("Content-Type", "application/json").body(body)
        } else {
            req
        }
    } else {
        req
    };

    match req.send().await {
        Ok(res) => {
            let status = res.status();
            let text = res.text().await.unwrap_or_default();
            let parsed_body: serde_json::Value =
                serde_json::from_str(&text).unwrap_or(serde_json::Value::String(text));
            Ok((status, Json(parsed_body)))
        }
        Err(e) => Err(AppError::BadGateway(e.to_string())),
    }
}

pub async fn verify_port(
    Json(payload): Json<VerifyPortRequest>,
) -> (StatusCode, Json<VerifyPortResponse>) {
    use std::time::Instant;
    use tokio::net::TcpStream;
    use tokio::time::{timeout, Duration};

    let target = format!("{}:{}", payload.host, payload.port);
    let start = Instant::now();

    match timeout(Duration::from_secs(3), TcpStream::connect(&target)).await {
        Ok(Ok(mut stream)) => {
            let latency_ms = start.elapsed().as_millis() as u64;

            use tokio::io::AsyncReadExt;
            let mut buf = [0; 128];
            let banner = match timeout(Duration::from_millis(500), stream.read(&mut buf)).await {
                Ok(Ok(n)) if n > 0 => Some(String::from_utf8_lossy(&buf[..n]).to_string()),
                _ => None,
            };

            (
                StatusCode::OK,
                Json(VerifyPortResponse {
                    is_active: true,
                    latency_ms: Some(latency_ms),
                    banner,
                }),
            )
        }
        _ => (
            StatusCode::OK,
            Json(VerifyPortResponse {
                is_active: false,
                latency_ms: None,
                banner: None,
            }),
        ),
    }
}

pub async fn submit_feedback(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<FeedbackRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let engine = crate::infrastructure::auditor::learning_feedback::LearningFeedbackEngine::new(
        state.db_pool.clone(),
    );
    engine
        .record_feedback(payload.signature.clone(), payload.action.clone())
        .await;

    Ok(Json(serde_json::json!({
        "status": "success",
        "message": format!("Recorded feedback {:?} for signature: {}", payload.action, payload.signature)
    })))
}

#[derive(serde::Deserialize)]
pub struct RuleFeedbackPayload {
    pub rule_id: String,
    pub target_url: String,
    pub action: crate::infrastructure::rule_engine::FeedbackAction,
}

pub async fn submit_rule_feedback(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RuleFeedbackPayload>,
) -> Result<Json<serde_json::Value>, AppError> {
    state.dynamic_rule_engine.feedback_engine.record_feedback(
        payload.rule_id.clone(),
        payload.target_url,
        "anonymous_user".to_string(),
        payload.action.clone(),
    );

    Ok(Json(serde_json::json!({
        "status": "success",
        "message": format!("Recorded feedback {:?} for rule: {}", payload.action, payload.rule_id)
    })))
}

#[derive(serde::Serialize)]
pub struct RuleEngineStatus {
    pub total_rules: usize,
    pub disabled_packs: Vec<String>,
    pub disabled_rules: Vec<String>,
    pub active_rules: Vec<serde_json::Value>,
    pub feedback_stats: std::collections::HashMap<String, serde_json::Value>,
    pub load_errors: Vec<String>,
    pub validation_errors: Vec<String>,
}

pub async fn get_rule_engine_status(
    State(state): State<Arc<AppState>>,
) -> Result<Json<RuleEngineStatus>, AppError> {
    let engine = &state.dynamic_rule_engine;
    let (disabled_packs, disabled_rules) = engine.get_governance_snapshot();

    let mut feedback_stats = std::collections::HashMap::new();
    let rules: Vec<serde_json::Value> = engine
        .rules()
        .iter()
        .map(|r| {
            let stats = engine.feedback_engine.get_rule_stats(&r.id);
            if stats.total_feedback > 0 {
                feedback_stats.insert(
                    r.id.clone(),
                    serde_json::json!({
                        "total_feedback": stats.total_feedback,
                        "confirmed": stats.confirmed,
                        "false_positives": stats.false_positives,
                        "ignored": stats.ignored,
                        "reputation_score": stats.reputation_score
                    }),
                );
            }
            serde_json::json!({
                "id": r.id,
                "name": r.name,
                "category": r.category,
                "pack": r.pack,
                "source": r.source,
                "version": r.version,
                "default_severity": r.default_severity,
                "default_confidence": r.default_confidence,
                "is_active": engine.is_rule_active(r)
            })
        })
        .collect();

    Ok(Json(RuleEngineStatus {
        total_rules: engine.rule_count(),
        disabled_packs: disabled_packs.into_iter().collect(),
        disabled_rules: disabled_rules.into_iter().collect(),
        active_rules: rules,
        feedback_stats,
        load_errors: engine.load_errors().to_vec(),
        validation_errors: engine.validation_errors().to_vec(),
    }))
}

pub async fn get_feedback_stats(
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, AppError> {
    let engine = &state.dynamic_rule_engine;
    let history = engine.feedback_engine.get_feedback_history();

    let mut rule_stats: std::collections::HashMap<String, serde_json::Value> =
        std::collections::HashMap::new();

    for rule in engine.rules() {
        let stats = engine.feedback_engine.get_rule_stats(&rule.id);
        if stats.total_feedback > 0 {
            rule_stats.insert(
                rule.id.clone(),
                serde_json::json!({
                    "rule_name": rule.name,
                    "total_feedback": stats.total_feedback,
                    "confirmed": stats.confirmed,
                    "false_positives": stats.false_positives,
                    "ignored": stats.ignored,
                    "reputation_score": stats.reputation_score
                }),
            );
        }
    }

    Ok(Json(serde_json::json!({
        "total_feedback_entries": history.len(),
        "rule_stats": rule_stats,
        "recent_feedback": history.iter().rev().take(20).map(|e| serde_json::json!({
            "rule_id": e.rule_id,
            "action": e.action,
            "target_url": e.target_url,
            "timestamp": e.timestamp
        })).collect::<Vec<_>>()
    })))
}

/// Export scan results to Burp Suite XML format.
pub async fn export_to_burp(
    Json(payload): Json<crate::domain::entities::MasterReport>,
) -> Result<Json<serde_json::Value>, AppError> {
    let burp_export = crate::infrastructure::export::burp::BurpExport::from_master_report(&payload);
    let xml = burp_export.to_xml();
    let filename = format!(
        "{}_limma_burp_export.xml",
        url::Url::parse(&payload.url)
            .map(|u| u.host_str().unwrap_or("target").to_string())
            .unwrap_or_else(|_| "target".to_string())
    );

    Ok(Json(serde_json::json!({
        "xml": xml,
        "filename": filename,
        "item_count": burp_export.items.len()
    })))
}

/// Export scan results to Nuclei YAML template format.
pub async fn export_to_nuclei(
    Json(payload): Json<crate::domain::entities::MasterReport>,
) -> Result<Json<serde_json::Value>, AppError> {
    let nuclei_export =
        crate::infrastructure::export::nuclei::NucleiExport::from_master_report(&payload);
    let yaml = nuclei_export.to_yaml();

    Ok(Json(serde_json::json!({
        "yaml": yaml,
        "template_count": nuclei_export.templates.len()
    })))
}

// ── Burp Suite Bridge Handlers ──

/// Handshake endpoint: Burp plugin registers itself and gets a session ID.
pub async fn burp_handshake(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<crate::domain::entities::BurpHandshakeRequest>,
) -> Result<Json<crate::domain::entities::BurpHandshakeResponse>, AppError> {
    let response = state.burp_bridge.create_session(&payload).await;
    Ok(Json(response))
}

/// Import HTTP traffic captured by Burp Suite into LIMMA.
pub async fn burp_import_traffic(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<crate::domain::entities::BurpImportTrafficRequest>,
) -> Result<Json<crate::domain::entities::BurpImportTrafficResponse>, AppError> {
    let response = state
        .burp_bridge
        .import_traffic(
            &payload.session_id,
            payload.items,
            &state.dynamic_rule_engine,
        )
        .await
        .map_err(AppError::BadRequest)?;
    Ok(Json(response))
}

/// Export LIMMA findings in Burp-native format for a given session.
pub async fn burp_get_findings(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(session_id): axum::extract::Path<String>,
) -> Result<Json<crate::domain::entities::BurpFindingsResponse>, AppError> {
    // Verify session exists
    let session = state
        .burp_bridge
        .get_session(&session_id)
        .ok_or_else(|| AppError::BadRequest(format!("Session not found: {}", session_id)))?;

    // Phase 2: Retrieve findings generated directly from the imported traffic
    let findings = state.burp_bridge.get_session_findings(&session_id);
    let total = findings.len();

    Ok(Json(crate::domain::entities::BurpFindingsResponse {
        session_id,
        target: session.target_url,
        findings,
        total_count: total,
        generated_at: chrono::Utc::now(),
    }))
}

/// SSE endpoint to stream real-time events to the Burp plugin
pub async fn burp_stream_events(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(session_id): axum::extract::Path<String>,
) -> Result<
    axum::response::Sse<
        impl futures::stream::Stream<
            Item = Result<axum::response::sse::Event, std::convert::Infallible>,
        >,
    >,
    AppError,
> {
    use axum::response::sse::Event;
    use tokio_stream::StreamExt;

    // Verify session exists and get a subscriber receiver
    let rx = state
        .burp_bridge
        .subscribe_to_events(&session_id)
        .ok_or_else(|| AppError::BadRequest("Session not found or inactive".to_string()))?;

    let stream = tokio_stream::wrappers::BroadcastStream::new(rx)
        .filter_map(|msg_result| {
            // broadcast stream can return RecvError::Lagged, we just ignore lagged messages
            msg_result.ok()
        })
        .map(|event| {
            let json_str = serde_json::to_string(&event).unwrap_or_else(|_| "{}".to_string());
            Ok(Event::default()
                .event("message") // Standard SSE event type
                .data(json_str))
        });

    Ok(axum::response::Sse::new(stream).keep_alive(axum::response::sse::KeepAlive::new()))
}

/// List all active Burp Bridge sessions.
pub async fn burp_list_sessions(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<crate::domain::entities::BurpBridgeSession>>, AppError> {
    let sessions = state.burp_bridge.get_all_sessions();
    Ok(Json(sessions))
}

// ── Delta Engine Handlers ──

#[derive(serde::Deserialize)]
pub struct HistoryQuery {
    target_url: String,
}

pub async fn get_history_trends(
    State(state): State<Arc<AppState>>,
    Query(query): Query<HistoryQuery>,
) -> Result<Json<Vec<crate::infrastructure::delta_engine::TrendPoint>>, AppError> {
    let trends = state
        .delta_engine
        .get_trends(&query.target_url)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(Json(trends))
}

#[derive(serde::Deserialize)]
pub struct DeltaQuery {
    target_url: String,
    current_scan_id: uuid::Uuid,
    previous_scan_id: uuid::Uuid,
}

pub async fn get_history_delta(
    State(state): State<Arc<AppState>>,
    Query(query): Query<DeltaQuery>,
) -> Result<Json<crate::infrastructure::delta_engine::DeltaResult>, AppError> {
    let delta = state
        .delta_engine
        .calculate_delta(
            &query.target_url,
            query.current_scan_id,
            query.previous_scan_id,
        )
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(Json(delta))
}

/// Fetch a single scan result by its UUID.
pub async fn get_scan_by_id(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(scan_id): axum::extract::Path<uuid::Uuid>,
) -> Result<Json<crate::infrastructure::delta_engine::ScanDetail>, AppError> {
    let detail = state
        .delta_engine
        .get_scan_by_id(scan_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    match detail {
        Some(d) => Ok(Json(d)),
        None => Err(AppError::BadRequest(format!("Scan not found: {}", scan_id))),
    }
}

/// List all scans, optionally filtered by target_url.
#[derive(serde::Deserialize)]
pub struct ListScansQuery {
    target_url: Option<String>,
    #[serde(default = "default_limit")]
    limit: i64,
}

fn default_limit() -> i64 {
    50
}

pub async fn list_scans(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListScansQuery>,
) -> Result<Json<Vec<crate::infrastructure::delta_engine::TrendPoint>>, AppError> {
    let scans = state
        .delta_engine
        .list_scans(query.target_url.as_deref(), query.limit)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(Json(scans))
}
