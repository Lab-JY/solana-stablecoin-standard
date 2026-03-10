use crate::constants::*;
use anchor_lang::prelude::*;

#[account]
pub struct StablecoinConfig {
    /// Master authority
    pub authority: Pubkey,
    /// Token mint address
    pub mint: Pubkey,
    /// Token name
    pub name: String,
    /// Token symbol
    pub symbol: String,
    /// Metadata URI
    pub uri: String,
    /// Token decimals
    pub decimals: u8,
    /// Global pause flag
    pub paused: bool,
    /// Cumulative tokens minted
    pub total_minted: u64,
    /// Cumulative tokens burned
    pub total_burned: u64,
    /// SSS-2: permanent delegate enabled
    pub enable_permanent_delegate: bool,
    /// SSS-2: transfer hook enabled
    pub enable_transfer_hook: bool,
    /// SSS-2: new accounts start frozen
    pub default_account_frozen: bool,
    /// SSS-2: transfer hook program ID
    pub transfer_hook_program: Option<Pubkey>,
    /// Pending authority for two-step transfer (uses reserved space)
    pub pending_authority: Option<Pubkey>,
    /// PDA bump
    pub bump: u8,
    /// Reserved for future use
    pub _reserved: [u8; 31],
}

impl StablecoinConfig {
    pub const LEN: usize = 8  // discriminator
        + 32  // authority
        + 32  // mint
        + (4 + MAX_NAME_LEN)    // name (String = 4 byte len + data)
        + (4 + MAX_SYMBOL_LEN)  // symbol
        + (4 + MAX_URI_LEN)     // uri
        + 1   // decimals
        + 1   // paused
        + 8   // total_minted
        + 8   // total_burned
        + 1   // enable_permanent_delegate
        + 1   // enable_transfer_hook
        + 1   // default_account_frozen
        + (1 + 32) // transfer_hook_program (Option<Pubkey>)
        + (1 + 32) // pending_authority (Option<Pubkey>)
        + 1   // bump
        + 31; // _reserved

    pub fn is_compliance_enabled(&self) -> bool {
        self.enable_permanent_delegate && self.enable_transfer_hook
    }
}

#[account]
pub struct RoleConfig {
    /// Parent stablecoin config
    pub stablecoin: Pubkey,
    /// Master authority (same as StablecoinConfig.authority)
    pub master_authority: Pubkey,
    /// Pauser role
    pub pauser: Pubkey,
    /// Minters with quotas
    pub minters: Vec<MinterInfo>,
    /// Burner addresses
    pub burners: Vec<Pubkey>,
    /// SSS-2: blacklister role
    pub blacklister: Pubkey,
    /// SSS-2: seizer role
    pub seizer: Pubkey,
    /// PDA bump
    pub bump: u8,
    /// Reserved for future use
    pub _reserved: [u8; 64],
}

impl RoleConfig {
    pub const LEN: usize = 8  // discriminator
        + 32  // stablecoin
        + 32  // master_authority
        + 32  // pauser
        + (4 + MAX_MINTERS * MinterInfo::LEN) // minters vec
        + (4 + MAX_BURNERS * 32)               // burners vec
        + 32  // blacklister
        + 32  // seizer
        + 1   // bump
        + 64; // _reserved
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct MinterInfo {
    /// Minter public key
    pub address: Pubkey,
    /// Maximum tokens this minter can mint
    pub quota: u64,
    /// Tokens already minted by this minter
    pub minted: u64,
}

impl MinterInfo {
    pub const LEN: usize = 32 + 8 + 8;

    pub fn remaining_quota(&self) -> u64 {
        self.quota.saturating_sub(self.minted)
    }
}

#[account]
pub struct BlacklistEntry {
    /// Parent stablecoin config
    pub stablecoin: Pubkey,
    /// Blacklisted address
    pub address: Pubkey,
    /// Reason for blacklisting
    pub reason: String,
    /// Timestamp when added
    pub added_at: i64,
    /// Who added this entry
    pub added_by: Pubkey,
    /// PDA bump
    pub bump: u8,
}

impl BlacklistEntry {
    pub const LEN: usize = 8  // discriminator
        + 32  // stablecoin
        + 32  // address
        + (4 + MAX_REASON_LEN) // reason
        + 8   // added_at
        + 32  // added_by
        + 1; // bump
}
