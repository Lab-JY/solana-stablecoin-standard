use anyhow::{Context, Result};
use solana_account_decoder::UiAccountEncoding;
use solana_client::rpc_client::RpcClient;
use solana_client::rpc_config::{RpcAccountInfoConfig, RpcProgramAccountsConfig};
use solana_client::rpc_filter::{Memcmp, RpcFilterType};
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

use crate::config::CliConfig;
use crate::output;

pub async fn execute(cfg: &CliConfig, mint_addr: &str, min_balance: Option<u64>) -> Result<()> {
    let rpc = RpcClient::new(&cfg.rpc_url);

    let mint_pubkey = Pubkey::from_str(mint_addr).context("Parsing mint address")?;

    let token_2022_program_id = Pubkey::from_str("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb")?;

    output::info("Fetching token accounts (this may take a moment)...");

    // Use getProgramAccounts with memcmp filter on the mint field
    // Token-2022 account layout: mint is at offset 0, 32 bytes
    let config = RpcProgramAccountsConfig {
        filters: Some(vec![RpcFilterType::Memcmp(Memcmp::new_raw_bytes(
            0,
            mint_pubkey.to_bytes().to_vec(),
        ))]),
        account_config: RpcAccountInfoConfig {
            encoding: Some(UiAccountEncoding::Base64),
            ..Default::default()
        },
        ..Default::default()
    };

    let token_accounts = rpc
        .get_program_accounts_with_config(&token_2022_program_id, config)
        .context("Fetching token accounts via getProgramAccounts")?;

    output::header("Token Holders");
    output::table_header(&[("ACCOUNT", 44), ("OWNER", 44), ("BALANCE", 18)]);
    output::divider();

    let mut count = 0u64;
    let min_bal = min_balance.unwrap_or(0);

    for (account_pubkey, account) in &token_accounts {
        let data = &account.data;
        if data.len() < 72 {
            continue;
        }

        // Token account layout: mint(32) + owner(32) + amount(8)
        let owner = Pubkey::try_from(&data[32..64]).unwrap_or_default();
        let amount = u64::from_le_bytes(data[64..72].try_into().unwrap_or([0u8; 8]));

        if amount < min_bal {
            continue;
        }

        output::table_row(&[
            (&account_pubkey.to_string(), 44),
            (&owner.to_string(), 44),
            (&amount.to_string(), 18),
        ]);
        count += 1;
    }

    output::divider();

    if let Some(bal) = min_balance {
        output::info(&format!(
            "Showing holders with balance >= {} ({} accounts)",
            bal, count
        ));
    } else {
        output::info(&format!("Total holders: {}", count));
    }

    output::success("Holders listed successfully");

    Ok(())
}
