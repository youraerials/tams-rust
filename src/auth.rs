use crate::{config::AuthConfig, error::TamsError};
use axum::{
    extract::{Request, State},
    http::{header::AUTHORIZATION, HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use base64::prelude::*;
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // Subject (user ID)
    pub exp: usize,  // Expiration time
    pub iat: usize,  // Issued at
}

pub struct AuthState {
    pub config: AuthConfig,
    pub decoding_key: DecodingKey,
}

impl AuthState {
    pub fn new(config: AuthConfig) -> Self {
        let decoding_key = DecodingKey::from_secret(config.jwt_secret.as_bytes());
        Self {
            config,
            decoding_key,
        }
    }
}

pub async fn auth_middleware(
    State(auth_state): State<Arc<AuthState>>,
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response, TamsError> {
    // Skip authentication if not required
    if !auth_state.config.require_auth {
        return Ok(next.run(request).await);
    }

    let auth_header = headers
        .get(AUTHORIZATION)
        .and_then(|header| header.to_str().ok())
        .ok_or_else(|| TamsError::Unauthorized("Missing Authorization header".to_string()))?;

    // Try JWT Bearer token first
    if auth_header.starts_with("Bearer ") {
        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or_else(|| TamsError::Unauthorized("Invalid Bearer token format".to_string()))?;

        validate_jwt_token(token, &auth_state.decoding_key)?;
    }
    // Try Basic auth
    else if auth_header.starts_with("Basic ") {
        let encoded = auth_header
            .strip_prefix("Basic ")
            .ok_or_else(|| TamsError::Unauthorized("Invalid Basic auth format".to_string()))?;

        validate_basic_auth(encoded, &auth_state.config)?;
    } else {
        return Err(TamsError::Unauthorized(
            "Unsupported authentication method".to_string(),
        ));
    }

    Ok(next.run(request).await)
}

fn validate_jwt_token(token: &str, decoding_key: &DecodingKey) -> Result<Claims, TamsError> {
    let validation = Validation::default();
    
    match decode::<Claims>(token, decoding_key, &validation) {
        Ok(token_data) => Ok(token_data.claims),
        Err(e) => Err(TamsError::Unauthorized(format!("Invalid JWT token: {}", e))),
    }
}

fn validate_basic_auth(encoded: &str, config: &AuthConfig) -> Result<(), TamsError> {
    let decoded = BASE64_STANDARD.decode(encoded)
        .map_err(|_| TamsError::Unauthorized("Invalid Base64 encoding".to_string()))?;

    let decoded_str = String::from_utf8(decoded)
        .map_err(|_| TamsError::Unauthorized("Invalid UTF-8 in Basic auth".to_string()))?;

    let parts: Vec<&str> = decoded_str.splitn(2, ':').collect();
    if parts.len() != 2 {
        return Err(TamsError::Unauthorized("Invalid Basic auth format".to_string()));
    }

    let (username, password) = (parts[0], parts[1]);

    if username != config.basic_auth_username || password != config.basic_auth_password {
        return Err(TamsError::Unauthorized("Invalid credentials".to_string()));
    }

    Ok(())
}

// Helper function to create JWT tokens (for testing or admin tools)
pub fn create_jwt_token(user_id: &str, secret: &str) -> Result<String, TamsError> {
    use jsonwebtoken::{encode, EncodingKey, Header};
    
    let now = chrono::Utc::now().timestamp() as usize;
    let claims = Claims {
        sub: user_id.to_string(),
        exp: now + 3600, // 1 hour
        iat: now,
    };

    let encoding_key = EncodingKey::from_secret(secret.as_bytes());
    encode(&Header::default(), &claims, &encoding_key)
        .map_err(|e| TamsError::Internal(format!("Failed to create JWT token: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jwt_token_creation_and_validation() {
        let secret = "test-secret-key-must-be-256-bits-long-for-security";
        let user_id = "test-user";

        // Create token
        let token = create_jwt_token(user_id, secret).unwrap();
        assert!(!token.is_empty());

        // Validate token
        let decoding_key = DecodingKey::from_secret(secret.as_bytes());
        let claims = validate_jwt_token(&token, &decoding_key).unwrap();
        assert_eq!(claims.sub, user_id);
    }

    #[test]
    fn test_basic_auth_validation() {
        let config = AuthConfig {
            require_auth: true,
            jwt_secret: "secret".to_string(),
            basic_auth_username: "admin".to_string(),
            basic_auth_password: "password".to_string(),
        };

        // Valid credentials
        let encoded = BASE64_STANDARD.encode("admin:password");
        assert!(validate_basic_auth(&encoded, &config).is_ok());

        // Invalid credentials
        let encoded = BASE64_STANDARD.encode("admin:wrong");
        assert!(validate_basic_auth(&encoded, &config).is_err());

        // Invalid format
        let encoded = BASE64_STANDARD.encode("invalid");
        assert!(validate_basic_auth(&encoded, &config).is_err());
    }
} 