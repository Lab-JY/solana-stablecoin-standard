use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;

use crate::handlers;
use crate::AppState;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/mint", post(handlers::mint))
        .route("/burn", post(handlers::burn))
        .route("/supply", get(handlers::get_supply))
        .route("/health", get(handlers::health))
}
