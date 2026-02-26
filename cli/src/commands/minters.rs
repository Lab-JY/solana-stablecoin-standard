use anyhow::{Context, Result};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    account::ReadableAccount,
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::Signer,
    transaction::Transaction,
};
use std::str::FromStr;

use crate::config::{self, CliConfig};
use crate::output;

/// Anchor instruction discriminator for `update_minter` (add/remove/update).
/// sha256("global:update_minter")[..8]
const UPDATE_MINTER_DISCRIMINATOR: [u8; 8] = [164, 129, 164, 88, 75, 29, 91, 38];

pub async fn execute_list(cfg: &CliConfig, mint_addr: &str) -> Result<()> {
    let rpc = RpcClient::new(&cfg.rpc_url);

    let mint_pubkey = Pubkey::from_str(mint_addr).context("Parsing mint address")?;

    let (stablecoin_config, _) =
        config::stablecoin_config_pda(&mint_pubkey, &cfg.sss_token_program_id);
    let (role_config_pda, _) =
        config::role_config_pda(&stablecoin_config, &cfg.sss_token_program_id);

    let role_account = rpc
        .get_account(&role_config_pda)
        .context("Fetching role config account")?;

    let role_data = role_account.data();
    if role_data.len() < 8 {
        anyhow::bail!("Invalid role config account data");
    }

    // Parse RoleConfig: skip 8-byte discriminator
    // Layout: stablecoin(32) + master_authority(32) + pauser(32) + minters vec + ...
    let data = &role_data[8..];
    let mut offset = 32 + 32 + 32; // skip stablecoin, master_authority, pauser

    // Parse minters Vec<MinterInfo>
    if offset + 4 > data.len() {
        anyhow::bail!("Buffer too short for minters vector");
    }
    let minters_len = u32::from_le_bytes(data[offset..offset + 4].try_into()?) as usize;
    offset += 4;

    output::header("Minters");
    output::table_header(&[
        ("ADDRESS", 44),
        ("QUOTA", 18),
        ("MINTED", 18),
        ("REMAINING", 18),
    ]);
    output::divider();

    if minters_len == 0 {
        output::info("No minters configured");
    }

    for _ in 0..minters_len {
        if offset + 32 + 8 + 8 > data.len() {
            break;
        }
        let address = Pubkey::try_from(&data[offset..offset + 32])?;
        offset += 32;
        let quota = u64::from_le_bytes(data[offset..offset + 8].try_into()?);
        offset += 8;
        let minted = u64::from_le_bytes(data[offset..offset + 8].try_into()?);
        offset += 8;

        let remaining = quota.saturating_sub(minted);

        output::table_row(&[
            (&address.to_string(), 44),
            (&quota.to_string(), 18),
            (&minted.to_string(), 18),
            (&remaining.to_string(), 18),
        ]);
    }

    output::success(&format!("Listed {} minter(s)", minters_len));

    Ok(())
}

pub async fn execute_add(
    cfg: &CliConfig,
    mint_addr: &str,
    minter_addr: &str,
    quota: u64,
) -> Result<()> {
    let rpc = RpcClient::new(&cfg.rpc_url);
    let payer = &cfg.keypair;

    let mint_pubkey = Pubkey::from_str(mint_addr).context("Parsing mint address")?;
    let minter_pubkey = Pubkey::from_str(minter_addr).context("Parsing minter address")?;

    let (stablecoin_config, _) =
        config::stablecoin_config_pda(&mint_pubkey, &cfg.sss_token_program_id);
    let (role_config, _) = config::role_config_pda(&stablecoin_config, &cfg.sss_token_program_id);

    // Build instruction data: discriminator + action(0=add) + minter pubkey + quota
    let mut data = UPDATE_MINTER_DISCRIMINATOR.to_vec();
    data.push(0); // action: 0 = Add
    data.extend_from_slice(minter_pubkey.as_ref());
    data.extend_from_slice(&quota.to_le_bytes());

    let accounts = vec![
        AccountMeta::new_readonly(payer.pubkey(), true), // authority (signer)
        AccountMeta::new_readonly(stablecoin_config, false), // stablecoin_config
        AccountMeta::new(role_config, false),            // role_config (mut)
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
        .context("Sending update_minter (add) transaction")?;

    output::header("Minter Added");
    output::field("Mint", mint_addr);
    output::field("Minter", minter_addr);
    output::field("Quota", &quota.to_string());
    output::tx_signature(&signature.to_string());
    output::success("Minter added successfully");

    Ok(())
}

pub async fn execute_remove(cfg: &CliConfig, mint_addr: &str, minter_addr: &str) -> Result<()> {
    let rpc = RpcClient::new(&cfg.rpc_url);
    let payer = &cfg.keypair;

    let mint_pubkey = Pubkey::from_str(mint_addr).context("Parsing mint address")?;
    let minter_pubkey = Pubkey::from_str(minter_addr).context("Parsing minter address")?;

    let (stablecoin_config, _) =
        config::stablecoin_config_pda(&mint_pubkey, &cfg.sss_token_program_id);
    let (role_config, _) = config::role_config_pda(&stablecoin_config, &cfg.sss_token_program_id);

    // Build instruction data: discriminator + action(1=remove) + minter pubkey
    let mut data = UPDATE_MINTER_DISCRIMINATOR.to_vec();
    data.push(1); // action: 1 = Remove
    data.extend_from_slice(minter_pubkey.as_ref());

    let accounts = vec![
        AccountMeta::new_readonly(payer.pubkey(), true), // authority (signer)
        AccountMeta::new_readonly(stablecoin_config, false), // stablecoin_config
        AccountMeta::new(role_config, false),            // role_config (mut)
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
        .context("Sending update_minter (remove) transaction")?;

    output::header("Minter Removed");
    output::field("Mint", mint_addr);
    output::field("Minter", minter_addr);
    output::tx_signature(&signature.to_string());
    output::success("Minter removed successfully");

    Ok(())
}
