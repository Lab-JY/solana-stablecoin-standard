use anyhow::{Context, Result};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    pubkey::Pubkey,
    signature::{Keypair, Signature},
    signer::Signer,
    transaction::Transaction,
};
use std::str::FromStr;
use std::sync::Arc;
use zeroize::Zeroize;

#[derive(Clone)]
pub struct SolanaClient {
    pub rpc: Arc<RpcClient>,
    pub payer: Arc<Keypair>,
    pub program_id: Pubkey,
}

impl SolanaClient {
    pub fn new(rpc_url: &str, keypair_path: &str, program_id: &str) -> Result<Self> {
        let rpc =
            RpcClient::new_with_commitment(rpc_url.to_string(), CommitmentConfig::confirmed());

        let mut keypair_bytes = std::fs::read(keypair_path).context("Failed to read keypair file")?;
        let mut keypair_data: Vec<u8> =
            serde_json::from_slice(&keypair_bytes).context("Failed to parse keypair JSON")?;
        let payer =
            Keypair::from_bytes(&keypair_data).context("Failed to create keypair from bytes")?;
        keypair_bytes.zeroize();
        keypair_data.zeroize();

        let program_id = Pubkey::from_str(program_id).context("Failed to parse program ID")?;

        Ok(Self {
            rpc: Arc::new(rpc),
            payer: Arc::new(payer),
            program_id,
        })
    }

    pub fn payer_pubkey(&self) -> Pubkey {
        self.payer.pubkey()
    }

    pub async fn send_transaction(&self, tx: Transaction) -> Result<Signature> {
        let sig = self
            .rpc
            .send_and_confirm_transaction(&tx)
            .context("Failed to send transaction")?;
        Ok(sig)
    }

    pub fn get_token_supply(&self, mint: &Pubkey) -> Result<u64> {
        let supply = self
            .rpc
            .get_token_supply(mint)
            .context("Failed to get token supply")?;
        let amount: u64 = supply
            .amount
            .parse()
            .context("Failed to parse supply amount")?;
        Ok(amount)
    }

    pub fn get_balance(&self, pubkey: &Pubkey) -> Result<u64> {
        let balance = self
            .rpc
            .get_balance(pubkey)
            .context("Failed to get balance")?;
        Ok(balance)
    }

    pub fn get_latest_blockhash(&self) -> Result<solana_sdk::hash::Hash> {
        let blockhash = self
            .rpc
            .get_latest_blockhash()
            .context("Failed to get latest blockhash")?;
        Ok(blockhash)
    }
}
