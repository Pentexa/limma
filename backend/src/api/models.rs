use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct CreateUserRequest {
    pub name: String,
    pub email: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: UserPublic,
}

#[derive(Serialize)]
pub struct UserPublic {
    pub id: String,
    pub name: String,
    pub email: String,
}

#[derive(Serialize)]
pub struct UserResponse {
    pub id: String,
    pub name: String,
    pub email: String,
}

#[derive(Deserialize)]
pub struct AnalysisRequest {
    pub url: String,
}

#[derive(Deserialize)]
pub struct ProxyRequest {
    pub url: String,
    pub method: String,
    pub body: Option<String>,
}

#[derive(Deserialize)]
pub struct VerifyPortRequest {
    pub host: String,
    pub port: u16,
}

#[derive(Serialize)]
pub struct VerifyPortResponse {
    pub is_active: bool,
    pub latency_ms: Option<u64>,
    pub banner: Option<String>,
}

#[derive(Deserialize)]
pub struct FeedbackRequest {
    pub signature: String,
    pub action: crate::domain::entities::FeedbackAction,
}
