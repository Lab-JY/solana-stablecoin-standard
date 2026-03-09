use axum::{routing::get, Json, Router};
use serde::Serialize;
use sss_shared::{shutdown_signal, Database};
use std::sync::Arc;
use tracing_subscriber::EnvFilter;

mod listener;
mod parser;

pub struct AppState {
    pub db: Database,
    pub program_id: String,
    pub rpc_url: String,
    pub ws_url: String,
    pub webhook_url: Option<String>,
    pub http_client: reqwest::Client,
}

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    service: String,
}

async fn health_handler() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        service: "indexer".to_string(),
    })
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
    let rpc_url =
        std::env::var("RPC_URL").unwrap_or_else(|_| "https://api.devnet.solana.com".to_string());
    let ws_url =
        std::env::var("WS_URL").unwrap_or_else(|_| "wss://api.devnet.solana.com".to_string());
    let program_id = std::env::var("PROGRAM_ID")
        .unwrap_or_else(|_| "11111111111111111111111111111111".to_string());
    let webhook_url = std::env::var("WEBHOOK_SERVICE_URL").ok();
    let health_port =
        std::env::var("INDEXER_HEALTH_PORT").unwrap_or_else(|_| "3003".to_string());

    tracing::info!(
        database_url = %database_url,
        rpc_url = %rpc_url,
        ws_url = %ws_url,
        program_id = %program_id,
        webhook_url = ?webhook_url,
        health_port = %health_port,
        "indexer service config"
    );

    let db = Database::new(&database_url).await?;

    let http_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()?;

    let state = Arc::new(AppState {
        db,
        program_id,
        rpc_url,
        ws_url,
        webhook_url,
        http_client,
    });

    // A-04: Health endpoint for indexer
    let health_app = Router::new().route("/health", get(health_handler));
    let health_addr = format!("0.0.0.0:{health_port}");
    let health_listener = tokio::net::TcpListener::bind(&health_addr).await?;
    tracing::info!("indexer health endpoint listening on {health_addr}");

    tracing::info!("indexer service starting, subscribing to program logs");

    // Run the WebSocket listener, health server, and shutdown signal concurrently
    tokio::select! {
        result = listener::run(state) => {
            result?;
        }
        result = axum::serve(health_listener, health_app) => {
            result?;
        }
        _ = shutdown_signal() => {}
    }

    Ok(())
}
