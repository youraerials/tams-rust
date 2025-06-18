use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;
use chrono;

#[derive(Error, Debug)]
pub enum TamsError {
    #[error("Configuration error: {0}")]
    Config(#[from] config::ConfigError),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("UUID error: {0}")]
    Uuid(#[from] uuid::Error),

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("JWT error: {0}")]
    Jwt(#[from] jsonwebtoken::errors::Error),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Internal server error: {0}")]
    Internal(String),

    #[error("Media storage error: {0}")]
    MediaStorage(String),

    #[error("File too large: maximum size is {max_size} bytes")]
    FileTooLarge { max_size: u64 },

    #[error("Invalid timerange: {0}")]
    InvalidTimerange(String),

    #[error("Segment overlap: {0}")]
    SegmentOverlap(String),

    #[error("Flow is read-only: {flow_id}")]
    ReadOnlyFlow { flow_id: String },

    #[error("Object not found: {object_id}")]
    ObjectNotFound { object_id: String },

    #[error("Flow not found: {flow_id}")]
    FlowNotFound { flow_id: String },

    #[error("Source not found: {source_id}")]
    SourceNotFound { source_id: String },

    #[error("Invalid format: expected {expected}, got {actual}")]
    InvalidFormat { expected: String, actual: String },

    #[error("Missing required field: {field}")]
    MissingField { field: String },

    #[error("Invalid input: {0}")]
    InvalidInput(String),
}

impl IntoResponse for TamsError {
    fn into_response(self) -> Response {
        let (status, error_message) = match &self {
            TamsError::NotFound(_) | TamsError::FlowNotFound { .. } | 
            TamsError::SourceNotFound { .. } | TamsError::ObjectNotFound { .. } => {
                (StatusCode::NOT_FOUND, self.to_string())
            }
            TamsError::BadRequest(_) | TamsError::Validation(_) | 
            TamsError::InvalidTimerange(_) | TamsError::InvalidFormat { .. } |
            TamsError::MissingField { .. } | TamsError::Uuid(_) | TamsError::Json(_) => {
                (StatusCode::BAD_REQUEST, self.to_string())
            }
            TamsError::Unauthorized(_) => {
                (StatusCode::UNAUTHORIZED, self.to_string())
            }
            TamsError::Forbidden(_) | TamsError::ReadOnlyFlow { .. } => {
                (StatusCode::FORBIDDEN, self.to_string())
            }
            TamsError::Conflict(_) | TamsError::SegmentOverlap(_) => {
                (StatusCode::CONFLICT, self.to_string())
            }
            TamsError::FileTooLarge { .. } => {
                (StatusCode::PAYLOAD_TOO_LARGE, self.to_string())
            }
            _ => {
                tracing::error!("Internal server error: {}", self);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string())
            }
        };

        let body = Json(json!({
            "error": error_message,
            "status": status.as_u16()
        }));

        (status, body).into_response()
    }
}

pub type TamsResult<T> = Result<T, TamsError>;

// Helper function to create validation errors
pub fn validation_error(msg: impl Into<String>) -> TamsError {
    TamsError::Validation(msg.into())
}

// Helper function to create not found errors
pub fn not_found(msg: impl Into<String>) -> TamsError {
    TamsError::NotFound(msg.into())
}

// Helper function to create bad request errors
pub fn bad_request(msg: impl Into<String>) -> TamsError {
    TamsError::BadRequest(msg.into())
}

// Helper function to create internal errors
pub fn internal_error(msg: impl Into<String>) -> TamsError {
    TamsError::Internal(msg.into())
}

impl From<chrono::ParseError> for TamsError {
    fn from(err: chrono::ParseError) -> Self {
        TamsError::InvalidInput(format!("Date parsing error: {}", err))
    }
} 