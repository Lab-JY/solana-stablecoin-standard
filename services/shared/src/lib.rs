pub mod auth;
pub mod db;
pub mod error;
pub mod metrics;
pub mod middleware;
pub mod rate_limit;
pub mod solana;
pub mod types;

pub use auth::{auth_middleware, AuthState};
pub use axum::extract::DefaultBodyLimit;
pub use db::Database;
pub use error::AppError;
pub use metrics::{metrics_handler, Metrics};
pub use middleware::{observability_middleware, security_headers_middleware};
pub use rate_limit::{rate_limit_middleware, RateLimiter};
pub use solana::SolanaClient;

/// Maximum request body size (64 KiB). Services should apply this via
/// `.layer(DefaultBodyLimit::max(MAX_BODY_SIZE))`.
pub const MAX_BODY_SIZE: usize = 65_536;

/// Build a CORS layer from the `ALLOWED_ORIGINS` env var (comma-separated, or `*`).
pub fn build_cors_layer() -> tower_http::cors::CorsLayer {
    use axum::http::{header, Method};
    use tower_http::cors::{AllowOrigin, CorsLayer};

    let origins =
        std::env::var("ALLOWED_ORIGINS").unwrap_or_else(|_| "http://localhost:3000".to_string());
    let allow_origin = if origins == "*" {
        AllowOrigin::any()
    } else {
        let list: Vec<_> = origins
            .split(',')
            .filter_map(|s| s.trim().parse().ok())
            .collect();
        AllowOrigin::list(list)
    };

    CorsLayer::new()
        .allow_origin(allow_origin)
        .allow_methods([Method::GET, Method::POST, Method::DELETE])
        .allow_headers([
            header::CONTENT_TYPE,
            header::AUTHORIZATION,
            "x-request-id".parse().expect("valid header name"),
        ])
}

/// Wait for SIGINT or SIGTERM, then log and return.
pub async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    tracing::info!("shutting down gracefully");
}
