use axum::{
    extract::{Path, Query, State},
    http::{header, StatusCode},
    response::IntoResponse,
    Json,
};
use sss_shared::{
    error::AppError,
    types::{AuditEntry, AuditTrailQuery, BlacklistEntry, BlacklistRequest, HealthResponse},
};
use std::sync::Arc;

use crate::AppState;

pub async fn add_to_blacklist(
    State(state): State<Arc<AppState>>,
    Json(req): Json<BlacklistRequest>,
) -> Result<(StatusCode, Json<BlacklistEntry>), AppError> {
    tracing::info!(address = %req.address, "adding address to blacklist");

    // Validate address format
    solana_sdk::pubkey::Pubkey::try_from(req.address.as_str())
        .map_err(|e| AppError::BadRequest(format!("Invalid Solana address: {e}")))?;

    // Check if already blacklisted
    let existing: Option<(i64,)> = sqlx::query_as("SELECT id FROM blacklist WHERE address = ?")
        .bind(&req.address)
        .fetch_optional(&state.db.pool)
        .await?;

    if existing.is_some() {
        return Err(AppError::BadRequest(format!(
            "Address {} is already blacklisted",
            req.address
        )));
    }

    let actor = state.solana.payer_pubkey().to_string();

    sqlx::query("INSERT INTO blacklist (address, reason, added_by) VALUES (?, ?, ?)")
        .bind(&req.address)
        .bind(&req.reason)
        .bind(&actor)
        .execute(&state.db.pool)
        .await?;

    // Log to audit trail
    sqlx::query("INSERT INTO audit_log (action, actor, target, details) VALUES (?, ?, ?, ?)")
        .bind("blacklist_add")
        .bind(&actor)
        .bind(&req.address)
        .bind(req.reason.as_deref().unwrap_or("No reason provided"))
        .execute(&state.db.pool)
        .await?;

    let entry: (i64, String, Option<String>, String, String) = sqlx::query_as(
        "SELECT id, address, reason, added_by, created_at FROM blacklist WHERE address = ?",
    )
    .bind(&req.address)
    .fetch_one(&state.db.pool)
    .await?;

    tracing::info!(address = %req.address, "address added to blacklist");

    Ok((
        StatusCode::CREATED,
        Json(BlacklistEntry {
            id: entry.0,
            address: entry.1,
            reason: entry.2,
            added_by: entry.3,
            created_at: entry.4,
        }),
    ))
}

pub async fn remove_from_blacklist(
    State(state): State<Arc<AppState>>,
    Path(address): Path<String>,
) -> Result<StatusCode, AppError> {
    tracing::info!(address = %address, "removing address from blacklist");

    let result = sqlx::query("DELETE FROM blacklist WHERE address = ?")
        .bind(&address)
        .execute(&state.db.pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(format!(
            "Address {address} not found in blacklist"
        )));
    }

    // Log to audit trail
    let actor = state.solana.payer_pubkey().to_string();
    sqlx::query("INSERT INTO audit_log (action, actor, target) VALUES (?, ?, ?)")
        .bind("blacklist_remove")
        .bind(&actor)
        .bind(&address)
        .execute(&state.db.pool)
        .await?;

    tracing::info!(address = %address, "address removed from blacklist");

    Ok(StatusCode::NO_CONTENT)
}

pub async fn get_blacklist(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<BlacklistEntry>>, AppError> {
    let rows: Vec<(i64, String, Option<String>, String, String)> = sqlx::query_as(
        "SELECT id, address, reason, added_by, created_at FROM blacklist ORDER BY created_at DESC",
    )
    .fetch_all(&state.db.pool)
    .await?;

    let entries: Vec<BlacklistEntry> = rows
        .into_iter()
        .map(|r| BlacklistEntry {
            id: r.0,
            address: r.1,
            reason: r.2,
            added_by: r.3,
            created_at: r.4,
        })
        .collect();

    Ok(Json(entries))
}

pub async fn get_audit_trail(
    State(state): State<Arc<AppState>>,
    Query(query): Query<AuditTrailQuery>,
) -> Result<impl IntoResponse, AppError> {
    let limit = query.limit.unwrap_or(100).min(1000);
    let offset = query.offset.unwrap_or(0);

    let mut sql = String::from(
        "SELECT id, action, actor, target, details, signature, created_at FROM audit_log WHERE 1=1",
    );
    let mut bindings: Vec<String> = Vec::new();

    if let Some(ref action) = query.action {
        sql.push_str(" AND action = ?");
        bindings.push(action.clone());
    }
    if let Some(ref actor) = query.actor {
        sql.push_str(" AND actor = ?");
        bindings.push(actor.clone());
    }
    if let Some(ref from) = query.from {
        sql.push_str(" AND created_at >= ?");
        bindings.push(from.clone());
    }
    if let Some(ref to) = query.to {
        sql.push_str(" AND created_at <= ?");
        bindings.push(to.clone());
    }

    sql.push_str(" ORDER BY created_at DESC LIMIT ? OFFSET ?");

    let mut q = sqlx::query_as::<
        _,
        (
            i64,
            String,
            String,
            Option<String>,
            Option<String>,
            Option<String>,
            String,
        ),
    >(&sql);
    for b in &bindings {
        q = q.bind(b);
    }
    q = q.bind(limit).bind(offset);

    let rows = q.fetch_all(&state.db.pool).await?;

    let entries: Vec<AuditEntry> = rows
        .into_iter()
        .map(|r| AuditEntry {
            id: r.0,
            action: r.1,
            actor: r.2,
            target: r.3,
            details: r.4,
            signature: r.5,
            created_at: r.6,
        })
        .collect();

    let format = query.format.as_deref().unwrap_or("json");

    match format {
        "csv" => {
            let mut wtr = csv::Writer::from_writer(Vec::new());
            // Header
            wtr.write_record([
                "id",
                "action",
                "actor",
                "target",
                "details",
                "signature",
                "created_at",
            ])
            .map_err(|e| AppError::Internal(format!("CSV write error: {e}")))?;
            for entry in &entries {
                wtr.write_record([
                    &entry.id.to_string(),
                    &entry.action,
                    &entry.actor,
                    entry.target.as_deref().unwrap_or(""),
                    entry.details.as_deref().unwrap_or(""),
                    entry.signature.as_deref().unwrap_or(""),
                    &entry.created_at,
                ])
                .map_err(|e| AppError::Internal(format!("CSV write error: {e}")))?;
            }
            let csv_data = String::from_utf8(
                wtr.into_inner()
                    .map_err(|e| AppError::Internal(format!("CSV flush error: {e}")))?,
            )
            .map_err(|e| AppError::Internal(format!("CSV encoding error: {e}")))?;

            Ok((
                [
                    (header::CONTENT_TYPE, "text/csv"),
                    (
                        header::CONTENT_DISPOSITION,
                        "attachment; filename=\"audit-trail.csv\"",
                    ),
                ],
                csv_data,
            )
                .into_response())
        }
        _ => Ok(Json(entries).into_response()),
    }
}

pub async fn health(State(state): State<Arc<AppState>>) -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        service: "compliance".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_seconds: state.start_time.elapsed().as_secs(),
    })
}
