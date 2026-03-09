use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use std::sync::Arc;

#[derive(Clone)]
pub struct AuthState {
    pub api_secret: Arc<String>,
}

impl AuthState {
    pub fn from_env() -> Self {
        let secret =
            std::env::var("API_SECRET_KEY").unwrap_or_else(|_| "changeme".to_string());
        Self {
            api_secret: Arc::new(secret),
        }
    }
}

/// Bearer token authentication middleware.
///
/// Skips authentication for `GET /health` and `GET /metrics`.
/// Expects header: `Authorization: Bearer <token>`.
pub async fn auth_middleware(
    State(state): State<AuthState>,
    req: Request,
    next: Next,
) -> Response {
    let path = req.uri().path();
    let method = req.method().clone();

    // Skip auth for health and metrics endpoints
    if method == http::Method::GET && (path == "/health" || path == "/metrics") {
        return next.run(req).await;
    }

    let auth_header = req
        .headers()
        .get(http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok());

    match auth_header {
        Some(value) if value.starts_with("Bearer ") => {
            let token = &value[7..];
            if token == state.api_secret.as_str() {
                next.run(req).await
            } else {
                unauthorized("Invalid bearer token")
            }
        }
        Some(_) => unauthorized("Malformed authorization header"),
        None => unauthorized("Missing authorization header"),
    }
}

fn unauthorized(message: &str) -> Response {
    let body = json!({
        "error": message,
        "status": 401,
    });
    (StatusCode::UNAUTHORIZED, Json(body)).into_response()
}
