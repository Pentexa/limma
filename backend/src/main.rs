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
use crate::infrastructure::persistence::InMemoryUserRepository;
use crate::infrastructure::scanner::HttpWebsiteScanner;
use crate::infrastructure::investigator::HttpInvestigator;
use crate::infrastructure::discoverer::HttpApiDiscoverer;
use crate::infrastructure::collector::HttpServiceCollector;
use crate::infrastructure::auditor::HttpSecurityAuditor;
use crate::infrastructure::mapper::HttpFormMapper;
use crate::api::handlers::{register_user, analyze_website, analyze_website_stream, investigate_server, investigate_server_stream, discover_apis, collect_services, audit_security, map_forms, generate_master_report, proxy_request, verify_port, submit_feedback, AppState};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Install ring as the default TLS crypto provider
    rustls::crypto::ring::default_provider()
        .install_default()
        .map_err(|_| anyhow::anyhow!("Failed to install rustls CryptoProvider"))?;

    // Initializing tracing
    tracing_subscriber::fmt::init();

    // Dependency Injection
    let user_repo = Arc::new(InMemoryUserRepository::new());
    let website_scanner = Arc::new(HttpWebsiteScanner::new());
    let server_investigator = Arc::new(HttpInvestigator::new());
    let api_discoverer = Arc::new(HttpApiDiscoverer::new());
    let service_collector = Arc::new(HttpServiceCollector::new());
    let security_auditor = Arc::new(HttpSecurityAuditor::new());
    let form_mapper = Arc::new(HttpFormMapper::new());
    
    let shared_state = Arc::new(AppState {
        user_repo,
        website_scanner,
        server_investigator,
        api_discoverer,
        service_collector,
        security_auditor,
        form_mapper,
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
        .route("/users", post(register_user))
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
