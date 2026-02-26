use anyhow::{Context, Result};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::Signer,
    system_program,
    transaction::Transaction,
};
use std::str::FromStr;

use crate::config::{self, CliConfig};
use crate::output;

/// Anchor instruction discriminator for `add_to_blacklist`.
/// sha256("global:add_to_blacklist")[..8]
const ADD_BLACKLIST_DISCRIMINATOR: [u8; 8] = [90, 115, 98, 231, 173, 119, 117, 176];

/// Anchor instruction discriminator for `remove_from_blacklist`.
/// sha256("global:remove_from_blacklist")[..8]
const REMOVE_BLACKLIST_DISCRIMINATOR: [u8; 8] = [47, 105, 20, 10, 165, 168, 203, 219];

pub async fn execute_add(
    cfg: &CliConfig,
    mint_addr: &str,
    address: &str,
    reason: Option<&str>,
) -> Result<()> {
    let rpc = RpcClient::new(&cfg.rpc_url);
    let payer = &cfg.keypair;

    let mint_pubkey = Pubkey::from_str(mint_addr).context("Parsing mint address")?;
    let target_pubkey = Pubkey::from_str(address).context("Parsing target address")?;

    let (stablecoin_config, _) =
        config::stablecoin_config_pda(&mint_pubkey, &cfg.sss_token_program_id);
    let (role_config, _) = config::role_config_pda(&stablecoin_config, &cfg.sss_token_program_id);
    let (blacklist_entry, _) =
        config::blacklist_entry_pda(&mint_pubkey, &target_pubkey, &cfg.sss_token_program_id);

    let reason_str = reason.unwrap_or("No reason provided");

    // Build instruction data: discriminator + address (Pubkey) + reason string (Borsh)
    let mut data = ADD_BLACKLIST_DISCRIMINATOR.to_vec();
    data.extend_from_slice(target_pubkey.as_ref());
    let reason_bytes = reason_str.as_bytes();
    data.extend_from_slice(&(reason_bytes.len() as u32).to_le_bytes());
    data.extend_from_slice(reason_bytes);

    let accounts = vec![
        AccountMeta::new(payer.pubkey(), true), // blacklister (signer, mut - pays rent)
        AccountMeta::new_readonly(stablecoin_config, false), // stablecoin_config
        AccountMeta::new_readonly(role_config, false), // role_config
        AccountMeta::new(blacklist_entry, false), // blacklist_entry (mut, init)
        AccountMeta::new_readonly(system_program::id(), false), // system_program
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

    let signature = rpc
        .send_and_confirm_transaction(&tx)
        .context("Sending add_to_blacklist transaction")?;

    output::header("Address Blacklisted");
    output::field("Mint", mint_addr);
    output::field("Address", address);
    output::field("Reason", reason_str);
    output::field("Blacklist PDA", &blacklist_entry.to_string());
    output::tx_signature(&signature.to_string());
    output::success("Address added to blacklist");

    Ok(())
}

pub async fn execute_remove(cfg: &CliConfig, mint_addr: &str, address: &str) -> Result<()> {
    let rpc = RpcClient::new(&cfg.rpc_url);
    let payer = &cfg.keypair;

    let mint_pubkey = Pubkey::from_str(mint_addr).context("Parsing mint address")?;
    let target_pubkey = Pubkey::from_str(address).context("Parsing target address")?;

    let (stablecoin_config, _) =
        config::stablecoin_config_pda(&mint_pubkey, &cfg.sss_token_program_id);
    let (role_config, _) = config::role_config_pda(&stablecoin_config, &cfg.sss_token_program_id);
    let (blacklist_entry, _) =
        config::blacklist_entry_pda(&mint_pubkey, &target_pubkey, &cfg.sss_token_program_id);

    let mut data = REMOVE_BLACKLIST_DISCRIMINATOR.to_vec();
    data.extend_from_slice(target_pubkey.as_ref());

    let accounts = vec![
        AccountMeta::new(payer.pubkey(), true), // blacklister (signer, mut - receives rent)
        AccountMeta::new_readonly(stablecoin_config, false), // stablecoin_config
        AccountMeta::new_readonly(role_config, false), // role_config
        AccountMeta::new(blacklist_entry, false), // blacklist_entry (mut, close)
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

    let signature = rpc
        .send_and_confirm_transaction(&tx)
        .context("Sending remove_from_blacklist transaction")?;

    output::header("Address Removed from Blacklist");
    output::field("Mint", mint_addr);
    output::field("Address", address);
    output::tx_signature(&signature.to_string());
    output::success("Address removed from blacklist");

    Ok(())
}
