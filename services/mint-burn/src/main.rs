use axum::{extract::DefaultBodyLimit, http::Method, middleware, Extension, Router};
use sss_shared::{
    auth_middleware, metrics_handler, observability_middleware, rate_limit_middleware,
    security_headers_middleware, AuthState, Database, Metrics, RateLimiter, SolanaClient,
};
use std::sync::Arc;
use std::time::Instant;
use tower_http::cors::{AllowOrigin, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

mod handlers;
mod routes;

pub struct AppState {
    pub db: Database,
    pub solana: SolanaClient,
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
    let rpc_url =
        std::env::var("RPC_URL").unwrap_or_else(|_| "https://api.devnet.solana.com".to_string());
    let keypair_path =
        std::env::var("KEYPAIR_PATH").unwrap_or_else(|_| "~/.config/solana/id.json".to_string());
    let program_id = std::env::var("PROGRAM_ID")
        .unwrap_or_else(|_| "11111111111111111111111111111111".to_string());
    let port = std::env::var("MINT_BURN_PORT").unwrap_or_else(|_| "3001".to_string());

    tracing::info!(
        database_url = %database_url,
        rpc_url = %rpc_url,
        program_id = %program_id,
        port = %port,
        api_secret = "<redacted>",
        "mint-burn service config"
    );

    let db = Database::new(&database_url).await?;
    let solana = SolanaClient::new(&rpc_url, &keypair_path, &program_id)?;
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
        solana,
        start_time: Instant::now(),
        metrics: metrics.clone(),
    });

    let cors = build_cors_layer();

    let app = Router::new()
        .merge(routes::routes())
        .route("/metrics", axum::routing::get(metrics_handler).with_state(metrics.clone()))
        .layer(DefaultBodyLimit::max(65_536))
        .layer(middleware::from_fn_with_state(auth_state, auth_middleware))
        .layer(middleware::from_fn_with_state(rate_limiter, rate_limit_middleware))
        .layer(middleware::from_fn(observability_middleware))
        .layer(Extension(metrics))
        .layer(middleware::from_fn(security_headers_middleware))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr = format!("0.0.0.0:{port}");
    tracing::info!("mint-burn service listening on {addr}");

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

fn build_cors_layer() -> CorsLayer {
    let origins = std::env::var("ALLOWED_ORIGINS").unwrap_or_else(|_| "http://localhost:3000".to_string());
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
