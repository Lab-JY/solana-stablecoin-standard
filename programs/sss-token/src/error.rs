use anchor_lang::prelude::*;

#[error_code]
pub enum StablecoinError {
    #[msg("The stablecoin is currently paused")]
    Paused,
    #[msg("Unauthorized: caller does not have the required role")]
    Unauthorized,
    #[msg("Minter quota exceeded")]
    MinterQuotaExceeded,
    #[msg("Minter not found")]
    MinterNotFound,
    #[msg("Maximum number of minters reached")]
    MaxMintersReached,
    #[msg("Maximum number of burners reached")]
    MaxBurnersReached,
    #[msg("Compliance module is not enabled for this stablecoin")]
    ComplianceNotEnabled,
    #[msg("Address is already blacklisted")]
    AlreadyBlacklisted,
    #[msg("Address is not blacklisted")]
    NotBlacklisted,
    #[msg("Name exceeds maximum length")]
    NameTooLong,
    #[msg("Symbol exceeds maximum length")]
    SymbolTooLong,
    #[msg("URI exceeds maximum length")]
    UriTooLong,
    #[msg("Reason exceeds maximum length")]
    ReasonTooLong,
    #[msg("Math overflow")]
    MathOverflow,
    #[msg("Account is frozen")]
    AccountFrozen,
    #[msg("Invalid authority")]
    InvalidAuthority,
    #[msg("Minter already exists")]
    MinterAlreadyExists,
    #[msg("Burner already exists")]
    BurnerAlreadyExists,
    #[msg("Burner not found")]
    BurnerNotFound,
}
