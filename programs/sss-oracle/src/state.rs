use anchor_lang::prelude::*;

#[account]
pub struct OracleConfig {
    /// Authority that can update oracle settings
    pub authority: Pubkey,
    /// The stablecoin mint this oracle serves
    pub stablecoin_mint: Pubkey,
    /// Switchboard aggregator feed address
    pub switchboard_feed: Pubkey,
    /// Base currency symbol (e.g., "EUR", "BRL")
    pub base_currency: String,
    /// Quote currency symbol (e.g., "USD")
    pub quote_currency: String,
    /// Maximum allowed staleness in slots
    pub max_staleness_slots: u64,
    /// Maximum confidence interval (basis points)
    pub max_confidence_interval: u64,
    /// Last cached price (9 decimal places)
    pub last_price: u64,
    /// Timestamp of last price update
    pub last_price_timestamp: i64,
    /// Slot of last price update
    pub last_price_slot: u64,
    /// Whether the oracle is enabled
    pub enabled: bool,
    /// PDA bump
    pub bump: u8,
}

impl OracleConfig {
    pub const MAX_CURRENCY_LEN: usize = 10;

    pub const LEN: usize = 8  // discriminator
        + 32  // authority
        + 32  // stablecoin_mint
        + 32  // switchboard_feed
        + (4 + Self::MAX_CURRENCY_LEN) // base_currency
        + (4 + Self::MAX_CURRENCY_LEN) // quote_currency
        + 8   // max_staleness_slots
        + 8   // max_confidence_interval
        + 8   // last_price
        + 8   // last_price_timestamp
        + 8   // last_price_slot
        + 1   // enabled
        + 1; // bump
}
