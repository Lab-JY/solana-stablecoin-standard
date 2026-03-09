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
