use limma::api::handlers::{
    analyze_website, analyze_website_stream, audit_security, blind_scan, burp_get_findings,
    burp_handshake, burp_import_traffic, burp_list_sessions, burp_stream_events, collect_services,
    create_custom_rule, delete_active_scan, delete_custom_rule, delete_history_scan, discover_apis,
    download_poc, export_to_burp, export_to_nuclei, generate_master_report, generate_poc,
    generate_poc_for_finding, get_active_finding, get_active_scan, get_consents_handler,
    get_feedback_stats, get_history_delta, get_history_trends, get_rule_engine_status,
    get_scan_by_id, get_settings_profiles, grant_consent_handler, investigate_server,
    investigate_server_stream, list_active_findings, list_active_findings_filtered,
    list_active_scans, list_scans, map_forms, proxy_request, revoke_consent_handler,
    start_active_scan, submit_feedback, submit_rule_feedback, update_active_finding,
    update_settings_profile, verify_exploit, verify_finding, verify_port, AppState,
};
use limma::infrastructure::auditor::HttpSecurityAuditor;
use limma::infrastructure::burp_bridge::BurpBridgeManager;
use limma::infrastructure::collector::HttpServiceCollector;
use limma::infrastructure::db::init_db;
use limma::infrastructure::delta_engine::DeltaEngine;
use limma::infrastructure::discoverer::HttpApiDiscoverer;
use limma::infrastructure::investigator::HttpInvestigator;
use limma::infrastructure::mapper::HttpFormMapper;

use anyhow::Context;
use axum::{routing::post, Router};
use limma::infrastructure::repositories::pg_settings::PgSettingsRepository;
use limma::infrastructure::rule_engine::{resolve_rules_dir, DynamicRuleEngine};
use limma::infrastructure::scanner::HttpWebsiteScanner;
use std::sync::Arc;
use std::time::Duration;
// Rate limiter disabled for development
// use tower_governor::{governor::GovernorConfigBuilder, GovernorLayer};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Install ring as the default TLS crypto provider
    rustls::crypto::ring::default_provider()
        .install_default()
        .map_err(|_| anyhow::anyhow!("Failed to install rustls CryptoProvider"))?;

    // Initializing tracing
    tracing_subscriber::fmt::init();

    // Database Initialization
    dotenvy::dotenv().ok();
    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgres://postgres:password@127.0.0.1:5432/limma?sslmode=disable".to_string()
    });

    tracing::info!("[DB] Connecting to: {}", database_url);
    let pool = init_db(&database_url)
        .await
        .context("Failed to initialize database")?;
    tracing::info!("[DB] Connected successfully");

    // Dependency Injection

    let website_scanner = Arc::new(HttpWebsiteScanner::new());
    let server_investigator = Arc::new(HttpInvestigator::new());
    let api_discoverer = Arc::new(HttpApiDiscoverer::new());
    let service_collector = Arc::new(HttpServiceCollector::new());
    let security_auditor = Arc::new(HttpSecurityAuditor::new().with_pool(pool.clone()));
    let form_mapper = Arc::new(HttpFormMapper::new());

    // Dynamic Rule Engine — loads YAML/JSON rules from /rules directory at startup
    let rules_dir = resolve_rules_dir();
    let dynamic_rule_engine = Arc::new(DynamicRuleEngine::new(&rules_dir));
    tracing::info!(
        "[DynamicRuleEngine] {} active rules loaded from {}",
        dynamic_rule_engine.rule_count(),
        rules_dir
    );

    // Load custom rules from DB and hot-load into the engine
    {
        let rows = sqlx::query_as::<_, (String, String, String)>(
            "SELECT id, name, yaml_content FROM custom_rules",
        )
        .fetch_all(&pool)
        .await
        .unwrap_or_default();

        let mut loaded = 0usize;
        for (_id, _name, yaml_content) in &rows {
            match serde_yaml::from_str::<limma::infrastructure::rule_engine::models::RuleDefinition>(
                yaml_content,
            ) {
                Ok(rule_def) => {
                    dynamic_rule_engine.add_rule(rule_def);
                    loaded += 1;
                }
                Err(e) => {
                    tracing::warn!("[CustomRules] Failed to parse custom rule '{}': {}", _id, e);
                }
            }
        }
        if loaded > 0 {
            tracing::info!("[CustomRules] {} custom rules loaded from database", loaded);
        }
    }

    let burp_bridge = Arc::new(BurpBridgeManager::new(pool.clone()).await);

    // Delta Engine
    let delta_engine = Arc::new(DeltaEngine::new(pool.clone()));

    // ── Faz F: Blind Detection & Exploitation ──
    let blind_detection_engine =
        Arc::new(limma::infrastructure::blind_detection::HttpBlindDetectionEngine::new());
    let poc_generator =
        Arc::new(limma::infrastructure::exploitation::poc_generator::CompositePocGenerator::new());
    let sandbox_verifier: Arc<dyn limma::infrastructure::exploitation::sandbox::SandboxVerifier> =
        match limma::infrastructure::exploitation::sandbox::docker_sandbox::DockerSandbox::new(30) {
            Ok(sandbox) => {
                tracing::info!("[Faz F] Sandbox: DockerSandbox initialized successfully");
                Arc::new(sandbox)
            }
            Err(e) => {
                tracing::warn!("[Faz F] Failed to init DockerSandbox: {}. Falling back to NoopSandboxProvider.", e);
                Arc::new(limma::infrastructure::exploitation::sandbox::NoopSandboxProvider::new())
            }
        };

    let safety_framework = Arc::new(limma::infrastructure::safety::SafetyFrameworkImpl::new(
        pool.clone(),
        vec![], // Open scope: all domains allowed
        60,     // 60 requests per minute rate limit
    ));

    let exploit_bridge = Arc::new(
        limma::infrastructure::exploitation::exploit_bridge::ExploitBridge::new(
            safety_framework.clone(),
            sandbox_verifier.clone(),
        ),
    );
    let blind_finding_repo = Arc::new(
        limma::infrastructure::repositories::blind_finding_repo::PgBlindFindingRepository::new(
            pool.clone(),
        ),
    );
    let poc_repo =
        Arc::new(limma::infrastructure::repositories::poc_repo::PgPocRepository::new(pool.clone()));
    let exploit_result_repo = Arc::new(
        limma::infrastructure::repositories::exploit_result_repo::PgExploitResultRepository::new(
            pool.clone(),
        ),
    );
    tracing::info!("[Faz F] Blind detection engine, PoC generator, safety framework, and exploit bridge initialized");

    // ── Faz 4: System Settings PostgreSQL Store ──
    let settings_repo = Arc::new(PgSettingsRepository::new(pool.clone()));
    use limma::domain::repositories::SettingsRepository;
    settings_repo
        .init_defaults()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to init default settings: {}", e))?;
    tracing::info!("[Faz 4] PgSettingsRepository initialized with PostgreSQL persistence");

    // ── Faz 1: Active Vulnerability Engine ──
    let payload_db =
        Arc::new(limma::infrastructure::active_detection::payloads::PayloadDatabase::new());

    // We create a generic client for detectors. In a real app, this might be per-profile
    let req_client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .context("Failed to build reqwest client for detectors")?;

    let active_detectors: Arc<Vec<Box<dyn limma::infrastructure::active_detection::detectors::VulnDetector>>> = Arc::new(vec![
        Box::new(limma::infrastructure::active_detection::detectors::xss_detector::XssDetector::new(req_client.clone())),
        Box::new(limma::infrastructure::active_detection::detectors::sqli_detector::SqliDetector::new(req_client.clone())),
        Box::new(limma::infrastructure::active_detection::detectors::cmdi_detector::CmdiDetector::new(req_client.clone())),
        Box::new(limma::infrastructure::active_detection::detectors::lfi_detector::LfiDetector::new(req_client.clone())),
        Box::new(limma::infrastructure::active_detection::detectors::ssrf_detector::SsrfDetector::new(req_client.clone())),
        Box::new(limma::infrastructure::active_detection::detectors::xxe_detector::XxeDetector::new(req_client.clone())),
        Box::new(limma::infrastructure::active_detection::detectors::redirect_detector::RedirectDetector::new(req_client.clone())),
        Box::new(limma::infrastructure::active_detection::detectors::jwt_detector::JwtDetector::new(req_client.clone())),
        Box::new(limma::infrastructure::active_detection::detectors::deser_detector::DeserDetector::new(req_client.clone())),
        Box::new(limma::infrastructure::active_detection::detectors::idor_detector::IdorDetector::new(req_client.clone())),
        Box::new(limma::infrastructure::active_detection::detectors::nosql_detector::NosqlDetector::new(req_client.clone())),
        Box::new(limma::infrastructure::active_detection::detectors::ssti_detector::SstiDetector::new(req_client.clone())),
    ]);
    let active_scan_repo = Arc::new(
        limma::infrastructure::repositories::active_scan_repo::PgActiveScanRepository::new(
            pool.clone(),
        ),
    );
    let active_finding_repo = Arc::new(
        limma::infrastructure::repositories::active_finding_repo::PgActiveFindingRepository::new(
            pool.clone(),
        ),
    );
    tracing::info!(
        "[Faz 1] Active Detection Engine initialized with {} vulnerability detectors",
        active_detectors.len()
    );

    let shared_state = Arc::new(AppState {
        website_scanner,
        server_investigator,
        api_discoverer,
        service_collector,
        security_auditor,
        form_mapper,
        dynamic_rule_engine,
        burp_bridge,
        delta_engine,
        blind_detection_engine,
        poc_generator,
        safety_framework: safety_framework.clone(),
        exploit_bridge: exploit_bridge.clone(),
        blind_finding_repo: blind_finding_repo.clone(),
        poc_repo,
        exploit_result_repo,
        settings_repo,
        active_scan_repo,
        active_finding_repo,
        active_detectors,
        payload_db,
        db_pool: pool,
    });

    // Configure middleware layers
    let cors = tower_http::cors::CorsLayer::new()
        .allow_origin(tower_http::cors::Any)
        .allow_headers(tower_http::cors::Any)
        .allow_methods(tower_http::cors::Any);
    // Rate limiter disabled for development

    // Build our application with routes
    let app = Router::new()
        .route("/analyze", post(analyze_website))
        .route(
            "/analyze/stream",
            axum::routing::get(analyze_website_stream),
        )
        .route("/investigate", post(investigate_server))
        .route(
            "/investigate/stream",
            axum::routing::get(investigate_server_stream),
        )
        .route("/discover-apis", post(discover_apis))
        .route("/collect-services", post(collect_services))
        .route("/audit-security", post(audit_security))
        .route("/map-forms", post(map_forms))
        .route("/master-report", post(generate_master_report))
        .route("/proxy-request", post(proxy_request))
        .route("/verify-port", post(verify_port))
        .route("/api/feedback", post(submit_feedback))
        .route(
            "/api/rule-engine-status",
            axum::routing::get(get_rule_engine_status),
        )
        .route("/api/dynamic-rule/feedback", post(submit_rule_feedback))
        .route(
            "/api/feedback-stats",
            axum::routing::get(get_feedback_stats),
        )
        // Custom Rules CRUD
        .route("/api/rules", post(create_custom_rule))
        .route("/api/rules/:id", axum::routing::delete(delete_custom_rule))
        .route(
            "/api/settings/consent",
            post(grant_consent_handler).get(get_consents_handler),
        )
        .route(
            "/api/settings/consent/:id",
            axum::routing::delete(revoke_consent_handler),
        )
        .route("/api/export/burp", post(export_to_burp))
        .route("/api/export/nuclei", post(export_to_nuclei))
        .route("/api/burp/handshake", post(burp_handshake))
        .route("/api/burp/import-traffic", post(burp_import_traffic))
        .route(
            "/api/burp/findings/:session_id",
            axum::routing::get(burp_get_findings),
        )
        .route(
            "/api/burp/stream/:session_id",
            axum::routing::get(burp_stream_events),
        )
        .route("/api/burp/sessions", axum::routing::get(burp_list_sessions))
        .route(
            "/api/history/trends",
            axum::routing::get(get_history_trends),
        )
        .route("/api/history/delta", axum::routing::get(get_history_delta))
        .route(
            "/api/history/scan/:scan_id",
            axum::routing::get(get_scan_by_id).delete(delete_history_scan),
        )
        .route("/api/history/scans", axum::routing::get(list_scans))
        // Faz 4: Settings API
        .route(
            "/api/settings/profiles",
            axum::routing::get(get_settings_profiles),
        )
        .route(
            "/api/settings/profiles/:id",
            axum::routing::put(update_settings_profile),
        )
        // Faz F: Blind Detection & Exploitation routes
        .route("/api/blind-scan", post(blind_scan))
        .route("/api/poc/generate", post(generate_poc))
        .route("/api/exploit/verify", post(verify_exploit))
        .route("/api/poc/:id", axum::routing::get(download_poc))
        // Faz 1: Active Vulnerability Detection routes
        .route("/api/active-scan", post(start_active_scan))
        .route("/api/active-scan/:id", axum::routing::get(get_active_scan))
        .route(
            "/api/active-scan/:scan_id/findings",
            axum::routing::get(list_active_findings),
        )
        .route(
            "/api/active-finding/:id",
            axum::routing::get(get_active_finding),
        )
        .route("/api/active-scans", axum::routing::get(list_active_scans))
        .route(
            "/api/active-scans/:id",
            axum::routing::delete(delete_active_scan),
        )
        .route(
            "/api/active-findings",
            axum::routing::get(list_active_findings_filtered),
        )
        .route(
            "/api/active-findings/:id",
            axum::routing::patch(update_active_finding),
        )
        .route(
            "/api/active-findings/:id/poc",
            post(generate_poc_for_finding),
        )
        .route("/api/active-findings/:id/verify", post(verify_finding))
        .with_state(shared_state)
        .layer(tower_http::timeout::TimeoutLayer::new(Duration::from_secs(
            300,
        )))
        // GovernorLayer (rate limiter) disabled for development
        .layer(cors);

    // Run our app
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8900")
        .await
        .context("Failed to bind to 0.0.0.0:8900")?;
    tracing::info!("[Server]: Limma Rust Backend listening on 8900");
    tracing::info!("- POST /master-report (Full Website Intelligence Report)");
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .await
    .context("Server exited unexpectedly")?;

    Ok(())
}
