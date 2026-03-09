use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

#[derive(Debug)]
pub enum AppError {
    BadRequest(String),
    NotFound(String),
    Internal(String),
    Solana(String),
    Database(String),
    Unauthorized(String),
    RateLimited(String),
    Forbidden(String),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::BadRequest(msg) => write!(f, "Bad request: {msg}"),
            AppError::NotFound(msg) => write!(f, "Not found: {msg}"),
            AppError::Internal(msg) => write!(f, "Internal error: {msg}"),
            AppError::Solana(msg) => write!(f, "Solana error: {msg}"),
            AppError::Database(msg) => write!(f, "Database error: {msg}"),
            AppError::Unauthorized(msg) => write!(f, "Unauthorized: {msg}"),
            AppError::RateLimited(msg) => write!(f, "Rate limited: {msg}"),
            AppError::Forbidden(msg) => write!(f, "Forbidden: {msg}"),
        }
    }
}

impl std::error::Error for AppError {}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, internal_msg, client_msg) = match &self {
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.clone(), msg.clone()),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone(), msg.clone()),
            AppError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone(), "Internal server error".to_string()),
            AppError::Solana(msg) => (StatusCode::BAD_GATEWAY, msg.clone(), "Upstream service error".to_string()),
            AppError::Database(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone(), "Database error".to_string()),
            AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg.clone(), msg.clone()),
            AppError::RateLimited(msg) => (StatusCode::TOO_MANY_REQUESTS, msg.clone(), msg.clone()),
            AppError::Forbidden(msg) => (StatusCode::FORBIDDEN, msg.clone(), msg.clone()),
        };

        tracing::error!(%status, message = %internal_msg, "request error");

        let body = json!({
            "error": client_msg,
            "status": status.as_u16(),
        });

        (status, Json(body)).into_response()
    }
}

impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        AppError::Internal(err.to_string())
    }
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        AppError::Database(err.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        AppError::BadRequest(err.to_string())
    }
}
