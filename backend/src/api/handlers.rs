use axum::{extract::State, Json, http::StatusCode};
use std::sync::Arc;
use crate::api::models::{CreateUserRequest, AnalysisRequest, ProxyRequest, VerifyPortRequest, VerifyPortResponse, FeedbackRequest};
use crate::application::use_cases::{RegisterUser, AnalyzeWebsite, InvestigateServer, DiscoverApis, CollectExternalServices, AuditSecurity, MapForms, GenerateMasterReport};
use crate::error::AppError;
use crate::infrastructure::persistence::InMemoryUserRepository;
use crate::infrastructure::scanner::HttpWebsiteScanner;
use crate::infrastructure::investigator::HttpInvestigator;
use crate::infrastructure::discoverer::HttpApiDiscoverer;
use crate::infrastructure::collector::HttpServiceCollector;
use crate::infrastructure::auditor::HttpSecurityAuditor;
use crate::infrastructure::mapper::HttpFormMapper;

pub struct AppState {
    pub user_repo: Arc<InMemoryUserRepository>,
    pub website_scanner: Arc<HttpWebsiteScanner>,
    pub server_investigator: Arc<HttpInvestigator>,
    pub api_discoverer: Arc<HttpApiDiscoverer>,
    pub service_collector: Arc<HttpServiceCollector>,
    pub security_auditor: Arc<HttpSecurityAuditor>,
    pub form_mapper: Arc<HttpFormMapper>,
}

pub async fn register_user(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateUserRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    let use_case = RegisterUser { repo: &*state.user_repo };
    
    match use_case.execute(payload.name, payload.email).await {
        Ok(user) => {
            let value = serde_json::to_value(user)?;
            Ok((StatusCode::CREATED, Json(value)))
        },
        Err(e) => Err(AppError::BadRequest(e)),
    }
}

pub async fn analyze_website(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<AnalysisRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let use_case = AnalyzeWebsite { scanner: &*state.website_scanner };
    
    let analysis = use_case.execute(payload.url).await.map_err(AppError::Internal)?;
    let value = serde_json::to_value(analysis)?;
    Ok(Json(value))
}

use axum::response::sse::{Event, Sse};
use tokio_stream::StreamExt;
use std::convert::Infallible;
use axum::extract::Query;
use serde::Deserialize;

use crate::domain::repositories::{WebsiteScanner, ServerInvestigator};

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
    let use_case = InvestigateServer { investigator: &*state.server_investigator };
    
    let info = use_case.execute(payload.url).await.map_err(AppError::Internal)?;
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
    let use_case = DiscoverApis { discoverer: &*state.api_discoverer };
    
    let res = use_case.execute(payload.url).await.map_err(AppError::Internal)?;
    let value = serde_json::to_value(res)?;
    Ok(Json(value))
}

pub async fn collect_services(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<AnalysisRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let use_case = CollectExternalServices { collector: &*state.service_collector };
    
    let res = use_case.execute(payload.url).await.map_err(AppError::Internal)?;
    let value = serde_json::to_value(res)?;
    Ok(Json(value))
}

pub async fn audit_security(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<AnalysisRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let use_case = AuditSecurity { auditor: &*state.security_auditor };
    
    let res = use_case.execute(payload.url).await.map_err(AppError::Internal)?;
    let value = serde_json::to_value(res)?;
    Ok(Json(value))
}

pub async fn map_forms(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<AnalysisRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let use_case = MapForms { mapper: &*state.form_mapper };
    
    let res = use_case.execute(payload.url).await.map_err(AppError::Internal)?;
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
    };

    let res = use_case.execute(payload.url).await.map_err(AppError::Internal)?;
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
            let parsed_body: serde_json::Value = serde_json::from_str(&text)
                .unwrap_or(serde_json::Value::String(text));
            Ok((status, Json(parsed_body)))
        },
        Err(e) => Err(AppError::BadGateway(e.to_string())),
    }
}

pub async fn verify_port(
    Json(payload): Json<VerifyPortRequest>,
) -> (StatusCode, Json<VerifyPortResponse>) {
    use tokio::net::TcpStream;
    use tokio::time::{timeout, Duration};
    use std::time::Instant;

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

            (StatusCode::OK, Json(VerifyPortResponse {
                is_active: true,
                latency_ms: Some(latency_ms),
                banner,
            }))
        },
        _ => {
            (StatusCode::OK, Json(VerifyPortResponse {
                is_active: false,
                latency_ms: None,
                banner: None,
            }))
        }
    }
}

pub async fn submit_feedback(
    Json(payload): Json<FeedbackRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let mut engine = crate::infrastructure::auditor::learning_feedback::LearningFeedbackEngine::new();
    engine.record_feedback(payload.signature.clone(), payload.action.clone());
    
    Ok(Json(serde_json::json!({
        "status": "success",
        "message": format!("Recorded feedback {:?} for signature: {}", payload.action, payload.signature)
    })))
}
