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

/// Anchor instruction discriminator for `mint`.
/// sha256("global:mint_tokens")[..8]
const MINT_DISCRIMINATOR: [u8; 8] = [59, 132, 24, 246, 122, 39, 8, 243];

pub async fn execute(cfg: &CliConfig, mint_addr: &str, recipient: &str, amount: u64) -> Result<()> {
    let rpc = RpcClient::new(&cfg.rpc_url);
    let payer = &cfg.keypair;

    let mint_pubkey = Pubkey::from_str(mint_addr).context("Parsing mint address")?;
    let recipient_pubkey = Pubkey::from_str(recipient).context("Parsing recipient address")?;

    let (stablecoin_config, _) =
        config::stablecoin_config_pda(&mint_pubkey, &cfg.sss_token_program_id);
    let (role_config, _) = config::role_config_pda(&stablecoin_config, &cfg.sss_token_program_id);

    let token_2022_program_id = Pubkey::from_str("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb")?;

    // Derive the recipient's associated token account (Token-2022)
    let recipient_ata = spl_associated_token_account_address(
        &recipient_pubkey,
        &mint_pubkey,
        &token_2022_program_id,
    );

    // Build instruction data: discriminator + amount (u64 LE)
    let mut data = MINT_DISCRIMINATOR.to_vec();
    data.extend_from_slice(&amount.to_le_bytes());

    // Ensure recipient ATA exists before minting.
    let create_ata_ix =
        spl_associated_token_account::instruction::create_associated_token_account_idempotent(
            &payer.pubkey(),
            &recipient_pubkey,
            &mint_pubkey,
            &token_2022_program_id,
        );

    let accounts = vec![
        AccountMeta::new(payer.pubkey(), true), // minter (signer, mut)
        AccountMeta::new(stablecoin_config, false), // stablecoin_config (mut)
        AccountMeta::new(role_config, false),   // role_config (mut for quota tracking)
        AccountMeta::new(mint_pubkey, false),   // mint (mut)
        AccountMeta::new(recipient_ata, false), // recipient token account (mut)
        AccountMeta::new_readonly(token_2022_program_id, false), // token_2022_program
    ];

    let ix = Instruction {
        program_id: cfg.sss_token_program_id,
        accounts,
        data,
    };

    let recent_blockhash = rpc.get_latest_blockhash().context("Fetching blockhash")?;

    let tx = Transaction::new_signed_with_payer(
        &[create_ata_ix, ix],
        Some(&payer.pubkey()),
        &[payer],
        recent_blockhash,
    );

    let signature = rpc
        .send_and_confirm_transaction(&tx)
        .context("Sending mint transaction")?;

    output::header("Tokens Minted");
    output::field("Mint", mint_addr);
    output::field("Recipient", recipient);
    output::field("Amount", &amount.to_string());
    output::field("Token Account", &recipient_ata.to_string());
    output::tx_signature(&signature.to_string());
    output::success("Mint completed successfully");

    Ok(())
}

/// Derive an associated token account address for Token-2022.
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
