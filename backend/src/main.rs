mod api;
mod application;
mod domain;
mod error;
mod infrastructure;

use crate::api::handlers::{
    analyze_website, analyze_website_stream, audit_security, burp_get_findings, burp_handshake,
    burp_import_traffic, burp_list_sessions, burp_stream_events, collect_services, discover_apis,
    export_to_burp, export_to_nuclei, generate_master_report, get_feedback_stats,
    get_history_delta, get_history_trends, get_me, get_rule_engine_status, get_scan_by_id,
    investigate_server, investigate_server_stream, list_scans, login_user, map_forms,
    proxy_request, register_user, submit_feedback, submit_rule_feedback, verify_port, AppState,
};
use crate::infrastructure::auditor::HttpSecurityAuditor;
use crate::infrastructure::burp_bridge::BurpBridgeManager;
use crate::infrastructure::collector::HttpServiceCollector;
use crate::infrastructure::db::init_db;
use crate::infrastructure::delta_engine::DeltaEngine;
use crate::infrastructure::discoverer::HttpApiDiscoverer;
use crate::infrastructure::investigator::HttpInvestigator;
use crate::infrastructure::mapper::HttpFormMapper;
use crate::infrastructure::persistence::PgUserRepository;
use crate::infrastructure::rule_engine::{resolve_rules_dir, DynamicRuleEngine};
use crate::infrastructure::scanner::HttpWebsiteScanner;
use anyhow::Context;
use axum::{routing::post, Router};
use std::sync::Arc;
use std::time::Duration;
use tower_governor::{governor::GovernorConfigBuilder, GovernorLayer};

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
    let jwt_secret = std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "default_dev_secret_change_in_production".to_string());
    tracing::info!("[DB] Connecting to: {}", database_url);
    let pool = init_db(&database_url)
        .await
        .context("Failed to initialize database")?;
    tracing::info!("[DB] Connected successfully");

    // Dependency Injection
    let user_repo = Arc::new(PgUserRepository::new(pool.clone()));
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

    let burp_bridge = Arc::new(BurpBridgeManager::new(pool.clone()).await);

    // Delta Engine
    let delta_engine = Arc::new(DeltaEngine::new(pool.clone()));

    let shared_state = Arc::new(AppState {
        user_repo,
        website_scanner,
        server_investigator,
        api_discoverer,
        service_collector,
        security_auditor,
        form_mapper,
        dynamic_rule_engine,
        burp_bridge,
        delta_engine,
        db_pool: pool,
        jwt_secret,
    });

    // Configure middleware layers
    let cors = tower_http::cors::CorsLayer::new()
        .allow_origin(tower_http::cors::Any)
        .allow_headers(tower_http::cors::Any)
        .allow_methods(tower_http::cors::Any);
    let governor_config = Arc::new(
        GovernorConfigBuilder::default()
            .per_second(20)
            .burst_size(40)
            .finish()
            .context("Failed to build rate limit configuration")?,
    );

    // Build our application with routes
    let app = Router::new()
        .route("/auth/register", post(register_user))
        .route("/auth/login", post(login_user))
        .route("/auth/me", axum::routing::get(get_me))
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
            axum::routing::get(get_scan_by_id),
        )
        .route("/api/history/scans", axum::routing::get(list_scans))
        .with_state(shared_state)
        .layer(tower_http::timeout::TimeoutLayer::new(Duration::from_secs(
            300,
        )))
        .layer(GovernorLayer {
            config: governor_config,
        })
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
