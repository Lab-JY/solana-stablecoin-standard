use anyhow::{Context, Result};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::Signer,
    transaction::Transaction,
};
use std::str::FromStr;

use crate::config::{self, CliConfig, CustomInitConfig};
use crate::output;

/// Anchor instruction discriminator for `initialize`.
/// sha256("global:initialize")[..8]
const INITIALIZE_DISCRIMINATOR: [u8; 8] = [175, 175, 109, 31, 13, 152, 155, 237];
/// Anchor instruction discriminator for `initialize_extra_account_meta_list`.
/// sha256("global:initialize_extra_account_meta_list")[..8]
const INIT_EXTRA_META_LIST_DISCRIMINATOR: [u8; 8] = [92, 197, 174, 197, 41, 124, 19, 3];

/// Execute the `init` command.
pub async fn execute(
    cfg: &CliConfig,
    preset: Option<&str>,
    custom_path: Option<&str>,
) -> Result<()> {
    let rpc = RpcClient::new(&cfg.rpc_url);
    let payer = &cfg.keypair;

    // Determine initialization parameters
    let (
        name,
        symbol,
        uri,
        decimals,
        enable_permanent_delegate,
        enable_transfer_hook,
        default_account_frozen,
    ) = match (preset, custom_path) {
        (Some("sss-1"), _) => {
            output::info("Initializing with SSS-1 (Minimal) preset");
            (
                "SSS-1 Stablecoin".to_string(),
                "SSS1".to_string(),
                "https://arweave.net/sss1-metadata".to_string(),
                6u8,
                false,
                false,
                false,
            )
        }
        (Some("sss-2"), _) => {
            output::info("Initializing with SSS-2 (Compliant) preset");
            (
                "SSS-2 Stablecoin".to_string(),
                "SSS2".to_string(),
                "https://arweave.net/sss2-metadata".to_string(),
                6u8,
                true,
                true,
                false,
            )
        }
        (_, Some(path)) => {
            output::info(&format!("Initializing with custom config: {}", path));
            let custom = CustomInitConfig::from_file(path)?;
            (
                custom.name,
                custom.symbol,
                custom.uri.unwrap_or_default(),
                custom.decimals.unwrap_or(6),
                custom.enable_permanent_delegate.unwrap_or(false),
                custom.enable_transfer_hook.unwrap_or(false),
                custom.default_account_frozen.unwrap_or(false),
            )
        }
        _ => {
            anyhow::bail!("Must specify --preset sss-1|sss-2 or --custom <path>");
        }
    };

    // Generate a new mint keypair
    let mint_keypair = solana_sdk::signature::Keypair::new();
    let mint_pubkey = mint_keypair.pubkey();

    // Derive PDAs
    let (stablecoin_config, _) =
        config::stablecoin_config_pda(&mint_pubkey, &cfg.sss_token_program_id);
    let (role_config, _) = config::role_config_pda(&stablecoin_config, &cfg.sss_token_program_id);

    // Build instruction data
    let mut data = INITIALIZE_DISCRIMINATOR.to_vec();

    // Serialize name (Borsh: 4-byte length + bytes)
    let name_bytes = name.as_bytes();
    data.extend_from_slice(&(name_bytes.len() as u32).to_le_bytes());
    data.extend_from_slice(name_bytes);

    // Serialize symbol
    let symbol_bytes = symbol.as_bytes();
    data.extend_from_slice(&(symbol_bytes.len() as u32).to_le_bytes());
    data.extend_from_slice(symbol_bytes);

    // Serialize uri
    let uri_bytes = uri.as_bytes();
    data.extend_from_slice(&(uri_bytes.len() as u32).to_le_bytes());
    data.extend_from_slice(uri_bytes);

    // Serialize decimals
    data.push(decimals);

    // Serialize enable_permanent_delegate
    data.push(enable_permanent_delegate as u8);

    // Serialize enable_transfer_hook
    data.push(enable_transfer_hook as u8);

    // Serialize default_account_frozen
    data.push(default_account_frozen as u8);

    let token_2022_program_id = Pubkey::from_str("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb")
        .context("Token-2022 program ID")?;

    // Build account metas matching the on-chain Initialize context
    let mut accounts = vec![
        AccountMeta::new(payer.pubkey(), true), // authority (signer, mut)
        AccountMeta::new(mint_pubkey, true),    // mint (signer, mut)
        AccountMeta::new(stablecoin_config, false), // stablecoin_config (mut, PDA)
        AccountMeta::new(role_config, false),   // role_config (mut, PDA)
    ];

    // Anchor optional accounts in the middle of an account list must always occupy a slot.
    // Passing this program ID signals `None`; passing transfer-hook ID signals `Some(...)`.
    let transfer_hook_or_none = if enable_transfer_hook {
        cfg.sss_transfer_hook_program_id
    } else {
        cfg.sss_token_program_id
    };
    accounts.push(AccountMeta::new_readonly(transfer_hook_or_none, false));

    accounts.extend([
        AccountMeta::new_readonly(token_2022_program_id, false), // token_2022_program
        AccountMeta::new_readonly(solana_sdk::system_program::id(), false), // system_program
    ]);

    let ix = Instruction {
        program_id: cfg.sss_token_program_id,
        accounts,
        data,
    };

    let recent_blockhash = rpc
        .get_latest_blockhash()
        .context("Fetching latest blockhash")?;

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&payer.pubkey()),
        &[payer, &mint_keypair],
        recent_blockhash,
    );

    let signature = rpc
        .send_and_confirm_transaction(&tx)
        .context("Sending initialize transaction")?;

    let mut extra_meta_pda: Option<Pubkey> = None;
    let mut extra_meta_sig: Option<String> = None;

    // SSS-2 requires transfer-hook extra account metadata initialization.
    if enable_transfer_hook {
        let (extra_account_meta_list, _) = Pubkey::find_program_address(
            &[b"extra-account-metas", mint_pubkey.as_ref()],
            &cfg.sss_transfer_hook_program_id,
        );

        let hook_ix = Instruction {
            program_id: cfg.sss_transfer_hook_program_id,
            accounts: vec![
                AccountMeta::new(payer.pubkey(), true),         // payer
                AccountMeta::new(extra_account_meta_list, false), // extra_account_meta_list PDA
                AccountMeta::new_readonly(mint_pubkey, false),  // mint
                AccountMeta::new_readonly(solana_sdk::system_program::id(), false), // system_program
            ],
            data: INIT_EXTRA_META_LIST_DISCRIMINATOR.to_vec(),
        };

        let hook_recent_blockhash = rpc
            .get_latest_blockhash()
            .context("Fetching blockhash for transfer hook metadata init")?;

        let hook_tx = Transaction::new_signed_with_payer(
            &[hook_ix],
            Some(&payer.pubkey()),
            &[payer],
            hook_recent_blockhash,
        );

        let hook_signature = rpc
            .send_and_confirm_transaction(&hook_tx)
            .context("Sending transfer hook metadata init transaction")?;

        extra_meta_pda = Some(extra_account_meta_list);
        extra_meta_sig = Some(hook_signature.to_string());
    }

    output::header("Stablecoin Initialized");
    output::field("Mint", &mint_pubkey.to_string());
    output::field("Config PDA", &stablecoin_config.to_string());
    output::field("Role PDA", &role_config.to_string());
    output::field("Name", &name);
    output::field("Symbol", &symbol);
    output::field("Decimals", &decimals.to_string());
    output::field("Permanent Delegate", &enable_permanent_delegate.to_string());
    output::field("Transfer Hook", &enable_transfer_hook.to_string());
    output::field("Default Frozen", &default_account_frozen.to_string());
    if let Some(extra_meta_list) = extra_meta_pda {
        output::field("Transfer Hook Meta PDA", &extra_meta_list.to_string());
    }
    if let Some(sig) = extra_meta_sig {
        output::field("Transfer Hook Meta Init Tx", &sig);
    }
    output::tx_signature(&signature.to_string());
    output::success("Stablecoin initialized successfully");

    Ok(())
}
