use axum::Router;
use sss_shared::Database;
use std::sync::Arc;
use std::time::Instant;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

mod delivery;
mod handlers;
mod routes;

pub struct AppState {
    pub db: Database,
    pub start_time: Instant,
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

    let db = Database::new(&database_url).await?;

    let state = Arc::new(AppState {
        db,
        start_time: Instant::now(),
    });

    // Spawn background delivery worker
    let delivery_state = state.clone();
    tokio::spawn(async move {
        delivery::run_delivery_worker(delivery_state, poll_interval).await;
    });

    let app = Router::new()
        .merge(routes::routes())
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr = format!("0.0.0.0:{port}");
    tracing::info!("webhook service listening on {addr}");

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
