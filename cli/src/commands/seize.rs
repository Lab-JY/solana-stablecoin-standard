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

/// Anchor instruction discriminator for `seize`.
/// sha256("global:seize")[..8]
const SEIZE_DISCRIMINATOR: [u8; 8] = [129, 159, 143, 31, 161, 224, 241, 84];

pub async fn execute(
    cfg: &CliConfig,
    mint_addr: &str,
    from_address: &str,
    to_address: &str,
) -> Result<()> {
    let rpc = RpcClient::new(&cfg.rpc_url);
    let payer = &cfg.keypair;

    let mint_pubkey = Pubkey::from_str(mint_addr).context("Parsing mint address")?;
    let from_pubkey = Pubkey::from_str(from_address).context("Parsing source address")?;
    let to_pubkey = Pubkey::from_str(to_address).context("Parsing treasury address")?;

    let (stablecoin_config, _) =
        config::stablecoin_config_pda(&mint_pubkey, &cfg.sss_token_program_id);
    let (role_config, _) = config::role_config_pda(&stablecoin_config, &cfg.sss_token_program_id);

    let token_2022_program_id = Pubkey::from_str("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb")?;

    // Derive associated token accounts
    let from_ata =
        spl_associated_token_account_address(&from_pubkey, &mint_pubkey, &token_2022_program_id);
    let to_ata =
        spl_associated_token_account_address(&to_pubkey, &mint_pubkey, &token_2022_program_id);

    // UX compatibility: seize full balance from source ATA.
    let amount = rpc
        .get_token_account_balance(&from_ata)
        .context("Fetching source token account balance")?
        .amount
        .parse::<u64>()
        .context("Parsing source token account balance")?;

    if amount == 0 {
        anyhow::bail!("Source token account has zero balance");
    }

    let mut data = SEIZE_DISCRIMINATOR.to_vec();
    data.extend_from_slice(&amount.to_le_bytes());

    let accounts = vec![
        AccountMeta::new_readonly(payer.pubkey(), true), // seizer (signer)
        AccountMeta::new_readonly(stablecoin_config, false), // stablecoin_config
        AccountMeta::new_readonly(role_config, false),   // role_config
        AccountMeta::new(mint_pubkey, false),            // mint (mut)
        AccountMeta::new(from_ata, false),               // from token account (mut)
        AccountMeta::new(to_ata, false),                 // to token account (mut)
        AccountMeta::new_readonly(token_2022_program_id, false), // token_2022_program
        AccountMeta::new_readonly(
            extra_account_meta_list_pda(&mint_pubkey, &cfg.sss_transfer_hook_program_id),
            false,
        ), // transfer hook validation PDA
        AccountMeta::new_readonly(
            config::blacklist_entry_pda(&mint_pubkey, &from_pubkey, &cfg.sss_token_program_id).0,
            false,
        ), // sender blacklist PDA
        AccountMeta::new_readonly(
            config::blacklist_entry_pda(&mint_pubkey, &to_pubkey, &cfg.sss_token_program_id).0,
            false,
        ), // receiver blacklist PDA
        AccountMeta::new_readonly(cfg.sss_token_program_id, false), // sss_token_program
        AccountMeta::new_readonly(cfg.sss_transfer_hook_program_id, false), // transfer_hook_program
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
        .context("Sending seize transaction")?;

    output::header("Tokens Seized");
    output::field("Mint", mint_addr);
    output::field("From", from_address);
    output::field("To (Treasury)", to_address);
    output::field("Amount", &amount.to_string());
    output::field("From Token Account", &from_ata.to_string());
    output::field("To Token Account", &to_ata.to_string());
    output::tx_signature(&signature.to_string());
    output::warn("Tokens have been seized via permanent delegate authority");
    output::success("Seize completed successfully");

    Ok(())
}

fn spl_associated_token_account_address(
    wallet: &Pubkey,
    mint: &Pubkey,
    token_program: &Pubkey,
) -> Pubkey {
    let ata_program_id = Pubkey::from_str("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL").unwrap();
    Pubkey::find_program_address(
        &[wallet.as_ref(), token_program.as_ref(), mint.as_ref()],
        &ata_program_id,
    )
    .0
}

fn extra_account_meta_list_pda(mint: &Pubkey, transfer_hook_program_id: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(
        &[b"extra-account-metas", mint.as_ref()],
        transfer_hook_program_id,
    )
    .0
}
