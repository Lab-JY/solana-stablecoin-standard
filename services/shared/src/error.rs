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

#[cfg(test)]
mod tests {
    use super::*;
    use axum::response::IntoResponse;
    use http::StatusCode;

    #[test]
    fn bad_request_returns_400() {
        let err = AppError::BadRequest("invalid input".into());
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn not_found_returns_404() {
        let err = AppError::NotFound("missing resource".into());
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn internal_returns_500() {
        let err = AppError::Internal("db crashed".into());
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn solana_returns_502() {
        let err = AppError::Solana("rpc timeout".into());
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::BAD_GATEWAY);
    }

    #[test]
    fn database_returns_500() {
        let err = AppError::Database("connection lost".into());
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn unauthorized_returns_401() {
        let err = AppError::Unauthorized("bad token".into());
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn rate_limited_returns_429() {
        let err = AppError::RateLimited("slow down".into());
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::TOO_MANY_REQUESTS);
    }

    #[test]
    fn forbidden_returns_403() {
        let err = AppError::Forbidden("not allowed".into());
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }

    /// Internal/Database/Solana variants must NOT leak internal details to clients.
    #[tokio::test]
    async fn internal_variants_hide_details() {
        use http_body_util::BodyExt;

        let cases: Vec<(AppError, &str)> = vec![
            (AppError::Internal("secret db password".into()), "Internal server error"),
            (AppError::Database("connection string leaked".into()), "Database error"),
            (AppError::Solana("rpc secret".into()), "Upstream service error"),
        ];

        for (err, expected_msg) in cases {
            let resp = err.into_response();
            let body_bytes = resp.into_body().collect().await.unwrap().to_bytes();
            let body: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
            assert_eq!(body["error"].as_str().unwrap(), expected_msg);
            // Must NOT contain internal details
            assert!(!body["error"].as_str().unwrap().contains("secret"));
        }
    }

    /// BadRequest/NotFound/Unauthorized/Forbidden/RateLimited return the provided message.
    #[tokio::test]
    async fn transparent_variants_show_message() {
        use http_body_util::BodyExt;

        let cases: Vec<(AppError, &str)> = vec![
            (AppError::BadRequest("field X required".into()), "field X required"),
            (AppError::NotFound("item 42 not found".into()), "item 42 not found"),
            (AppError::Unauthorized("invalid token".into()), "invalid token"),
            (AppError::Forbidden("admin only".into()), "admin only"),
            (AppError::RateLimited("try later".into()), "try later"),
        ];

        for (err, expected_msg) in cases {
            let resp = err.into_response();
            let body_bytes = resp.into_body().collect().await.unwrap().to_bytes();
            let body: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
            assert_eq!(body["error"].as_str().unwrap(), expected_msg);
        }
    }

    #[test]
    fn display_impl_works() {
        let err = AppError::BadRequest("oops".into());
        assert_eq!(format!("{err}"), "Bad request: oops");
    }

    #[test]
    fn from_anyhow_produces_internal() {
        let err: AppError = anyhow::anyhow!("boom").into();
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }
}
