use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // user id
    pub exp: usize,  // expiration timestamp
    pub iat: usize,  // issued at
}

/// Create a JWT token for the given user_id, valid for 24 hours.
pub fn create_token(user_id: Uuid, secret: &str) -> Result<String, String> {
    let now = chrono::Utc::now().timestamp() as usize;
    let claims = Claims {
        sub: user_id.to_string(),
        exp: now + 86400, // 24 hours
        iat: now,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    )
    .map_err(|e| format!("Token creation failed: {}", e))
}

/// Verify and decode a JWT token, returning the claims if valid.
pub fn verify_token(token: &str, secret: &str) -> Result<Claims, String> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::default(),
    )
    .map_err(|e| format!("Token verification failed: {}", e))?;

    Ok(token_data.claims)
}

/// Extract user UUID from validated claims.
pub fn user_id_from_claims(claims: &Claims) -> Result<Uuid, String> {
    Uuid::parse_str(&claims.sub).map_err(|e| format!("Invalid user ID in token: {}", e))
}
