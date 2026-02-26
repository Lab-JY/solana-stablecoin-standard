use anchor_lang::prelude::*;

#[error_code]
pub enum OracleError {
    #[msg("Oracle is disabled")]
    OracleDisabled,
    #[msg("Invalid price feed data")]
    InvalidFeedData,
    #[msg("Invalid price: must be greater than zero")]
    InvalidPrice,
    #[msg("Price data is stale")]
    StalePriceData,
    #[msg("Slippage tolerance exceeded")]
    SlippageExceeded,
    #[msg("Math overflow")]
    MathOverflow,
    #[msg("Unauthorized")]
    Unauthorized,
    #[msg("Invalid feed account")]
    InvalidFeed,
    #[msg("Currency symbol too long (max 10 chars)")]
    CurrencyTooLong,
}
