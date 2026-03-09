use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use std::sync::Arc;
use subtle::ConstantTimeEq;

#[derive(Clone)]
pub struct AuthState {
    pub api_secret: Arc<String>,
}

impl AuthState {
    pub fn from_env() -> Self {
        let secret =
            std::env::var("API_SECRET_KEY").expect("FATAL: API_SECRET_KEY must be set");
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
            let is_valid: bool = token.as_bytes().ct_eq(state.api_secret.as_bytes()).into();
            if is_valid {
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

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        middleware,
        routing::get,
        Router,
    };
    use http::Request;
    use tower::ServiceExt;

    const TEST_SECRET: &str = "test-secret-token";

    fn app() -> Router {
        let state = AuthState {
            api_secret: Arc::new(TEST_SECRET.to_string()),
        };
        Router::new()
            .route("/health", get(|| async { "ok" }))
            .route("/metrics", get(|| async { "metrics" }))
            .route("/api/data", get(|| async { "data" }))
            .route("/health", axum::routing::post(|| async { "post-health" }))
            .layer(middleware::from_fn_with_state(state.clone(), auth_middleware))
            .with_state(state)
    }

    #[tokio::test]
    async fn valid_bearer_token_accepted() {
        let resp = app()
            .oneshot(
                Request::builder()
                    .uri("/api/data")
                    .header("Authorization", format!("Bearer {TEST_SECRET}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn invalid_token_returns_401() {
        let resp = app()
            .oneshot(
                Request::builder()
                    .uri("/api/data")
                    .header("Authorization", "Bearer wrong-token")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn missing_auth_header_returns_401() {
        let resp = app()
            .oneshot(
                Request::builder()
                    .uri("/api/data")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn malformed_header_no_bearer_prefix_returns_401() {
        let resp = app()
            .oneshot(
                Request::builder()
                    .uri("/api/data")
                    .header("Authorization", "Basic abc123")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn get_health_bypasses_auth() {
        let resp = app()
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn get_metrics_bypasses_auth() {
        let resp = app()
            .oneshot(
                Request::builder()
                    .uri("/metrics")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn post_health_requires_auth() {
        let resp = app()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    // --- Security-specific tests (TC-08) ---

    #[tokio::test]
    async fn token_with_extra_whitespace_rejected() {
        let resp = app()
            .oneshot(
                Request::builder()
                    .uri("/api/data")
                    .header("Authorization", format!("Bearer  {TEST_SECRET}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        // Leading space in token means it won't match the secret
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn empty_bearer_token_rejected() {
        let resp = app()
            .oneshot(
                Request::builder()
                    .uri("/api/data")
                    .header("Authorization", "Bearer ")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn very_long_token_handled_without_panic() {
        let long_token = "x".repeat(100_000);
        let resp = app()
            .oneshot(
                Request::builder()
                    .uri("/api/data")
                    .header("Authorization", format!("Bearer {long_token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn unauthorized_body_contains_error_message() {
        use http_body_util::BodyExt;

        let resp = app()
            .oneshot(
                Request::builder()
                    .uri("/api/data")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let body_bytes = resp.into_body().collect().await.unwrap().to_bytes();
        let body: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
        assert_eq!(body["status"], 401);
        assert!(body["error"].as_str().unwrap().contains("Missing"));
    }
}
