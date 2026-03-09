use axum::{extract::State, Json};
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    transaction::Transaction,
};
use sss_shared::{
    error::AppError,
    types::{BurnRequest, HealthResponse, MintRequest, SupplyResponse, TransactionResponse},
};
use std::str::FromStr;
use std::sync::Arc;

use crate::AppState;

/// SPL Token-2022 program ID
const TOKEN_2022_PROGRAM_ID: &str = "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb";

/// SPL Associated Token Account program ID
const ATA_PROGRAM_ID: &str = "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL";

pub async fn mint(
    State(state): State<Arc<AppState>>,
    Json(req): Json<MintRequest>,
) -> Result<Json<TransactionResponse>, AppError> {
    tracing::info!(recipient = %req.recipient, amount = req.amount, "processing mint request");

    let recipient = Pubkey::from_str(&req.recipient)
        .map_err(|e| AppError::BadRequest(format!("Invalid recipient address: {e}")))?;
    let mint = Pubkey::from_str(&req.mint)
        .map_err(|e| AppError::BadRequest(format!("Invalid mint address: {e}")))?;

    if req.amount == 0 {
        return Err(AppError::BadRequest("Amount must be greater than 0".into()));
    }

    // Check if recipient is blacklisted
    let blacklisted: Option<(i64,)> =
        sqlx::query_as("SELECT id FROM blacklist WHERE address = ?")
            .bind(&req.recipient)
            .fetch_optional(&state.db.pool)
            .await
            .map_err(|e| AppError::Database(format!("Blacklist check failed: {e}")))?;
    if blacklisted.is_some() {
        return Err(AppError::BadRequest(
            "Recipient address is blacklisted".into(),
        ));
    }

    let token_program = Pubkey::from_str(TOKEN_2022_PROGRAM_ID)
        .map_err(|e| AppError::Internal(format!("Invalid token program ID: {e}")))?;

    // Derive PDAs
    let (stablecoin_config, _) =
        Pubkey::find_program_address(&[b"stablecoin", mint.as_ref()], &state.solana.program_id);
    let (role_config, _) = Pubkey::find_program_address(
        &[b"roles", stablecoin_config.as_ref()],
        &state.solana.program_id,
    );

    // Derive associated token account for recipient
    let recipient_ata = get_associated_token_address(&recipient, &mint, &token_program)?;

    // Build mint_tokens instruction with Anchor discriminator
    let discriminator = anchor_discriminator("global", "mint_tokens");
    let mut data = discriminator.to_vec();
    data.extend_from_slice(&req.amount.to_le_bytes());

    let ix = Instruction {
        program_id: state.solana.program_id,
        accounts: vec![
            AccountMeta::new(state.solana.payer_pubkey(), true),
            AccountMeta::new(stablecoin_config, false),
            AccountMeta::new(role_config, false),
            AccountMeta::new(mint, false),
            AccountMeta::new(recipient_ata, false),
            AccountMeta::new_readonly(token_program, false),
        ],
        data,
    };

    let blockhash = state
        .solana
        .get_latest_blockhash()
        .map_err(|e| AppError::Solana(format!("Failed to get blockhash: {e}")))?;

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&state.solana.payer_pubkey()),
        &[&*state.solana.payer],
        blockhash,
    );

    let signature = state
        .solana
        .send_transaction(tx)
        .await
        .map_err(|e| AppError::Solana(format!("Transaction failed: {e}")))?;

    // Log to audit
    sqlx::query(
        "INSERT INTO audit_log (action, actor, target, details, signature) VALUES (?, ?, ?, ?, ?)",
    )
    .bind("mint")
    .bind(state.solana.payer_pubkey().to_string())
    .bind(&req.recipient)
    .bind(format!("amount: {}", req.amount))
    .bind(signature.to_string())
    .execute(&state.db.pool)
    .await
    .map_err(|e| AppError::Database(format!("Failed to log audit: {e}")))?;

    tracing::info!(%signature, "mint transaction confirmed");

    Ok(Json(TransactionResponse {
        signature: signature.to_string(),
        slot: None,
    }))
}

pub async fn burn(
    State(state): State<Arc<AppState>>,
    Json(req): Json<BurnRequest>,
) -> Result<Json<TransactionResponse>, AppError> {
    tracing::info!(amount = req.amount, "processing burn request");

    let mint = Pubkey::from_str(&req.mint)
        .map_err(|e| AppError::BadRequest(format!("Invalid mint address: {e}")))?;

    if req.amount == 0 {
        return Err(AppError::BadRequest("Amount must be greater than 0".into()));
    }

    let token_program = Pubkey::from_str(TOKEN_2022_PROGRAM_ID)
        .map_err(|e| AppError::Internal(format!("Invalid token program ID: {e}")))?;

    // Derive PDAs
    let (stablecoin_config, _) =
        Pubkey::find_program_address(&[b"stablecoin", mint.as_ref()], &state.solana.program_id);
    let (role_config, _) = Pubkey::find_program_address(
        &[b"roles", stablecoin_config.as_ref()],
        &state.solana.program_id,
    );

    let source = if let Some(src) = &req.source {
        Pubkey::from_str(src)
            .map_err(|e| AppError::BadRequest(format!("Invalid source address: {e}")))?
    } else {
        state.solana.payer_pubkey()
    };

    // Check if source address is blacklisted
    let source_str = source.to_string();
    let blacklisted: Option<(i64,)> =
        sqlx::query_as("SELECT id FROM blacklist WHERE address = ?")
            .bind(&source_str)
            .fetch_optional(&state.db.pool)
            .await
            .map_err(|e| AppError::Database(format!("Blacklist check failed: {e}")))?;
    if blacklisted.is_some() {
        return Err(AppError::BadRequest(
            "Source address is blacklisted".into(),
        ));
    }

    let source_ata = get_associated_token_address(&source, &mint, &token_program)?;

    // Build burn_tokens instruction
    let discriminator = anchor_discriminator("global", "burn_tokens");
    let mut data = discriminator.to_vec();
    data.extend_from_slice(&req.amount.to_le_bytes());

    let ix = Instruction {
        program_id: state.solana.program_id,
        accounts: vec![
            AccountMeta::new(state.solana.payer_pubkey(), true),
            AccountMeta::new(stablecoin_config, false),
            AccountMeta::new_readonly(role_config, false),
            AccountMeta::new(mint, false),
            AccountMeta::new(source_ata, false),
            AccountMeta::new_readonly(token_program, false),
        ],
        data,
    };

    let blockhash = state
        .solana
        .get_latest_blockhash()
        .map_err(|e| AppError::Solana(format!("Failed to get blockhash: {e}")))?;

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&state.solana.payer_pubkey()),
        &[&*state.solana.payer],
        blockhash,
    );

    let signature = state
        .solana
        .send_transaction(tx)
        .await
        .map_err(|e| AppError::Solana(format!("Transaction failed: {e}")))?;

    // Log to audit
    sqlx::query(
        "INSERT INTO audit_log (action, actor, target, details, signature) VALUES (?, ?, ?, ?, ?)",
    )
    .bind("burn")
    .bind(state.solana.payer_pubkey().to_string())
    .bind(req.source.as_deref().unwrap_or("self"))
    .bind(format!("amount: {}", req.amount))
    .bind(signature.to_string())
    .execute(&state.db.pool)
    .await
    .map_err(|e| AppError::Database(format!("Failed to log audit: {e}")))?;

    tracing::info!(%signature, "burn transaction confirmed");

    Ok(Json(TransactionResponse {
        signature: signature.to_string(),
        slot: None,
    }))
}

pub async fn get_supply(
    State(state): State<Arc<AppState>>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<SupplyResponse>, AppError> {
    let mint_str = params
        .get("mint")
        .ok_or_else(|| AppError::BadRequest("Missing 'mint' query parameter".into()))?;
    let mint = Pubkey::from_str(mint_str)
        .map_err(|e| AppError::BadRequest(format!("Invalid mint address: {e}")))?;

    let supply = state
        .solana
        .get_token_supply(&mint)
        .map_err(|e| AppError::Solana(format!("Failed to get supply: {e}")))?;

    Ok(Json(SupplyResponse {
        mint: mint.to_string(),
        supply,
        decimals: 6,
    }))
}

pub async fn health(State(state): State<Arc<AppState>>) -> Json<HealthResponse> {
    let db_connected = sqlx::query("SELECT 1")
        .execute(&state.db.pool)
        .await
        .is_ok();

    let rpc_reachable = state.solana.get_latest_blockhash().is_ok();

    Json(HealthResponse {
        status: "ok".to_string(),
        service: "mint-burn".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_seconds: state.start_time.elapsed().as_secs(),
        db_connected,
        rpc_reachable,
    })
}

/// Derive associated token address (ATA) from owner, mint, and token program.
/// This replicates the logic of `get_associated_token_address_with_program_id`
/// without pulling in the spl-associated-token-account crate.
fn get_associated_token_address(
    owner: &Pubkey,
    mint: &Pubkey,
    token_program: &Pubkey,
) -> Result<Pubkey, AppError> {
    let ata_program = Pubkey::from_str(ATA_PROGRAM_ID)
        .map_err(|e| AppError::Internal(format!("Invalid ATA program ID: {e}")))?;

    let (address, _) = Pubkey::find_program_address(
        &[owner.as_ref(), token_program.as_ref(), mint.as_ref()],
        &ata_program,
    );
    Ok(address)
}

/// Compute Anchor instruction discriminator: sha256("global:<name>")[..8]
fn anchor_discriminator(namespace: &str, name: &str) -> [u8; 8] {
    let preimage = format!("{namespace}:{name}");
    let hash = <sha2::Sha256 as sha2::Digest>::digest(preimage.as_bytes());
    let mut disc = [0u8; 8];
    disc.copy_from_slice(&hash[..8]);
    disc
}
