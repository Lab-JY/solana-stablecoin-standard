use crate::parser;
use crate::AppState;
use anyhow::{Context, Result};
use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use std::sync::Arc;
use tokio_tungstenite::{connect_async, tungstenite::Message};

pub async fn run(state: Arc<AppState>) -> Result<()> {
    loop {
        match subscribe_and_process(state.clone()).await {
            Ok(()) => {
                tracing::warn!("WebSocket connection closed, reconnecting in 5s...");
            }
            Err(e) => {
                tracing::error!(error = %e, "WebSocket error, reconnecting in 5s...");
            }
        }
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    }
}

async fn subscribe_and_process(state: Arc<AppState>) -> Result<()> {
    let (mut ws_stream, _) = connect_async(&state.ws_url)
        .await
        .context("Failed to connect to WebSocket")?;

    tracing::info!(ws_url = %state.ws_url, "connected to WebSocket");

    // Subscribe to program logs
    let subscribe_msg = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "logsSubscribe",
        "params": [
            {
                "mentions": [state.program_id]
            },
            {
                "commitment": "confirmed"
            }
        ]
    });

    ws_stream
        .send(Message::Text(subscribe_msg.to_string()))
        .await
        .context("Failed to send subscribe message")?;

    tracing::info!(program_id = %state.program_id, "subscribed to program logs");

    while let Some(msg) = ws_stream.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                if let Err(e) = handle_message(&state, &text).await {
                    tracing::error!(error = %e, "failed to handle message");
                }
            }
            Ok(Message::Ping(data)) => {
                ws_stream.send(Message::Pong(data)).await.ok();
            }
            Ok(Message::Close(_)) => {
                tracing::warn!("WebSocket closed by server");
                break;
            }
            Err(e) => {
                tracing::error!(error = %e, "WebSocket receive error");
                break;
            }
            _ => {}
        }
    }

    Ok(())
}

async fn handle_message(state: &Arc<AppState>, text: &str) -> Result<()> {
    let msg: serde_json::Value = serde_json::from_str(text)?;

    // Check if it's a notification (not subscription confirmation)
    if msg.get("method").and_then(|m| m.as_str()) != Some("logsNotification") {
        // Could be subscription confirmation
        if let Some(result) = msg.get("result") {
            tracing::info!(subscription_id = %result, "subscription confirmed");
        }
        return Ok(());
    }

    let params = msg
        .get("params")
        .and_then(|p| p.get("result"))
        .and_then(|r| r.get("value"))
        .context("Missing value in notification")?;

    let signature = params
        .get("signature")
        .and_then(|s| s.as_str())
        .unwrap_or("unknown");

    let slot = msg
        .get("params")
        .and_then(|p| p.get("result"))
        .and_then(|r| r.get("context"))
        .and_then(|c| c.get("slot"))
        .and_then(|s| s.as_i64())
        .unwrap_or(0);

    let logs = params
        .get("logs")
        .and_then(|l| l.as_array())
        .cloned()
        .unwrap_or_default();

    let log_strings: Vec<String> = logs
        .iter()
        .filter_map(|l| l.as_str().map(|s| s.to_string()))
        .collect();

    // Check for errors in the transaction
    let err = params.get("err");
    if err.is_some() && !err.unwrap().is_null() {
        tracing::debug!(signature = %signature, "skipping failed transaction");
        return Ok(());
    }

    // Parse events from logs
    let events = parser::parse_events(&log_strings, &state.program_id);

    for event in events {
        tracing::info!(
            event_type = %event.event_type,
            signature = %signature,
            "indexed event"
        );

        // Store event in database
        let data_json = serde_json::to_string(&event.data)?;

        let result = sqlx::query(
            "INSERT OR IGNORE INTO events (event_type, signature, slot, program_id, data) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(&event.event_type)
        .bind(signature)
        .bind(slot)
        .bind(&state.program_id)
        .bind(&data_json)
        .execute(&state.db.pool)
        .await;

        match result {
            Ok(r) => {
                if r.rows_affected() > 0 {
                    // Forward to webhook service if configured
                    if let Some(ref webhook_url) = state.webhook_url {
                        forward_to_webhook(webhook_url, signature, &event.event_type, &event.data)
                            .await;
                    }
                }
            }
            Err(e) => {
                tracing::error!(error = %e, "failed to store event");
            }
        }
    }

    Ok(())
}

async fn forward_to_webhook(
    webhook_url: &str,
    signature: &str,
    event_type: &str,
    data: &serde_json::Value,
) {
    let client = reqwest::Client::new();
    let payload = json!({
        "signature": signature,
        "event_type": event_type,
        "data": data,
    });

    match client
        .post(format!("{webhook_url}/events"))
        .json(&payload)
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await
    {
        Ok(resp) => {
            tracing::debug!(status = %resp.status(), "forwarded event to webhook service");
        }
        Err(e) => {
            tracing::warn!(error = %e, "failed to forward event to webhook service");
        }
    }
}
