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

/// Anchor instruction discriminator for `freeze_account`.
/// sha256("global:freeze_account")[..8]
const FREEZE_DISCRIMINATOR: [u8; 8] = [253, 75, 82, 133, 167, 238, 43, 130];

pub async fn execute(cfg: &CliConfig, mint_addr: &str, address: &str) -> Result<()> {
    let rpc = RpcClient::new(&cfg.rpc_url);
    let payer = &cfg.keypair;

    let mint_pubkey = Pubkey::from_str(mint_addr).context("Parsing mint address")?;
    let target_pubkey = Pubkey::from_str(address).context("Parsing target address")?;

    let (stablecoin_config, _) =
        config::stablecoin_config_pda(&mint_pubkey, &cfg.sss_token_program_id);
    let (role_config, _) = config::role_config_pda(&stablecoin_config, &cfg.sss_token_program_id);

    let token_2022_program_id = Pubkey::from_str("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb")?;

    // Derive the target's associated token account
    let target_ata =
        spl_associated_token_account_address(&target_pubkey, &mint_pubkey, &token_2022_program_id);

    let data = FREEZE_DISCRIMINATOR.to_vec();

    let accounts = vec![
        AccountMeta::new_readonly(payer.pubkey(), true), // authority (signer)
        AccountMeta::new_readonly(stablecoin_config, false), // stablecoin_config
        AccountMeta::new_readonly(role_config, false),   // role_config
        AccountMeta::new_readonly(mint_pubkey, false),   // mint
        AccountMeta::new(target_ata, false),             // token account to freeze (mut)
        AccountMeta::new_readonly(token_2022_program_id, false), // token_2022_program
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
        .context("Sending freeze transaction")?;

    output::header("Account Frozen");
    output::field("Mint", mint_addr);
    output::field("Address", address);
    output::field("Token Account", &target_ata.to_string());
    output::tx_signature(&signature.to_string());
    output::success("Account frozen successfully");

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
