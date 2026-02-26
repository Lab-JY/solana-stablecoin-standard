use anyhow::{Context, Result};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{account::ReadableAccount, pubkey::Pubkey};
use std::str::FromStr;

use crate::config::{self, CliConfig};
use crate::output;

pub async fn execute_status(cfg: &CliConfig, mint_addr: &str) -> Result<()> {
    let rpc = RpcClient::new(&cfg.rpc_url);

    let mint_pubkey = Pubkey::from_str(mint_addr).context("Parsing mint address")?;

    let (stablecoin_config_pda, _) =
        config::stablecoin_config_pda(&mint_pubkey, &cfg.sss_token_program_id);
    let (role_config_pda, _) =
        config::role_config_pda(&stablecoin_config_pda, &cfg.sss_token_program_id);

    // Fetch the stablecoin config account
    let config_account = rpc
        .get_account(&stablecoin_config_pda)
        .context("Fetching stablecoin config account (is the mint initialized?)")?;

    let config_data = config_account.data();

    // Parse StablecoinConfig from account data (skip 8-byte Anchor discriminator)
    if config_data.len() < 8 {
        anyhow::bail!("Invalid stablecoin config account data");
    }

    let data = &config_data[8..];

    // Parse fields manually according to Borsh layout
    let authority = Pubkey::try_from(&data[0..32]).context("Parsing authority")?;
    let mint = Pubkey::try_from(&data[32..64]).context("Parsing mint")?;

    // String fields: 4-byte length prefix + bytes
    let mut offset = 64;

    let name = read_borsh_string(data, &mut offset)?;
    let symbol = read_borsh_string(data, &mut offset)?;
    let uri = read_borsh_string(data, &mut offset)?;

    let decimals = data[offset];
    offset += 1;

    let paused = data[offset] != 0;
    offset += 1;

    let total_minted = u64::from_le_bytes(data[offset..offset + 8].try_into()?);
    offset += 8;

    let total_burned = u64::from_le_bytes(data[offset..offset + 8].try_into()?);
    offset += 8;

    let enable_permanent_delegate = data[offset] != 0;
    offset += 1;

    let enable_transfer_hook = data[offset] != 0;
    offset += 1;

    let default_account_frozen = data[offset] != 0;

    output::header("Stablecoin Status");
    output::divider();
    output::field("Mint", &mint.to_string());
    output::field("Authority", &authority.to_string());
    output::field("Name", &name);
    output::field("Symbol", &symbol);
    output::field("URI", &uri);
    output::field("Decimals", &decimals.to_string());
    output::divider();
    output::field("Paused", &format_bool(paused));
    output::field("Total Minted", &total_minted.to_string());
    output::field("Total Burned", &total_burned.to_string());
    output::field(
        "Circulating Supply",
        &total_minted.saturating_sub(total_burned).to_string(),
    );
    output::divider();
    output::field(
        "Permanent Delegate",
        &format_bool(enable_permanent_delegate),
    );
    output::field("Transfer Hook", &format_bool(enable_transfer_hook));
    output::field("Default Frozen", &format_bool(default_account_frozen));
    output::divider();
    output::field("Config PDA", &stablecoin_config_pda.to_string());
    output::field("Role PDA", &role_config_pda.to_string());

    // Try to fetch role config for additional info
    if let Ok(role_account) = rpc.get_account(&role_config_pda) {
        let role_data = role_account.data();
        if role_data.len() > 8 + 64 {
            let rd = &role_data[8..];
            let master_authority = Pubkey::try_from(&rd[32..64]).unwrap_or_default();
            let pauser = Pubkey::try_from(&rd[64..96]).unwrap_or_default();

            output::divider();
            output::header("Role Configuration");
            output::field("Master Authority", &master_authority.to_string());
            output::field("Pauser", &pauser.to_string());
        }
    }

    output::success("Status retrieved successfully");

    Ok(())
}

pub async fn execute_supply(cfg: &CliConfig, mint_addr: &str) -> Result<()> {
    let rpc = RpcClient::new(&cfg.rpc_url);

    let mint_pubkey = Pubkey::from_str(mint_addr).context("Parsing mint address")?;

    let (stablecoin_config_pda, _) =
        config::stablecoin_config_pda(&mint_pubkey, &cfg.sss_token_program_id);

    let config_account = rpc
        .get_account(&stablecoin_config_pda)
        .context("Fetching stablecoin config account")?;

    let config_data = config_account.data();
    if config_data.len() < 8 {
        anyhow::bail!("Invalid stablecoin config account data");
    }

    let data = &config_data[8..];

    // Skip to numeric fields: authority(32) + mint(32) + 3 strings + decimals(1) + paused(1)
    let mut offset = 64;
    let _name = read_borsh_string(data, &mut offset)?;
    let _symbol = read_borsh_string(data, &mut offset)?;
    let _uri = read_borsh_string(data, &mut offset)?;

    let decimals = data[offset];
    offset += 1;

    // skip paused
    offset += 1;

    let total_minted = u64::from_le_bytes(data[offset..offset + 8].try_into()?);
    offset += 8;

    let total_burned = u64::from_le_bytes(data[offset..offset + 8].try_into()?);

    let circulating = total_minted.saturating_sub(total_burned);

    // Also fetch the on-chain mint supply for comparison
    let token_supply = rpc.get_token_supply(&mint_pubkey).ok();

    output::header("Supply Information");
    output::field("Mint", mint_addr);
    output::field("Decimals", &decimals.to_string());
    output::divider();
    output::field("Total Minted (program)", &total_minted.to_string());
    output::field("Total Burned (program)", &total_burned.to_string());
    output::field("Circulating (program)", &circulating.to_string());

    if let Some(supply) = token_supply {
        output::divider();
        output::field("On-chain Supply", &supply.amount);
        output::field("UI Amount", &format!("{}", supply.ui_amount.unwrap_or(0.0)));
    }

    output::success("Supply retrieved successfully");

    Ok(())
}

fn read_borsh_string(data: &[u8], offset: &mut usize) -> Result<String> {
    if *offset + 4 > data.len() {
        anyhow::bail!("Buffer too short for string length at offset {}", offset);
    }
    let len = u32::from_le_bytes(data[*offset..*offset + 4].try_into()?) as usize;
    *offset += 4;
    if *offset + len > data.len() {
        anyhow::bail!("Buffer too short for string data at offset {}", offset);
    }
    let s =
        String::from_utf8(data[*offset..*offset + len].to_vec()).context("Parsing string field")?;
    *offset += len;
    Ok(s)
}

fn format_bool(v: bool) -> String {
    if v {
        "Yes".to_string()
    } else {
        "No".to_string()
    }
}
