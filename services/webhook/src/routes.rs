use axum::{
    routing::{delete, get, post},
    Router,
};
use std::sync::Arc;

use crate::handlers;
use crate::AppState;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/webhooks", post(handlers::register_webhook))
        .route("/webhooks", get(handlers::list_webhooks))
        .route("/webhooks/{id}", delete(handlers::delete_webhook))
        .route("/events", post(handlers::receive_event))
        .route("/health", get(handlers::health))
}
