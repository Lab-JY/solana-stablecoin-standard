use anyhow::{Context, Result};
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use solana_transaction_status::UiTransactionEncoding;
use std::str::FromStr;

use crate::config::{self, CliConfig};
use crate::output;

/// Event discriminators from on-chain program events.
/// These correspond to the Anchor event discriminator format.
const EVENT_TYPES: &[(&str, &str)] = &[
    ("initialize", "Initialized"),
    ("mint", "TokensMinted"),
    ("burn", "TokensBurned"),
    ("freeze", "AccountFrozen"),
    ("thaw", "AccountThawed"),
    ("pause", "Paused"),
    ("unpause", "Unpaused"),
    ("blacklist_add", "BlacklistAdded"),
    ("blacklist_remove", "BlacklistRemoved"),
    ("seize", "TokensSeized"),
    ("minter_add", "MinterAdded"),
    ("minter_remove", "MinterRemoved"),
    ("role_update", "RoleUpdated"),
];

pub async fn execute(cfg: &CliConfig, mint_addr: &str, action_filter: Option<&str>) -> Result<()> {
    let rpc = RpcClient::new(&cfg.rpc_url);

    let mint_pubkey = Pubkey::from_str(mint_addr).context("Parsing mint address")?;

    let (stablecoin_config, _) =
        config::stablecoin_config_pda(&mint_pubkey, &cfg.sss_token_program_id);

    output::header("Audit Log");

    if let Some(filter) = action_filter {
        output::info(&format!("Filtering by action: {}", filter));

        let valid = EVENT_TYPES.iter().any(|(k, _)| *k == filter);
        if !valid {
            output::warn(&format!("Unknown action type '{}'. Valid types:", filter));
            for (k, desc) in EVENT_TYPES {
                output::field(k, desc);
            }
            return Ok(());
        }
    }

    output::divider();

    // Fetch recent transaction signatures for the stablecoin config account
    let signatures = rpc
        .get_signatures_for_address(&stablecoin_config)
        .context("Fetching transaction signatures for stablecoin config")?;

    if signatures.is_empty() {
        output::info("No transactions found for this stablecoin");
        return Ok(());
    }

    output::table_header(&[("SLOT", 12), ("SIGNATURE", 88), ("STATUS", 10)]);
    output::divider();

    let mut displayed = 0u64;

    for sig_info in &signatures {
        let status = if sig_info.err.is_some() {
            "Failed"
        } else {
            "Success"
        };

        // If action filter is set, fetch and inspect transaction logs
        if action_filter.is_some() {
            let sig = sig_info.signature.parse().unwrap_or_default();
            if let Ok(tx) = rpc.get_transaction(&sig, UiTransactionEncoding::Json) {
                let logs: Option<Vec<String>> = tx
                    .transaction
                    .meta
                    .as_ref()
                    .and_then(|m| m.log_messages.clone().into());

                if let Some(log_messages) = logs {
                    let filter_str = action_filter.unwrap();
                    let event_name = EVENT_TYPES
                        .iter()
                        .find(|(k, _)| *k == filter_str)
                        .map(|(_, v)| *v)
                        .unwrap_or(filter_str);

                    let matches = log_messages
                        .iter()
                        .any(|log: &String| log.contains(event_name));

                    if !matches {
                        continue;
                    }
                }
            }
        }

        output::table_row(&[
            (&sig_info.slot.to_string(), 12),
            (&sig_info.signature, 88),
            (status, 10),
        ]);

        displayed += 1;
        if displayed >= 50 {
            output::info("Showing latest 50 transactions. Use filters to narrow results.");
            break;
        }
    }

    output::divider();
    output::info(&format!(
        "Displayed {} of {} total transactions",
        displayed,
        signatures.len()
    ));

    if action_filter.is_none() {
        output::info("Use --action <type> to filter. Available types:");
        for (k, desc) in EVENT_TYPES {
            output::field(&format!("  {}", k), desc);
        }
    }

    output::success("Audit log retrieved");

    Ok(())
}
