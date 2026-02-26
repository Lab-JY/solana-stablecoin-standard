use axum::{
    routing::{delete, get, post},
    Router,
};
use std::sync::Arc;

use crate::handlers;
use crate::AppState;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/blacklist", post(handlers::add_to_blacklist))
        .route("/blacklist", get(handlers::get_blacklist))
        .route(
            "/blacklist/{address}",
            delete(handlers::remove_from_blacklist),
        )
        .route("/audit-trail", get(handlers::get_audit_trail))
        .route("/health", get(handlers::health))
}
