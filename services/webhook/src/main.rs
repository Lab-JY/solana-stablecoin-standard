use axum::{http::Method, middleware, Router};
use sss_shared::{
    auth_middleware, metrics_handler, rate_limit_middleware, request_id_middleware, AuthState,
    Database, Metrics, RateLimiter,
};
use std::sync::Arc;
use std::time::Instant;
use tower_http::cors::{AllowOrigin, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

mod delivery;
mod handlers;
mod routes;

pub struct AppState {
    pub db: Database,
    pub start_time: Instant,
    pub metrics: Metrics,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse()?))
        .json()
        .init();

    let database_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:./data/sss.db".to_string());
    let port = std::env::var("WEBHOOK_PORT").unwrap_or_else(|_| "3004".to_string());
    let poll_interval: u64 = std::env::var("WEBHOOK_POLL_INTERVAL_SECS")
        .unwrap_or_else(|_| "5".to_string())
        .parse()
        .unwrap_or(5);

    tracing::info!(
        database_url = %database_url,
        port = %port,
        poll_interval_secs = %poll_interval,
        api_secret = "<redacted>",
        "webhook service config"
    );

    let db = Database::new(&database_url).await?;
    let metrics = Metrics::new();
    let auth_state = AuthState::from_env();
    let rate_limiter = RateLimiter::from_env();

    let state = Arc::new(AppState {
        db,
        start_time: Instant::now(),
        metrics: metrics.clone(),
    });

    // Spawn background delivery worker
    let delivery_state = state.clone();
    tokio::spawn(async move {
        delivery::run_delivery_worker(delivery_state, poll_interval).await;
    });

    let cors = build_cors_layer();

    let app = Router::new()
        .merge(routes::routes())
        .route("/metrics", axum::routing::get(metrics_handler).with_state(metrics))
        .layer(middleware::from_fn_with_state(rate_limiter, rate_limit_middleware))
        .layer(middleware::from_fn_with_state(auth_state, auth_middleware))
        .layer(middleware::from_fn(request_id_middleware))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr = format!("0.0.0.0:{port}");
    tracing::info!("webhook service listening on {addr}");

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

fn build_cors_layer() -> CorsLayer {
    let origins = std::env::var("ALLOWED_ORIGINS").unwrap_or_else(|_| "*".to_string());
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
            axum::http::header::CONTENT_TYPE,
            axum::http::header::AUTHORIZATION,
            "x-request-id".parse().expect("valid header name"),
        ])
}

async fn shutdown_signal() {
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
