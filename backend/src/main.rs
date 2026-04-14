#![allow(dead_code, unused_imports, unused_variables, unused_mut)]

mod domain;
mod application;
mod infrastructure;
mod api;
mod error;

use axum::{
    routing::post,
    Router,
};
use std::sync::Arc;
use std::time::Duration;
use anyhow::Context;
use tower_governor::{
    governor::GovernorConfigBuilder,
    GovernorLayer,
};
use crate::infrastructure::persistence::PgUserRepository;
use crate::infrastructure::db::init_db;
use crate::infrastructure::scanner::HttpWebsiteScanner;
use crate::infrastructure::investigator::HttpInvestigator;
use crate::infrastructure::discoverer::HttpApiDiscoverer;
use crate::infrastructure::collector::HttpServiceCollector;
use crate::infrastructure::auditor::HttpSecurityAuditor;
use crate::infrastructure::mapper::HttpFormMapper;
use crate::infrastructure::rule_engine::{DynamicRuleEngine, resolve_rules_dir};
use crate::api::handlers::{register_user, login_user, get_me, analyze_website, analyze_website_stream, investigate_server, investigate_server_stream, discover_apis, collect_services, audit_security, map_forms, generate_master_report, proxy_request, verify_port, submit_feedback, get_rule_engine_status, submit_rule_feedback, get_feedback_stats, AppState};

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
    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| "postgres://postgres:password@127.0.0.1:5432/limma?sslmode=disable".to_string());
    let jwt_secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "default_dev_secret_change_in_production".to_string());
    tracing::info!("[DB] Connecting to: {}", database_url);
    let pool = init_db(&database_url).await.context("Failed to initialize database")?;
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
    tracing::info!("[DynamicRuleEngine] {} active rules loaded from {}", dynamic_rule_engine.rule_count(), rules_dir);
    
    let shared_state = Arc::new(AppState {
        user_repo,
        website_scanner,
        server_investigator,
        api_discoverer,
        service_collector,
        security_auditor,
        form_mapper,
        dynamic_rule_engine,
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
        .route("/analyze/stream", axum::routing::get(analyze_website_stream))
        .route("/investigate", post(investigate_server))
        .route("/investigate/stream", axum::routing::get(investigate_server_stream))
        .route("/discover-apis", post(discover_apis))
        .route("/collect-services", post(collect_services))
        .route("/audit-security", post(audit_security))
        .route("/map-forms", post(map_forms))
        .route("/master-report", post(generate_master_report))
        .route("/proxy-request", post(proxy_request))
        .route("/verify-port", post(verify_port))
        .route("/api/feedback", post(submit_feedback))
        .route("/api/rule-engine-status", axum::routing::get(get_rule_engine_status))
        .route("/api/dynamic-rule/feedback", post(submit_rule_feedback))
        .route("/api/feedback-stats", axum::routing::get(get_feedback_stats))
        .with_state(shared_state)
        .layer(tower_http::timeout::TimeoutLayer::new(Duration::from_secs(300)))
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
    axum::serve(listener, app.into_make_service_with_connect_info::<std::net::SocketAddr>())
        .await
        .context("Server exited unexpectedly")?;

    Ok(())
}
