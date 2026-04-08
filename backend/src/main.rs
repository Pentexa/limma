mod domain;
mod application;
mod infrastructure;
mod api;

use axum::{
    routing::post,
    Router,
};
use std::sync::Arc;
use crate::infrastructure::persistence::InMemoryUserRepository;
use crate::infrastructure::scanner::HttpWebsiteScanner;
use crate::infrastructure::investigator::HttpInvestigator;
use crate::infrastructure::discoverer::HttpApiDiscoverer;
use crate::infrastructure::collector::HttpServiceCollector;
use crate::infrastructure::auditor::HttpSecurityAuditor;
use crate::infrastructure::mapper::HttpFormMapper;
use crate::api::handlers::{register_user, analyze_website, analyze_website_stream, investigate_server, investigate_server_stream, discover_apis, collect_services, audit_security, map_forms, generate_master_report, proxy_request, verify_port, submit_feedback, AppState};

#[tokio::main]
async fn main() {
    // Install ring as the default TLS crypto provider (required when both ring + aws-lc-rs are linked)
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install rustls CryptoProvider");

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

    // Configure CORS
    let cors = tower_http::cors::CorsLayer::new()
        .allow_origin(tower_http::cors::Any)
        .allow_headers(tower_http::cors::Any)
        .allow_methods(tower_http::cors::Any);

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
        .layer(cors);

    // Run our app
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8900").await.unwrap();
    println!("[Server]: Limma Rust Backend (Clean Architecture) listening on 8900");
    println!("- POST /master-report (Full Website Intelligence Report)");
    axum::serve(listener, app).await.unwrap();
}
