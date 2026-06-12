use crate::api::models::{
    AnalysisRequest, FeedbackRequest, ProxyRequest, VerifyPortRequest, VerifyPortResponse,
};
use crate::application::use_cases::{
    AnalyzeWebsite, AuditSecurity, CollectExternalServices, DiscoverApis, GenerateMasterReport,
    InvestigateServer, MapForms,
};
use crate::error::AppError;
use crate::infrastructure::auditor::HttpSecurityAuditor;
use crate::infrastructure::blind_detection::HttpBlindDetectionEngine;
use crate::infrastructure::collector::HttpServiceCollector;
use crate::infrastructure::discoverer::HttpApiDiscoverer;
use crate::infrastructure::exploitation::poc_generator::CompositePocGenerator;
use crate::infrastructure::subdomain_discovery::HttpSubdomainDiscoverer;

use crate::infrastructure::investigator::HttpInvestigator;
use crate::infrastructure::mapper::HttpFormMapper;

use crate::infrastructure::repositories::blind_finding_repo::PgBlindFindingRepository;
use crate::infrastructure::repositories::exploit_result_repo::PgExploitResultRepository;
use crate::infrastructure::repositories::poc_repo::PgPocRepository;
use crate::infrastructure::rule_engine::SharedDynamicRuleEngine;
use crate::infrastructure::safety::SafetyFrameworkImpl;
use crate::infrastructure::scanner::HttpWebsiteScanner;
use axum::{extract::State, http::StatusCode, Json};
use std::{
    net::{IpAddr, SocketAddr},
    sync::Arc,
};

pub struct AppState {
    pub website_scanner: Arc<HttpWebsiteScanner>,
    pub server_investigator: Arc<HttpInvestigator>,
    pub subdomain_discoverer: Arc<HttpSubdomainDiscoverer>,
    pub certificate_discoverer:
        Arc<crate::infrastructure::subdomain_discovery::certificate::CertificateDiscoverer>,
    pub api_discoverer: Arc<HttpApiDiscoverer>,
    pub service_collector: Arc<HttpServiceCollector>,
    pub security_auditor: Arc<HttpSecurityAuditor>,
    pub form_mapper: Arc<HttpFormMapper>,
    pub dynamic_rule_engine: SharedDynamicRuleEngine,
    pub delta_engine: Arc<crate::infrastructure::delta_engine::DeltaEngine>,
    // Faz F: Blind Detection & Exploitation
    pub blind_detection_engine: Arc<HttpBlindDetectionEngine>,
    pub poc_generator: Arc<CompositePocGenerator>,
    pub safety_framework: Arc<SafetyFrameworkImpl>,
    pub exploit_bridge: Arc<crate::infrastructure::exploitation::exploit_bridge::ExploitBridge>,
    pub blind_finding_repo: Arc<PgBlindFindingRepository>,
    pub poc_repo: Arc<PgPocRepository>,
    pub exploit_result_repo: Arc<PgExploitResultRepository>,
    pub settings_repo: Arc<dyn crate::domain::repositories::SettingsRepository>,

    // Faz 1: Active Vulnerability Detection
    pub active_scan_repo:
        Arc<crate::infrastructure::repositories::active_scan_repo::PgActiveScanRepository>,
    pub active_finding_repo:
        Arc<crate::infrastructure::repositories::active_finding_repo::PgActiveFindingRepository>,
    pub active_detectors:
        Arc<Vec<Box<dyn crate::infrastructure::active_detection::detectors::VulnDetector>>>,
    pub payload_db: Arc<crate::infrastructure::active_detection::payloads::PayloadDatabase>,
    pub db_pool: sqlx::PgPool,
    pub scan_controller: Arc<crate::infrastructure::scan_controller::ScanController>,
}

struct ResolvedExternalUrl {
    host: String,
    addrs: Vec<SocketAddr>,
}

/// Helper: resolve a SettingsProfile from the repo, falling back to default.
async fn resolve_profile(
    state: &AppState,
    profile_id: Option<&str>,
) -> Result<crate::domain::entities::SettingsProfile, AppError> {
    if let Some(key) = profile_id {
        return state
            .settings_repo
            .get_profile(key)
            .await
            .map_err(AppError::Internal)?
            .ok_or_else(|| AppError::BadRequest(format!("Unknown profile_id: {}", key)));
    }

    Ok(state
        .settings_repo
        .get_profile("default")
        .await
        .map_err(AppError::Internal)?
        .unwrap_or_else(crate::domain::entities::SettingsProfile::default))
}

fn allow_private_targets() -> bool {
    std::env::var("LIMMA_ALLOW_PRIVATE_TARGETS")
        .map(|value| {
            matches!(
                value.to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            )
        })
        .unwrap_or(false)
}

fn validate_external_url(raw_url: &str) -> Result<(), AppError> {
    let url = url::Url::parse(raw_url)
        .map_err(|e| AppError::BadRequest(format!("Invalid URL: {}", e)))?;

    if !matches!(url.scheme(), "http" | "https") {
        return Err(AppError::BadRequest(
            "Only http and https URLs are allowed".to_string(),
        ));
    }

    let host = url
        .host_str()
        .ok_or_else(|| AppError::BadRequest("URL must include a host".to_string()))?;

    validate_external_host(host)
}

async fn resolve_external_url(raw_url: &str) -> Result<ResolvedExternalUrl, AppError> {
    validate_external_url(raw_url)?;

    let url = url::Url::parse(raw_url)
        .map_err(|e| AppError::BadRequest(format!("Invalid URL: {}", e)))?;
    let host = url
        .host_str()
        .ok_or_else(|| AppError::BadRequest("URL must include a host".to_string()))?;

    let addrs =
        resolve_external_socket_addrs(host, url.port_or_known_default().unwrap_or(443)).await?;
    Ok(ResolvedExternalUrl {
        host: host.to_ascii_lowercase(),
        addrs,
    })
}

async fn resolve_external_socket_addrs(host: &str, port: u16) -> Result<Vec<SocketAddr>, AppError> {
    validate_external_host(host)?;

    let normalized = host
        .trim()
        .trim_start_matches('[')
        .trim_end_matches(']')
        .trim_end_matches('.')
        .to_ascii_lowercase();

    if let Ok(ip) = normalized.parse::<IpAddr>() {
        return Ok(vec![SocketAddr::new(ip, port)]);
    }

    let addrs: Vec<SocketAddr> = tokio::net::lookup_host((normalized.as_str(), port))
        .await
        .map_err(|e| AppError::BadRequest(format!("Could not resolve host {}: {}", host, e)))?
        .collect();

    if addrs.is_empty() {
        return Err(AppError::BadRequest(format!(
            "Host did not resolve to any address: {}",
            host
        )));
    }

    if !allow_private_targets() {
        if let Some(blocked_addr) = addrs.iter().find(|addr| is_blocked_ip(addr.ip())) {
            return Err(AppError::SafetyViolation(format!(
                "Host resolves to private/internal IP: {} -> {}",
                host,
                blocked_addr.ip()
            )));
        }
    }

    Ok(addrs)
}

fn validate_external_host(host: &str) -> Result<(), AppError> {
    if allow_private_targets() {
        return Ok(());
    }

    let normalized = host
        .trim()
        .trim_start_matches('[')
        .trim_end_matches(']')
        .trim_end_matches('.')
        .to_ascii_lowercase();

    if normalized.is_empty() {
        return Err(AppError::BadRequest("Host must not be empty".to_string()));
    }

    let blocked_names = [
        "localhost",
        "ip6-localhost",
        "ip6-loopback",
        "metadata",
        "metadata.google.internal",
    ];
    if blocked_names.contains(&normalized.as_str()) || normalized.ends_with(".localhost") {
        return Err(AppError::SafetyViolation(format!(
            "Private/internal host is not allowed: {}",
            host
        )));
    }

    if let Ok(ip) = normalized.parse::<IpAddr>() {
        if is_blocked_ip(ip) {
            return Err(AppError::SafetyViolation(format!(
                "Private/internal IP is not allowed: {}",
                host
            )));
        }
    }

    Ok(())
}

fn is_blocked_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(addr) => {
            let octets = addr.octets();
            addr.is_private()
                || addr.is_loopback()
                || addr.is_link_local()
                || addr.is_unspecified()
                || addr.is_multicast()
                || addr.is_broadcast()
                || octets[0] == 0
                || (octets[0] == 100 && (64..=127).contains(&octets[1]))
        }
        IpAddr::V6(addr) => {
            if let Some(v4) = addr.to_ipv4_mapped() {
                return is_blocked_ip(IpAddr::V4(v4));
            }

            addr.is_loopback()
                || addr.is_unspecified()
                || addr.is_unique_local()
                || addr.is_unicast_link_local()
                || addr.is_multicast()
        }
    }
}

pub async fn analyze_website(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<AnalysisRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let profile = resolve_profile(&state, payload.profile_id.as_deref()).await?;
    let use_case = AnalyzeWebsite {
        scanner: &*state.website_scanner,
    };

    let analysis = use_case
        .execute(payload.url, &profile)
        .await
        .map_err(AppError::Internal)?;
    let value = serde_json::to_value(analysis)?;
    Ok(Json(value))
}

pub async fn get_health(
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, AppError> {
    // Check DB
    let db_ok = sqlx::query("SELECT 1")
        .execute(&state.db_pool)
        .await
        .is_ok();

    // Check Docker
    let docker_ok = match bollard::Docker::connect_with_local_defaults() {
        Ok(docker) => docker.ping().await.is_ok(),
        Err(_) => false,
    };

    Ok(Json(serde_json::json!({
        "status": if db_ok && docker_ok { "ok" } else { "degraded" },
        "database": if db_ok { "connected" } else { "error" },
        "docker_daemon": if docker_ok { "running" } else { "unavailable" },
        "timestamp": chrono::Utc::now().to_rfc3339()
    })))
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
    let settings_repo = state.settings_repo.clone();
    let url = query.url.clone();

    tokio::spawn(async move {
        let profile = settings_repo
            .get_profile("default")
            .await
            .ok()
            .flatten()
            .unwrap_or_else(crate::domain::entities::SettingsProfile::default);
        let config = crate::domain::engine_config::EngineConfig::from_profile(&profile);
        if let Err(e) = scanner.scan_stream(&url, &config, tx).await {
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
    let profile = resolve_profile(&state, payload.profile_id.as_deref()).await?;
    let use_case = InvestigateServer {
        investigator: &*state.server_investigator,
    };

    let info = use_case
        .execute(payload.url, &profile)
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
    let settings_repo = state.settings_repo.clone();
    let url = query.url.clone();

    tokio::spawn(async move {
        let profile = settings_repo
            .get_profile("default")
            .await
            .ok()
            .flatten()
            .unwrap_or_else(crate::domain::entities::SettingsProfile::default);
        let config = crate::domain::engine_config::EngineConfig::from_profile(&profile);
        if let Err(e) = investigator.investigate_stream(&url, &config, tx).await {
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
    let profile = resolve_profile(&state, payload.profile_id.as_deref()).await?;
    let use_case = DiscoverApis {
        discoverer: &*state.api_discoverer,
    };

    let res = use_case
        .execute(payload.url, &profile)
        .await
        .map_err(AppError::Internal)?;
    let value = serde_json::to_value(res)?;
    Ok(Json(value))
}

pub async fn discover_subdomains(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<crate::api::models::SubdomainDiscoveryRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let profile = resolve_profile(&state, payload.profile_id.as_deref()).await?;
    let use_case = crate::application::use_cases::subdomain_discovery::DiscoverSubdomains {
        discoverer: &*state.subdomain_discoverer,
    };

    let res = use_case
        .execute(payload.domain, &profile)
        .await
        .map_err(AppError::Internal)?;
    let value = serde_json::to_value(res)?;
    Ok(Json(value))
}

pub async fn discover_certificates(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<crate::api::models::DiscoverCertificatesRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let profile = resolve_profile(&state, payload.profile_id.as_deref()).await?;
    let config = crate::domain::engine_config::EngineConfig::from_profile(&profile);

    let res = state
        .certificate_discoverer
        .discover(payload, &config)
        .await;
    let value = serde_json::to_value(res)?;
    Ok(Json(value))
}

pub async fn collect_services(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<AnalysisRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let profile = resolve_profile(&state, payload.profile_id.as_deref()).await?;
    let use_case = CollectExternalServices {
        collector: &*state.service_collector,
    };

    let res = use_case
        .execute(payload.url, &profile)
        .await
        .map_err(AppError::Internal)?;
    let value = serde_json::to_value(res)?;
    Ok(Json(value))
}

pub async fn audit_security(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<AnalysisRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let profile = resolve_profile(&state, payload.profile_id.as_deref()).await?;
    let use_case = AuditSecurity {
        auditor: &*state.security_auditor,
    };

    let res = use_case
        .execute(payload.url, &profile)
        .await
        .map_err(AppError::Internal)?;
    let value = serde_json::to_value(res)?;
    Ok(Json(value))
}

pub async fn map_forms(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<AnalysisRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let profile = resolve_profile(&state, payload.profile_id.as_deref()).await?;
    let use_case = MapForms {
        mapper: &*state.form_mapper,
    };

    let res = use_case
        .execute(payload.url, &profile)
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
        settings_repo: &*state.settings_repo,
    };

    let res = use_case
        .execute(payload.url, payload.profile_id)
        .await
        .map_err(AppError::Internal)?;

    // Save scan to database for historical tracking only if requested
    if payload.save_to_history.unwrap_or(false) {
        if let Err(e) = state.delta_engine.save_scan(&res).await {
            tracing::error!("Failed to save scan to delta engine: {}", e);
        }
    }

    let value = serde_json::to_value(res)?;
    Ok(Json(value))
}

pub async fn proxy_request(
    Json(payload): Json<ProxyRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    let resolved = resolve_external_url(&payload.url).await?;

    let method = payload.method.to_uppercase();
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .redirect(reqwest::redirect::Policy::none())
        .resolve_to_addrs(&resolved.host, &resolved.addrs)
        .build()
        .map_err(|e| AppError::Internal(format!("Failed to build proxy client: {}", e)))?;

    let req = match method.as_str() {
        "GET" => client.get(&payload.url),
        "POST" => client.post(&payload.url),
        _ => {
            return Err(AppError::BadRequest(
                "Proxy only supports GET and POST".to_string(),
            ))
        }
    };

    let req = if let Some(body) = payload.body {
        if method == "POST" {
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
) -> Result<(StatusCode, Json<VerifyPortResponse>), AppError> {
    use std::time::Instant;
    use tokio::net::TcpStream;
    use tokio::time::{timeout, Duration};

    let target_addrs = resolve_external_socket_addrs(&payload.host, payload.port).await?;
    let start = Instant::now();

    let response = match timeout(
        Duration::from_secs(3),
        TcpStream::connect(target_addrs.as_slice()),
    )
    .await
    {
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
    };

    Ok(response)
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
    let rules_snap = engine.rules_snapshot();
    let rules: Vec<serde_json::Value> = rules_snap
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
        disabled_packs: disabled_packs.into_iter().collect::<Vec<String>>(),
        disabled_rules: disabled_rules.into_iter().collect::<Vec<String>>(),
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

    let rules_snap = engine.rules_snapshot();
    for rule in &rules_snap {
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

// ── Custom Rule CRUD Handlers ──

#[derive(serde::Deserialize)]
pub struct CreateRulePayload {
    pub id: String,
    pub name: String,
    pub yaml_content: String,
}

/// POST /api/rules — Create a custom rule from YAML content
pub async fn create_custom_rule(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateRulePayload>,
) -> Result<Json<serde_json::Value>, AppError> {
    // 1. Validate the YAML content parses into a valid RuleDefinition
    let rule_def: crate::infrastructure::rule_engine::models::RuleDefinition =
        serde_yaml::from_str(&payload.yaml_content)
            .map_err(|e| AppError::BadRequest(format!("Invalid YAML rule: {}", e)))?;

    // 2. Validate via the validator pipeline
    let (valid, errors) =
        crate::infrastructure::rule_engine::validator::validate_rules(vec![rule_def.clone()]);
    if valid.is_empty() {
        return Err(AppError::BadRequest(format!(
            "Rule validation failed: {}",
            errors.join("; ")
        )));
    }

    // 3. Persist to database
    sqlx::query(
        r#"
        INSERT INTO custom_rules (id, name, yaml_content, created_at, updated_at)
        VALUES ($1, $2, $3, NOW(), NOW())
        ON CONFLICT (id) DO UPDATE SET
            name = EXCLUDED.name,
            yaml_content = EXCLUDED.yaml_content,
            updated_at = NOW()
        "#,
    )
    .bind(&payload.id)
    .bind(&payload.name)
    .bind(&payload.yaml_content)
    .execute(&state.db_pool)
    .await
    .map_err(|e| AppError::Internal(format!("DB error: {}", e)))?;

    // 4. Hot-load into engine
    let rule = valid.into_iter().next().unwrap();
    state.dynamic_rule_engine.add_rule(rule);

    Ok(Json(serde_json::json!({
        "status": "success",
        "message": format!("Rule '{}' created and loaded", payload.id)
    })))
}

/// DELETE /api/rules/:id — Delete a custom rule
pub async fn delete_custom_rule(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(rule_id): axum::extract::Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    // 1. Delete from database
    let result = sqlx::query("DELETE FROM custom_rules WHERE id = $1")
        .bind(&rule_id)
        .execute(&state.db_pool)
        .await
        .map_err(|e| AppError::Internal(format!("DB error: {}", e)))?;

    // 2. Remove from engine
    state.dynamic_rule_engine.remove_rule(&rule_id);

    if result.rows_affected() == 0 {
        return Ok(Json(serde_json::json!({
            "status": "success",
            "message": format!("Rule '{}' removed from engine (was not a custom rule in DB)", rule_id)
        })));
    }

    Ok(Json(serde_json::json!({
        "status": "success",
        "message": format!("Rule '{}' deleted", rule_id)
    })))
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

/// Delete a single historical scan by its UUID.
pub async fn delete_history_scan(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(scan_id): axum::extract::Path<uuid::Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    state
        .delta_engine
        .delete_scan(scan_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(Json(
        serde_json::json!({ "success": true, "message": "Scan deleted" }),
    ))
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

// ── Faz F: Blind Detection & Exploitation Handlers ──

/// POST /api/blind-scan — Execute blind vulnerability detection
pub async fn blind_scan(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<crate::api::models::BlindScanApiRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let request = crate::application::use_cases::blind_scan::BlindScanRequest {
        scan_id: payload.scan_id.unwrap_or_else(uuid::Uuid::new_v4),
        target_url: payload.target_url,
        target_id: payload.target_id.unwrap_or_else(uuid::Uuid::new_v4),
        detection_types: payload.detection_types,
        max_duration_seconds: payload.max_duration_seconds.unwrap_or(120),
    };

    let use_case = crate::application::use_cases::blind_scan::PerformBlindScan {
        finding_repo: &*state.blind_finding_repo,
        safety_framework: &*state.safety_framework,
        detection_engine: &*state.blind_detection_engine,
    };

    let result = use_case
        .execute(request)
        .await
        .map_err(AppError::Internal)?;
    let value = serde_json::to_value(result)?;
    Ok(Json(value))
}

// ── Phase 4: System Settings Profile Handlers ──

pub async fn get_settings_profiles(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<crate::domain::entities::SettingsProfile>>, AppError> {
    let profiles = state
        .settings_repo
        .get_all_profiles()
        .await
        .map_err(AppError::Internal)?;
    Ok(Json(profiles))
}

pub async fn update_settings_profile(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(profile_id): axum::extract::Path<String>,
    Json(mut payload): Json<crate::domain::entities::SettingsProfile>,
) -> Result<Json<serde_json::Value>, AppError> {
    // Ensure the payload ID matches the path ID
    payload.id = profile_id.clone();

    state
        .settings_repo
        .save_profile(payload)
        .await
        .map_err(AppError::Internal)?;

    Ok(Json(serde_json::json!({
        "status": "success",
        "message": format!("Profile {} updated successfully", profile_id)
    })))
}

/// POST /api/poc/generate — Generate a PoC for a blind finding
pub async fn generate_poc(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<crate::application::use_cases::generate_poc::GeneratePocRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let use_case = crate::application::use_cases::generate_poc::GeneratePoc {
        finding_repo: &*state.blind_finding_repo,
        poc_repo: &*state.poc_repo,
        generator: &*state.poc_generator,
    };

    let poc = use_case
        .execute(payload)
        .await
        .map_err(AppError::Internal)?;
    let value = serde_json::to_value(poc)?;
    Ok(Json(value))
}

/// POST /api/exploit/verify — Verify a PoC in sandbox
pub async fn verify_exploit(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<crate::application::use_cases::verify_exploit::VerifyExploitRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let use_case = crate::application::use_cases::verify_exploit::VerifyExploit {
        poc_repo: &*state.poc_repo,
        exploit_result_repo: &*state.exploit_result_repo,
        exploit_bridge: &state.exploit_bridge,
    };

    let payload_clone = payload.clone();
    let result = use_case
        .execute(payload)
        .await
        .map_err(AppError::Internal)?;

    if matches!(
        payload_clone.execution_level,
        crate::domain::entities::SafetyLevel::L3ActiveWithConsent
    ) {
        let _ = state
            .safety_framework
            .log_audit(
                "L3_EXPLOIT_EXECUTED",
                Some(&format!("PoC ID: {}", payload_clone.poc_id)),
                Some(&payload_clone.target_url),
                None,
            )
            .await;
    }

    let value = serde_json::to_value(result)?;
    Ok(Json(value))
}

/// GET /api/poc/:id — Download PoC code
pub async fn download_poc(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    use crate::domain::repositories::PocRepository;

    let poc = state
        .poc_repo
        .find_by_id(id)
        .await
        .map_err(AppError::Internal)?
        .ok_or_else(|| AppError::NotFound(format!("PoC not found: {}", id)))?;

    Ok((
        StatusCode::OK,
        Json(serde_json::json!({
            "id": poc.id,
            "code": poc.code,
            "language": format!("{:?}", poc.language),
            "poc_type": format!("{:?}", poc.poc_type),
            "safety_level": format!("{:?}", poc.safety_level),
            "verification_status": format!("{:?}", poc.verification_status),
        })),
    ))
}

// ── Faz 1: Active Vulnerability Detection Phase ──

pub async fn start_active_scan(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<crate::api::models::ActiveScanApiRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    use crate::application::use_cases::active_scan::PerformActiveScan;
    use crate::domain::active_vuln::{ActiveScanConfig, ActiveScanStatus};
    use crate::domain::repositories::ActiveScanRepository;

    // Resolve settings profile → EngineConfig for intensity-aware payload selection
    let profile = resolve_profile(&state, payload.profile_id.as_deref()).await?;
    let engine_config = crate::domain::engine_config::EngineConfig::from_profile(&profile);

    // Derive safe_mode and waf_bypass from EngineConfig
    let safe_mode = !engine_config.active_exploit_enabled;
    let enable_waf_bypass = engine_config.avoid_waf
        && matches!(
            engine_config.fuzzing_intensity,
            crate::domain::engine_config::FuzzingIntensity::High
                | crate::domain::engine_config::FuzzingIntensity::Aggressive
        );

    // Create PayloadSelector for the active scan
    let payload_selector = Arc::new(
        crate::infrastructure::active_detection::payload_selector::PayloadSelector::from_config(
            &engine_config,
            state.payload_db.clone(),
        ),
    );

    let use_case = PerformActiveScan {
        scan_repo: state.active_scan_repo.clone(),
        finding_repo: state.active_finding_repo.clone(),
        detectors: state.active_detectors.clone(),
        payload_selector,
    };

    let config = ActiveScanConfig {
        target_url: payload.target_url,
        vuln_types: payload.vuln_types,
        scan_mode: payload.scan_mode,
        enable_headless_browser: payload.enable_headless_browser,
        max_browser_tabs: payload.max_browser_tabs,
        bearer_token: payload.bearer_token,
        cookie: payload.cookie,
        custom_headers: payload.custom_headers,
        basic_auth_user: payload.basic_auth_user,
        basic_auth_pass: payload.basic_auth_pass,
        enable_json_fuzzing: payload.enable_json_fuzzing,
        enable_xss_verification: payload.enable_xss_verification,
        allow_destructive_methods: payload.allow_destructive_methods,
        l3_consent_accepted: payload.l3_consent_accepted,
        max_scan_duration_sec: payload.max_scan_duration_sec,
        max_requests_per_endpoint: payload.max_requests_per_endpoint,
        follow_redirects: payload
            .follow_redirects
            .unwrap_or(engine_config.follow_redirects),
        enable_waf_bypass,
        safe_mode,
        custom_parameters: payload.custom_parameters,
    };

    let scan_id = uuid::Uuid::new_v4();
    let handle = state.scan_controller.register_scan(scan_id).await;

    let scan_controller = state.scan_controller.clone();
    let active_scan_repo = state.active_scan_repo.clone();
    tokio::spawn(async move {
        if let Err(e) = use_case.execute(scan_id, config, handle).await {
            tracing::error!("Active scan {} failed: {}", scan_id, e);
            if let Err(update_err) = active_scan_repo
                .update_status(scan_id, ActiveScanStatus::Failed)
                .await
            {
                tracing::error!(
                    "Failed to mark active scan {} as failed: {}",
                    scan_id,
                    update_err
                );
            }
        }
        scan_controller.unregister_scan(&scan_id).await;
    });

    Ok(Json(serde_json::json!({
        "status": "success",
        "scan_id": scan_id
    })))
}

pub async fn get_active_scan(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    use crate::domain::repositories::ActiveScanRepository;

    let scan = state
        .active_scan_repo
        .find_by_id(id)
        .await
        .map_err(AppError::Internal)?
        .ok_or_else(|| AppError::NotFound(format!("Active scan not found: {}", id)))?;

    Ok(Json(serde_json::to_value(scan)?))
}

pub async fn list_active_findings(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(scan_id): axum::extract::Path<uuid::Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    use crate::domain::repositories::ActiveFindingRepository;

    let findings = state
        .active_finding_repo
        .find_by_scan_id(scan_id)
        .await
        .map_err(AppError::Internal)?;

    Ok(Json(serde_json::to_value(findings)?))
}

pub async fn get_active_finding(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    use crate::domain::repositories::ActiveFindingRepository;

    let finding = state
        .active_finding_repo
        .find_by_id(id)
        .await
        .map_err(AppError::Internal)?
        .ok_or_else(|| AppError::NotFound(format!("Finding not found: {}", id)))?;

    Ok(Json(serde_json::to_value(finding)?))
}

// ── Phase 2: Active Scan & Finding Management Endpoints ──

pub async fn list_active_scans(
    State(state): State<Arc<AppState>>,
    Query(query): Query<crate::domain::active_vuln::ScanQueryParams>,
) -> Result<Json<serde_json::Value>, AppError> {
    use crate::domain::repositories::ActiveScanRepository;

    let scans = state
        .active_scan_repo
        .list_scans(&query)
        .await
        .map_err(AppError::Internal)?;

    Ok(Json(serde_json::to_value(scans)?))
}

pub async fn pause_active_scan(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    state
        .scan_controller
        .pause_scan(&id)
        .await
        .map_err(AppError::BadRequest)?;
    Ok(Json(serde_json::json!({
        "status": "success",
        "message": format!("Scan {} paused", id)
    })))
}

pub async fn resume_active_scan(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    state
        .scan_controller
        .resume_scan(&id)
        .await
        .map_err(AppError::BadRequest)?;
    Ok(Json(serde_json::json!({
        "status": "success",
        "message": format!("Scan {} resumed", id)
    })))
}

pub async fn cancel_active_scan(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    state
        .scan_controller
        .cancel_scan(&id)
        .await
        .map_err(AppError::BadRequest)?;
    Ok(Json(serde_json::json!({
        "status": "success",
        "message": format!("Scan {} cancelled", id)
    })))
}

pub async fn delete_active_scan(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    use crate::domain::repositories::{ActiveFindingRepository, ActiveScanRepository};

    // First delete associated findings
    state
        .active_finding_repo
        .delete_by_scan_id(id)
        .await
        .map_err(AppError::Internal)?;

    // Then delete the scan itself
    state
        .active_scan_repo
        .delete_scan(id)
        .await
        .map_err(AppError::Internal)?;

    Ok(Json(serde_json::json!({
        "status": "success",
        "message": format!("Scan {} deleted successfully", id)
    })))
}

pub async fn list_active_findings_filtered(
    State(state): State<Arc<AppState>>,
    Query(query): Query<crate::domain::active_vuln::ActiveFindingQueryParams>,
) -> Result<Json<serde_json::Value>, AppError> {
    use crate::domain::repositories::ActiveFindingRepository;

    let findings = state
        .active_finding_repo
        .find_by_filters(&query)
        .await
        .map_err(AppError::Internal)?;

    Ok(Json(serde_json::to_value(findings)?))
}

#[derive(serde::Deserialize)]
pub struct UpdateActiveFindingRequest {
    pub verified: bool,
    pub false_positive: bool,
}

pub async fn update_active_finding(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
    Json(payload): Json<UpdateActiveFindingRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    use crate::domain::repositories::ActiveFindingRepository;

    state
        .active_finding_repo
        .update_status(id, payload.verified, payload.false_positive)
        .await
        .map_err(AppError::Internal)?;

    Ok(Json(serde_json::json!({
        "status": "success",
        "message": format!("Finding {} updated successfully", id)
    })))
}

pub async fn generate_poc_for_finding(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    use crate::domain::entities::*;
    use crate::domain::repositories::ActiveFindingRepository;

    // 1. Fetch the active finding
    let finding = state
        .active_finding_repo
        .find_by_id(id)
        .await
        .map_err(AppError::Internal)?
        .ok_or_else(|| AppError::NotFound(format!("Finding not found: {}", id)))?;

    // 2. Map ActiveVulnType → PocType
    let poc_type = match finding.vuln_type {
        crate::domain::active_vuln::ActiveVulnType::SqlInjectionError
        | crate::domain::active_vuln::ActiveVulnType::SqlInjectionUnion
        | crate::domain::active_vuln::ActiveVulnType::SqlInjectionBlindTime
        | crate::domain::active_vuln::ActiveVulnType::SqlInjectionBlindBoolean
        | crate::domain::active_vuln::ActiveVulnType::NoSqlInjection => PocType::SqlInjection,

        crate::domain::active_vuln::ActiveVulnType::CommandInjection
        | crate::domain::active_vuln::ActiveVulnType::CommandInjectionBlind => {
            PocType::CommandInjection
        }

        crate::domain::active_vuln::ActiveVulnType::LocalFileInclusion
        | crate::domain::active_vuln::ActiveVulnType::RemoteFileInclusion
        | crate::domain::active_vuln::ActiveVulnType::PathTraversal => PocType::PathTraversal,

        crate::domain::active_vuln::ActiveVulnType::ServerSideRequestForgery => {
            PocType::ServerSideRequestForgery
        }

        crate::domain::active_vuln::ActiveVulnType::XmlExternalEntity => PocType::XmlExternalEntity,

        crate::domain::active_vuln::ActiveVulnType::ReflectedXss
        | crate::domain::active_vuln::ActiveVulnType::StoredXss
        | crate::domain::active_vuln::ActiveVulnType::DomXss => PocType::CrossSiteScripting,

        _ => PocType::CommandInjection, // generic fallback
    };

    // 3. Generate PoC code directly
    let poc_code = format!(
        "#!/usr/bin/env python3\n\
         # PoC for: {:?}\n\
         # Target: {}\n\
         # Parameter: {}\n\
         # Method: {}\n\
         # Severity: {:?}\n\
         \n\
         import requests\n\
         \n\
         TARGET = \"{}\"\n\
         PAYLOAD = \"{}\"\n\
         \n\
         def exploit():\n\
             resp = requests.{}(\n\
                 TARGET,\n\
                 params={{\"{}\":PAYLOAD}} if \"{}\" == \"GET\" else None,\n\
                 data={{\"{}\":PAYLOAD}} if \"{}\" != \"GET\" else None,\n\
                 timeout=10,\n\
                 allow_redirects=False\n\
             )\n\
             print(f\"[*] Status: {{resp.status_code}}\")\n\
             print(f\"[*] Response length: {{len(resp.text)}}\")\n\
             if PAYLOAD in resp.text:\n\
                 print(\"[+] VULNERABLE - Payload reflected in response!\")\n\
             else:\n\
                 print(\"[-] Payload not reflected — manual verification needed.\")\n\
         \n\
         if __name__ == \"__main__\":\n\
             exploit()\n",
        finding.vuln_type,
        finding.target_url,
        finding.affected_parameter,
        finding.http_method,
        finding.severity,
        finding.target_url,
        finding.payload_used.replace('"', "\\\""),
        finding.http_method.to_lowercase(),
        finding.affected_parameter,
        finding.http_method,
        finding.affected_parameter,
        finding.http_method,
    );

    // 4. Create and save the PoC entity
    let poc = Poc {
        id: uuid::Uuid::new_v4(),
        finding_id: id,
        poc_type,
        code: poc_code,
        language: PocLanguage::Python,
        safety_level: SafetyLevel::L1SafeReadOnly,
        verification_status: ExploitVerificationStatus::Pending,
        created_at: chrono::Utc::now(),
    };

    use crate::domain::repositories::PocRepository;
    state
        .poc_repo
        .save(&poc)
        .await
        .map_err(AppError::Internal)?;

    // 5. Update active finding with poc_id
    state
        .active_finding_repo
        .update_poc_id(id, poc.id)
        .await
        .map_err(AppError::Internal)?;

    Ok(Json(serde_json::to_value(poc)?))
}

#[derive(serde::Deserialize)]
pub struct VerifyFindingPayload {
    pub execution_level: Option<crate::domain::entities::SafetyLevel>,
}

pub async fn verify_finding(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
    Json(payload_data): Json<VerifyFindingPayload>,
) -> Result<Json<serde_json::Value>, AppError> {
    use crate::domain::repositories::ActiveFindingRepository;

    let finding = state
        .active_finding_repo
        .find_by_id(id)
        .await
        .map_err(AppError::Internal)?
        .ok_or_else(|| AppError::NotFound(format!("Finding not found: {}", id)))?;

    if let Some(poc_id) = finding.poc_id {
        let payload = crate::application::use_cases::verify_exploit::VerifyExploitRequest {
            poc_id,
            execution_level: payload_data
                .execution_level
                .unwrap_or(crate::domain::entities::SafetyLevel::L2VerifiedSandbox),
            target_url: finding.target_url.clone(),
        };

        // Audit log if L3 is selected
        if matches!(
            payload.execution_level,
            crate::domain::entities::SafetyLevel::L3ActiveWithConsent
        ) {
            let _ = state
                .safety_framework
                .log_audit(
                    "L3_EXPLOIT_EXECUTED_VIA_FINDING",
                    Some(&format!("Finding ID: {}, PoC ID: {}", finding.id, poc_id)),
                    Some(&finding.target_url),
                    None,
                )
                .await;
        }

        let use_case = crate::application::use_cases::verify_exploit::VerifyExploit {
            poc_repo: &*state.poc_repo,
            exploit_result_repo: &*state.exploit_result_repo,
            exploit_bridge: &state.exploit_bridge,
        };

        let result = use_case
            .execute(payload)
            .await
            .map_err(AppError::Internal)?;

        // Update finding status based on verification result
        let verified = result.success;
        let false_positive = !result.success;

        state
            .active_finding_repo
            .update_status(id, verified, false_positive)
            .await
            .map_err(AppError::Internal)?;

        Ok(Json(serde_json::to_value(result)?))
    } else {
        Err(AppError::BadRequest(format!(
            "Finding {} does not have a generated POC to verify",
            id
        )))
    }
}

// ── Consent Management Handlers ──

#[derive(serde::Deserialize)]
pub struct GrantConsentPayload {
    pub target_domain: String,
    pub requested_by: String,
    pub scope_level: String,
    pub expires_in_hours: i64,
}

pub async fn grant_consent_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<GrantConsentPayload>,
) -> Result<Json<serde_json::Value>, AppError> {
    let id = state
        .safety_framework
        .grant_consent(
            &payload.target_domain,
            &payload.requested_by,
            &payload.scope_level,
            payload.expires_in_hours,
        )
        .await
        .map_err(AppError::BadRequest)?;

    // Audit log
    let _ = state
        .safety_framework
        .log_audit(
            "CONSENT_GRANTED",
            Some(&format!(
                "Level: {}, Expires in {}h",
                payload.scope_level, payload.expires_in_hours
            )),
            Some(&payload.target_domain),
            Some(&payload.requested_by),
        )
        .await;

    Ok(Json(serde_json::json!({
        "status": "success",
        "consent_id": id
    })))
}

#[derive(serde::Deserialize)]
pub struct RevokeConsentPayload {
    pub target_domain: String,
}

pub async fn revoke_consent_handler(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
    Json(payload): Json<RevokeConsentPayload>,
) -> Result<Json<serde_json::Value>, AppError> {
    state
        .safety_framework
        .revoke_consent(id, &payload.target_domain)
        .await
        .map_err(AppError::BadRequest)?;

    // Audit log
    let _ = state
        .safety_framework
        .log_audit(
            "CONSENT_REVOKED",
            Some(&format!("Revoked consent ID: {}", id)),
            Some(&payload.target_domain),
            None,
        )
        .await;

    Ok(Json(serde_json::json!({
        "status": "success"
    })))
}

pub async fn get_consents_handler(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<crate::domain::entities::ConsentRecord>>, AppError> {
    let consents = state
        .safety_framework
        .get_consents()
        .await
        .map_err(AppError::Internal)?;
    Ok(Json(consents))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn private_ip_classifier_blocks_internal_ranges() {
        assert!(is_blocked_ip("127.0.0.1".parse().unwrap()));
        assert!(is_blocked_ip("10.0.0.1".parse().unwrap()));
        assert!(is_blocked_ip("172.16.0.1".parse().unwrap()));
        assert!(is_blocked_ip("192.168.1.1".parse().unwrap()));
        assert!(is_blocked_ip("169.254.169.254".parse().unwrap()));
        assert!(is_blocked_ip("::1".parse().unwrap()));
        assert!(is_blocked_ip("fc00::1".parse().unwrap()));
        assert!(!is_blocked_ip("8.8.8.8".parse().unwrap()));
        assert!(!is_blocked_ip("2001:4860:4860::8888".parse().unwrap()));
    }

    #[test]
    fn external_url_validator_rejects_bad_schemes_and_localhost() {
        let previous = std::env::var("LIMMA_ALLOW_PRIVATE_TARGETS").ok();
        std::env::remove_var("LIMMA_ALLOW_PRIVATE_TARGETS");

        assert!(validate_external_url("ftp://example.com/file").is_err());
        assert!(validate_external_url("http://localhost:8080").is_err());
        assert!(validate_external_url("http://127.0.0.1:8080").is_err());
        assert!(validate_external_url("https://example.com").is_ok());

        if let Some(value) = previous {
            std::env::set_var("LIMMA_ALLOW_PRIVATE_TARGETS", value);
        }
    }
}
