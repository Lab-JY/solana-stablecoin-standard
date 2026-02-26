use sss_shared::Database;
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

    let db = Database::new(&database_url).await?;

    let state = Arc::new(AppState {
        db,
        program_id,
        rpc_url,
        ws_url,
        webhook_url,
    });

    tracing::info!("indexer service starting, subscribing to program logs");

    // Run the WebSocket listener
    listener::run(state).await?;

    Ok(())
}
