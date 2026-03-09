use crate::AppState;
use chrono::Utc;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use sss_shared::types::WebhookPayload;
use std::sync::Arc;

type HmacSha256 = Hmac<Sha256>;

const MAX_RETRY_ATTEMPTS: i32 = 5;

/// Background worker that polls for pending deliveries and sends them
pub async fn run_delivery_worker(
    state: Arc<AppState>,
    poll_interval_secs: u64,
    mut shutdown: tokio::sync::watch::Receiver<bool>,
) {
    tracing::info!(poll_interval_secs, "starting webhook delivery worker");

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .expect("failed to build HTTP client");

    loop {
        tokio::select! {
            _ = tokio::time::sleep(tokio::time::Duration::from_secs(poll_interval_secs)) => {
                if let Err(e) = process_pending_deliveries(&state, &client).await {
                    tracing::error!(error = %e, "delivery worker error");
                }
            }
            _ = shutdown.changed() => {
                tracing::info!("delivery worker shutting down");
                break;
            }
        }
    }
}

async fn process_pending_deliveries(
    state: &Arc<AppState>,
    client: &reqwest::Client,
) -> anyhow::Result<()> {
    let now = Utc::now().to_rfc3339();

    // Fetch deliveries with webhook and event data in a single JOIN query
    let deliveries: Vec<(i64, i64, i64, i32, String, Option<String>, String, String)> =
        sqlx::query_as(
            r#"
            SELECT d.id, d.webhook_id, d.event_id, d.attempts,
                   w.url, w.secret,
                   e.event_type, e.data
            FROM webhook_deliveries d
            JOIN webhooks w ON w.id = d.webhook_id
            JOIN events e ON e.id = d.event_id
            WHERE d.status = 'pending'
              AND (d.next_retry_at IS NULL OR d.next_retry_at <= ?)
              AND d.attempts < ?
              AND w.active = TRUE
            ORDER BY d.created_at ASC
            LIMIT 50
            "#,
        )
        .bind(&now)
        .bind(MAX_RETRY_ATTEMPTS)
        .fetch_all(&state.db.pool)
        .await?;

    if deliveries.is_empty() {
        return Ok(());
    }

    tracing::debug!(count = deliveries.len(), "processing pending deliveries");

    for (delivery_id, webhook_id, event_id, attempts, url, secret, event_type, data_str) in
        deliveries
    {
        let data: serde_json::Value = serde_json::from_str(&data_str).unwrap_or_default();

        let payload = WebhookPayload {
            event_id,
            event_type: event_type.clone(),
            data,
            timestamp: Utc::now().to_rfc3339(),
        };

        let payload_json = serde_json::to_string(&payload)?;

        // Build request with optional HMAC signature
        let mut request = client.post(&url).header("Content-Type", "application/json");

        if let Some(ref secret) = secret {
            let signature = compute_hmac(secret, &payload_json);
            request = request.header("X-Webhook-Signature", signature);
        }

        let result = request.body(payload_json).send().await;

        let new_attempts = attempts + 1;
        let now_str = Utc::now().to_rfc3339();

        match result {
            Ok(resp) => {
                let status_code = resp.status().as_u16() as i32;
                if resp.status().is_success() {
                    tracing::info!(
                        delivery_id,
                        webhook_id,
                        status_code,
                        "webhook delivered successfully"
                    );
                    sqlx::query(
                        "UPDATE webhook_deliveries SET status = 'delivered', attempts = ?, last_attempt_at = ?, response_code = ? WHERE id = ?",
                    )
                    .bind(new_attempts)
                    .bind(&now_str)
                    .bind(status_code)
                    .bind(delivery_id)
                    .execute(&state.db.pool)
                    .await?;
                } else {
                    let error_msg = format!("HTTP {status_code}");
                    handle_failure(
                        state,
                        delivery_id,
                        new_attempts,
                        &now_str,
                        Some(status_code),
                        &error_msg,
                    )
                    .await?;
                }
            }
            Err(e) => {
                let error_msg = e.to_string();
                handle_failure(state, delivery_id, new_attempts, &now_str, None, &error_msg)
                    .await?;
            }
        }
    }

    Ok(())
}

async fn handle_failure(
    state: &Arc<AppState>,
    delivery_id: i64,
    attempts: i32,
    now_str: &str,
    response_code: Option<i32>,
    error_message: &str,
) -> anyhow::Result<()> {
    if attempts >= MAX_RETRY_ATTEMPTS {
        tracing::warn!(
            delivery_id,
            attempts,
            error = %error_message,
            "webhook delivery failed permanently"
        );
        sqlx::query(
            "UPDATE webhook_deliveries SET status = 'failed', attempts = ?, last_attempt_at = ?, response_code = ?, error_message = ? WHERE id = ?",
        )
        .bind(attempts)
        .bind(now_str)
        .bind(response_code)
        .bind(error_message)
        .bind(delivery_id)
        .execute(&state.db.pool)
        .await?;
    } else {
        // Exponential backoff: 2^attempts seconds (2, 4, 8, 16, 32)
        let backoff_secs = 2i64.pow(attempts as u32);
        let next_retry = Utc::now() + chrono::Duration::seconds(backoff_secs);
        let next_retry_str = next_retry.to_rfc3339();

        tracing::warn!(
            delivery_id,
            attempts,
            next_retry = %next_retry_str,
            error = %error_message,
            "webhook delivery failed, scheduling retry"
        );

        sqlx::query(
            "UPDATE webhook_deliveries SET attempts = ?, last_attempt_at = ?, next_retry_at = ?, response_code = ?, error_message = ? WHERE id = ?",
        )
        .bind(attempts)
        .bind(now_str)
        .bind(&next_retry_str)
        .bind(response_code)
        .bind(error_message)
        .bind(delivery_id)
        .execute(&state.db.pool)
        .await?;
    }

    Ok(())
}

/// Compute HMAC-SHA256 signature for webhook payload
fn compute_hmac(secret: &str, payload: &str) -> String {
    let mut mac =
        HmacSha256::new_from_slice(secret.as_bytes()).expect("HMAC can take key of any size");
    mac.update(payload.as_bytes());
    let result = mac.finalize();
    let code_bytes = result.into_bytes();
    format!("sha256={}", hex_encode(&code_bytes))
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
