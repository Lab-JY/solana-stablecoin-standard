use axum::{middleware, Extension, Router};
use sss_shared::{
    auth_middleware, build_cors_layer, metrics_handler, rate_limit_middleware,
    observability_middleware, security_headers_middleware, shutdown_signal, AuthState, Database,
    Metrics, RateLimiter,
};
use std::sync::Arc;
use std::time::Instant;
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

    let cleanup_limiter = rate_limiter.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(120)).await;
            cleanup_limiter.cleanup_expired();
        }
    });

    let state = Arc::new(AppState {
        db,
        start_time: Instant::now(),
        metrics: metrics.clone(),
    });

    // A-07: Create shutdown watch channel for delivery worker
    let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);

    // Spawn background delivery worker
    let delivery_state = state.clone();
    tokio::spawn(async move {
        delivery::run_delivery_worker(delivery_state, poll_interval, shutdown_rx).await;
    });

    let cors = build_cors_layer();

    let app = Router::new()
        .merge(routes::routes())
        .route("/metrics", axum::routing::get(metrics_handler).with_state(metrics.clone()))
        .layer(middleware::from_fn_with_state(auth_state, auth_middleware))
        .layer(middleware::from_fn_with_state(rate_limiter, rate_limit_middleware))
        .layer(middleware::from_fn(observability_middleware))
        .layer(Extension(metrics))
        .layer(middleware::from_fn(security_headers_middleware))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr = format!("0.0.0.0:{port}");
    tracing::info!("webhook service listening on {addr}");

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    // Signal delivery worker to stop
    let _ = shutdown_tx.send(true);

    Ok(())
}
