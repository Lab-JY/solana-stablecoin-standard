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
    pub http_client: reqwest::Client,
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

    tracing::info!(
        database_url = %database_url,
        rpc_url = %rpc_url,
        ws_url = %ws_url,
        program_id = %program_id,
        webhook_url = ?webhook_url,
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

    tracing::info!("indexer service starting, subscribing to program logs");

    // Run the WebSocket listener with graceful shutdown
    tokio::select! {
        result = listener::run(state) => {
            result?;
        }
        _ = shutdown_signal() => {}
    }

    Ok(())
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
