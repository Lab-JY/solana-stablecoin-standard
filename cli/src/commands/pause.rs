use anyhow::{Context, Result};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::Signer,
    transaction::Transaction,
};
use std::str::FromStr;

use crate::config::{self, CliConfig};
use crate::output;

/// Anchor instruction discriminator for `pause`.
/// sha256("global:pause")[..8]
const PAUSE_DISCRIMINATOR: [u8; 8] = [211, 22, 221, 251, 74, 121, 193, 47];

/// Anchor instruction discriminator for `unpause`.
/// sha256("global:unpause")[..8]
const UNPAUSE_DISCRIMINATOR: [u8; 8] = [169, 144, 4, 38, 10, 141, 188, 255];

pub async fn execute_pause(cfg: &CliConfig, mint_addr: &str) -> Result<()> {
    execute_pause_toggle(cfg, mint_addr, true).await
}

pub async fn execute_unpause(cfg: &CliConfig, mint_addr: &str) -> Result<()> {
    execute_pause_toggle(cfg, mint_addr, false).await
}

async fn execute_pause_toggle(cfg: &CliConfig, mint_addr: &str, pause: bool) -> Result<()> {
    let rpc = RpcClient::new(&cfg.rpc_url);
    let payer = &cfg.keypair;

    let mint_pubkey = Pubkey::from_str(mint_addr).context("Parsing mint address")?;

    let (stablecoin_config, _) =
        config::stablecoin_config_pda(&mint_pubkey, &cfg.sss_token_program_id);
    let (role_config, _) = config::role_config_pda(&stablecoin_config, &cfg.sss_token_program_id);

    let discriminator = if pause {
        PAUSE_DISCRIMINATOR
    } else {
        UNPAUSE_DISCRIMINATOR
    };

    let data = discriminator.to_vec();

    let accounts = vec![
        AccountMeta::new_readonly(payer.pubkey(), true), // pauser (signer)
        AccountMeta::new(stablecoin_config, false),      // stablecoin_config (mut)
        AccountMeta::new_readonly(role_config, false),   // role_config
    ];

    let ix = Instruction {
        program_id: cfg.sss_token_program_id,
        accounts,
        data,
    };

    let recent_blockhash = rpc.get_latest_blockhash().context("Fetching blockhash")?;

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&payer.pubkey()),
        &[payer],
        recent_blockhash,
    );

    let signature = rpc.send_and_confirm_transaction(&tx).context(if pause {
        "Sending pause transaction"
    } else {
        "Sending unpause transaction"
    })?;

    let action = if pause { "Paused" } else { "Unpaused" };

    output::header(&format!("Stablecoin {}", action));
    output::field("Mint", mint_addr);
    output::field("Action", action);
    output::tx_signature(&signature.to_string());
    output::success(&format!(
        "Stablecoin {} successfully",
        action.to_lowercase()
    ));

    Ok(())
}
