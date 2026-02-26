use anyhow::{Context, Result};
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::{read_keypair_file, Keypair};
use std::str::FromStr;

/// Program IDs for the SSS on-chain programs.
pub const SSS_TOKEN_PROGRAM_ID: &str = "AhZamuppxULmpM9QGXcZJ9ZR3fvQbDd4gPsxLtDoMQmE";
pub const SSS_TRANSFER_HOOK_PROGRAM_ID: &str = "Gf5xP5YMRdhb7jRGiDsZW2guwwRMi4RQt4b5r44VPhTU";

pub const DEFAULT_RPC_URL: &str = "http://127.0.0.1:8899";
pub const DEFAULT_KEYPAIR_PATH: &str = "~/.config/solana/id.json";

/// CLI configuration loaded at startup.
pub struct CliConfig {
    pub keypair: Keypair,
    pub rpc_url: String,
    pub sss_token_program_id: Pubkey,
    pub sss_transfer_hook_program_id: Pubkey,
}

impl CliConfig {
    /// Load CLI configuration from flags or defaults.
    pub fn load(keypair_path: Option<&str>, rpc_url: Option<&str>) -> Result<Self> {
        let keypair_path = keypair_path
            .map(|s| s.to_string())
            .unwrap_or_else(|| shellexpand::tilde(DEFAULT_KEYPAIR_PATH).to_string());

        let expanded = shellexpand::tilde(&keypair_path).to_string();

        let keypair = read_keypair_file(&expanded)
            .map_err(|e| anyhow::anyhow!("Failed to read keypair from {}: {}", expanded, e))
            .context("Loading signer keypair")?;

        let rpc_url = rpc_url
            .map(|s| s.to_string())
            .unwrap_or_else(|| DEFAULT_RPC_URL.to_string());

        let sss_token_program_id =
            Pubkey::from_str(SSS_TOKEN_PROGRAM_ID).context("Parsing sss_token program ID")?;
        let sss_transfer_hook_program_id = Pubkey::from_str(SSS_TRANSFER_HOOK_PROGRAM_ID)
            .context("Parsing sss_transfer_hook program ID")?;

        Ok(CliConfig {
            keypair,
            rpc_url,
            sss_token_program_id,
            sss_transfer_hook_program_id,
        })
    }
}

/// PDA seeds used by the on-chain programs.
pub mod seeds {
    pub const STABLECOIN: &[u8] = b"stablecoin";
    pub const ROLES: &[u8] = b"roles";
    pub const BLACKLIST: &[u8] = b"blacklist";
}

/// Derive the StablecoinConfig PDA.
pub fn stablecoin_config_pda(mint: &Pubkey, program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[seeds::STABLECOIN, mint.as_ref()], program_id)
}

/// Derive the RoleConfig PDA.
pub fn role_config_pda(stablecoin: &Pubkey, program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[seeds::ROLES, stablecoin.as_ref()], program_id)
}

/// Derive a BlacklistEntry PDA.
pub fn blacklist_entry_pda(mint: &Pubkey, address: &Pubkey, program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[seeds::BLACKLIST, mint.as_ref(), address.as_ref()],
        program_id,
    )
}

/// TOML configuration for `init --custom`.
#[derive(serde::Deserialize)]
pub struct CustomInitConfig {
    pub name: String,
    pub symbol: String,
    pub uri: Option<String>,
    pub decimals: Option<u8>,
    pub enable_permanent_delegate: Option<bool>,
    pub enable_transfer_hook: Option<bool>,
    pub default_account_frozen: Option<bool>,
}

impl CustomInitConfig {
    pub fn from_file(path: &str) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Reading custom config file: {}", path))?;
        let config: CustomInitConfig =
            toml::from_str(&content).context("Parsing custom config TOML")?;
        Ok(config)
    }
}

/// A tiny helper so we can use `~` in paths without pulling in a big crate.
mod shellexpand {
    pub fn tilde(path: &str) -> std::borrow::Cow<'_, str> {
        if let Some(rest) = path.strip_prefix("~/") {
            if let Some(home) = std::env::var_os("HOME") {
                return std::borrow::Cow::Owned(format!("{}/{}", home.to_string_lossy(), rest));
            }
        }
        std::borrow::Cow::Borrowed(path)
    }
}
