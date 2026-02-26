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

/// Anchor instruction discriminator for `burn`.
/// sha256("global:burn_tokens")[..8]
const BURN_DISCRIMINATOR: [u8; 8] = [76, 15, 51, 254, 229, 215, 121, 66];

pub async fn execute(cfg: &CliConfig, mint_addr: &str, amount: u64) -> Result<()> {
    let rpc = RpcClient::new(&cfg.rpc_url);
    let payer = &cfg.keypair;

    let mint_pubkey = Pubkey::from_str(mint_addr).context("Parsing mint address")?;

    let (stablecoin_config, _) =
        config::stablecoin_config_pda(&mint_pubkey, &cfg.sss_token_program_id);
    let (role_config, _) = config::role_config_pda(&stablecoin_config, &cfg.sss_token_program_id);

    let token_2022_program_id = Pubkey::from_str("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb")?;

    // Derive the burner's associated token account
    let burner_ata =
        spl_associated_token_account_address(&payer.pubkey(), &mint_pubkey, &token_2022_program_id);

    let mut data = BURN_DISCRIMINATOR.to_vec();
    data.extend_from_slice(&amount.to_le_bytes());

    let accounts = vec![
        AccountMeta::new(payer.pubkey(), true), // burner (signer, mut)
        AccountMeta::new(stablecoin_config, false), // stablecoin_config (mut)
        AccountMeta::new_readonly(role_config, false), // role_config
        AccountMeta::new(mint_pubkey, false),   // mint (mut)
        AccountMeta::new(burner_ata, false),    // burner token account (mut)
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
        .context("Sending burn transaction")?;

    output::header("Tokens Burned");
    output::field("Mint", mint_addr);
    output::field("Amount", &amount.to_string());
    output::field("Burner", &payer.pubkey().to_string());
    output::tx_signature(&signature.to_string());
    output::success("Burn completed successfully");

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
