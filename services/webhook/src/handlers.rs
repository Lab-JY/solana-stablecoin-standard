use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use sss_shared::{
    error::AppError,
    types::{HealthResponse, PaginationParams, WebhookEntry, WebhookRegistration},
};
use std::sync::Arc;

use crate::AppState;

fn is_internal_url(url_str: &str) -> bool {
    // Extract host from URL without the `url` crate.
    // URL format: scheme://[userinfo@]host[:port]/path
    let after_scheme = url_str
        .find("://")
        .map(|i| &url_str[i + 3..])
        .unwrap_or(url_str);
    let after_userinfo = after_scheme
        .find('@')
        .map(|i| &after_scheme[i + 1..])
        .unwrap_or(after_scheme);
    // Strip path and port
    let host_port = after_userinfo.split('/').next().unwrap_or(after_userinfo);
    let host = host_port.rsplit_once(':').map_or(host_port, |(h, _)| h);
    let lower = host.to_lowercase();

    lower == "localhost"
        || lower == "127.0.0.1"
        || lower == "0.0.0.0"
        || lower.starts_with("10.")
        || lower.starts_with("172.16.")
        || lower.starts_with("192.168.")
        || lower.starts_with("169.254.")
        || lower == "::1"
        || lower == "[::1]"
}

pub async fn register_webhook(
    State(state): State<Arc<AppState>>,
    Json(req): Json<WebhookRegistration>,
) -> Result<(StatusCode, Json<WebhookEntry>), AppError> {
    tracing::info!(url = %req.url, "registering webhook");

    if req.url.is_empty() {
        return Err(AppError::BadRequest("URL is required".into()));
    }

    if req.event_types.is_empty() {
        return Err(AppError::BadRequest(
            "At least one event type is required".into(),
        ));
    }

    // Validate URL format
    if !req.url.starts_with("http://") && !req.url.starts_with("https://") {
        return Err(AppError::BadRequest(
            "URL must start with http:// or https://".into(),
        ));
    }

    // Q-09: SSRF protection - reject internal/private IP addresses
    if is_internal_url(&req.url) {
        return Err(AppError::BadRequest(
            "Webhook URL must not point to internal addresses".into(),
        ));
    }

    let event_types_json = serde_json::to_string(&req.event_types)
        .map_err(|e| AppError::Internal(format!("Failed to serialize event types: {e}")))?;

    sqlx::query("INSERT INTO webhooks (url, event_types, secret) VALUES (?, ?, ?)")
        .bind(&req.url)
        .bind(&event_types_json)
        .bind(&req.secret)
        .execute(&state.db.pool)
        .await?;

    let row: (i64, String, String, Option<String>, bool, String) = sqlx::query_as(
        "SELECT id, url, event_types, secret, active, created_at FROM webhooks WHERE url = ? ORDER BY id DESC LIMIT 1",
    )
    .bind(&req.url)
    .fetch_one(&state.db.pool)
    .await?;

    let event_types: Vec<String> = serde_json::from_str(&row.2).unwrap_or_default();

    tracing::info!(webhook_id = row.0, "webhook registered");

    Ok((
        StatusCode::CREATED,
        Json(WebhookEntry {
            id: row.0,
            url: row.1,
            event_types,
            secret: None,
            active: row.4,
            created_at: row.5,
        }),
    ))
}

pub async fn list_webhooks(
    State(state): State<Arc<AppState>>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<Vec<WebhookEntry>>, AppError> {
    let limit = params.limit.unwrap_or(100).min(1000) as i64;
    let offset = params.offset.unwrap_or(0) as i64;

    let rows: Vec<(i64, String, String, Option<String>, bool, String)> = sqlx::query_as(
        "SELECT id, url, event_types, secret, active, created_at FROM webhooks ORDER BY created_at DESC LIMIT ? OFFSET ?",
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(&state.db.pool)
    .await?;

    let entries: Vec<WebhookEntry> = rows
        .into_iter()
        .map(|r| {
            let event_types: Vec<String> = serde_json::from_str(&r.2).unwrap_or_default();
            WebhookEntry {
                id: r.0,
                url: r.1,
                event_types,
                secret: None,
                active: r.4,
                created_at: r.5,
            }
        })
        .collect();

    Ok(Json(entries))
}

pub async fn delete_webhook(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<StatusCode, AppError> {
    tracing::info!(webhook_id = id, "deleting webhook");

    // Q-10: Atomic deletion using transaction
    let mut tx = state.db.pool.begin().await?;

    // Delete associated deliveries first
    sqlx::query("DELETE FROM webhook_deliveries WHERE webhook_id = ?")
        .bind(id)
        .execute(&mut *tx)
        .await?;

    let result = sqlx::query("DELETE FROM webhooks WHERE id = ?")
        .bind(id)
        .execute(&mut *tx)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("Webhook {id} not found")));
    }

    tx.commit().await?;

    tracing::info!(webhook_id = id, "webhook deleted");

    Ok(StatusCode::NO_CONTENT)
}

/// Receive event from indexer service and enqueue for delivery
#[derive(serde::Deserialize)]
pub struct IncomingEvent {
    pub signature: String,
    pub event_type: String,
    pub data: serde_json::Value,
}

pub async fn receive_event(
    State(state): State<Arc<AppState>>,
    Json(event): Json<IncomingEvent>,
) -> Result<StatusCode, AppError> {
    tracing::info!(event_type = %event.event_type, signature = %event.signature, "received event from indexer");

    // Store event, using ON CONFLICT + RETURNING to atomically get the ID (Q-11)
    let data_json = serde_json::to_string(&event.data)
        .map_err(|e| AppError::Internal(format!("Failed to serialize event data: {e}")))?;

    let (event_id,): (i64,) = sqlx::query_as(
        "INSERT INTO events (event_type, signature, slot, program_id, data) VALUES (?, ?, 0, '', ?) \
         ON CONFLICT(signature) DO UPDATE SET event_type = event_type \
         RETURNING id",
    )
    .bind(&event.event_type)
    .bind(&event.signature)
    .bind(&data_json)
    .fetch_one(&state.db.pool)
    .await?;

    // Find matching webhooks
    let webhooks: Vec<(i64, String)> =
        sqlx::query_as("SELECT id, event_types FROM webhooks WHERE active = TRUE")
            .fetch_all(&state.db.pool)
            .await?;

    for (webhook_id, event_types_json) in webhooks {
        let event_types: Vec<String> = serde_json::from_str(&event_types_json).unwrap_or_default();

        // Check if this webhook is subscribed to this event type
        let matches = event_types
            .iter()
            .any(|t| t == &event.event_type || t == "*");
        if !matches {
            continue;
        }

        // Create delivery entry
        sqlx::query(
            "INSERT INTO webhook_deliveries (webhook_id, event_id, status) VALUES (?, ?, 'pending')",
        )
        .bind(webhook_id)
        .bind(event_id)
        .execute(&state.db.pool)
        .await?;
    }

    Ok(StatusCode::ACCEPTED)
}

pub async fn health(State(state): State<Arc<AppState>>) -> Json<HealthResponse> {
    let db_connected = sqlx::query("SELECT 1")
        .execute(&state.db.pool)
        .await
        .is_ok();

    Json(HealthResponse {
        status: "ok".to_string(),
        service: "webhook".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_seconds: state.start_time.elapsed().as_secs(),
        db_connected,
        rpc_reachable: false,
    })
}
