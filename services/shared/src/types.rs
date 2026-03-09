use serde::{Deserialize, Serialize};

// --- Mint/Burn types ---

#[derive(Debug, Serialize, Deserialize)]
pub struct MintRequest {
    pub recipient: String,
    pub amount: u64,
    pub mint: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BurnRequest {
    pub amount: u64,
    pub mint: String,
    pub source: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionResponse {
    pub signature: String,
    pub slot: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SupplyResponse {
    pub mint: String,
    pub supply: u64,
    pub decimals: u8,
}

// --- Compliance types ---

#[derive(Debug, Serialize, Deserialize)]
pub struct BlacklistRequest {
    pub address: String,
    pub reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BlacklistEntry {
    pub id: i64,
    pub address: String,
    pub reason: Option<String>,
    pub added_by: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuditEntry {
    pub id: i64,
    pub action: String,
    pub actor: String,
    pub target: Option<String>,
    pub details: Option<String>,
    pub signature: Option<String>,
    pub created_at: String,
}

// --- Webhook types ---

#[derive(Debug, Serialize, Deserialize)]
pub struct WebhookRegistration {
    pub url: String,
    pub event_types: Vec<String>,
    pub secret: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WebhookEntry {
    pub id: i64,
    pub url: String,
    pub event_types: Vec<String>,
    pub secret: Option<String>,
    pub active: bool,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WebhookPayload {
    pub event_id: i64,
    pub event_type: String,
    pub data: serde_json::Value,
    pub timestamp: String,
}

// --- Event types ---

#[derive(Debug, Serialize, Deserialize)]
pub struct IndexedEvent {
    pub id: i64,
    pub event_type: String,
    pub signature: String,
    pub slot: i64,
    pub block_time: Option<i64>,
    pub program_id: String,
    pub data: serde_json::Value,
    pub processed: bool,
    pub created_at: String,
}

// --- Common types ---

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub service: String,
    pub version: String,
    pub uptime_seconds: u64,
    pub db_connected: bool,
    pub rpc_reachable: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaginationParams {
    pub offset: Option<i64>,
    pub limit: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuditTrailQuery {
    pub format: Option<String>,
    pub action: Option<String>,
    pub actor: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
    pub offset: Option<i64>,
    pub limit: Option<i64>,
}
