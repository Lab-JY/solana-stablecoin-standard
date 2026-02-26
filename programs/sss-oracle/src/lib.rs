use anchor_lang::prelude::*;

declare_id!("8kRVqx5JN2rSfn2haXBqgaLnQBrXzNSYj6PH9fKRk5bN");

pub mod error;
pub mod state;

use error::*;
use state::*;

#[program]
pub mod sss_oracle {
    use super::*;

    /// Initialize the oracle configuration with a Switchboard aggregator feed.
    /// This sets up the oracle module for price-based minting operations.
    pub fn initialize_oracle(
        ctx: Context<InitializeOracle>,
        base_currency: String,
        quote_currency: String,
        max_staleness_slots: u64,
        max_confidence_interval: u64,
    ) -> Result<()> {
        require!(base_currency.len() <= 10, OracleError::CurrencyTooLong);
        require!(quote_currency.len() <= 10, OracleError::CurrencyTooLong);

        let oracle_config = &mut ctx.accounts.oracle_config;
        oracle_config.authority = ctx.accounts.authority.key();
        oracle_config.stablecoin_mint = ctx.accounts.stablecoin_mint.key();
        oracle_config.switchboard_feed = ctx.accounts.switchboard_feed.key();
        oracle_config.base_currency = base_currency;
        oracle_config.quote_currency = quote_currency;
        oracle_config.max_staleness_slots = max_staleness_slots;
        oracle_config.max_confidence_interval = max_confidence_interval;
        oracle_config.last_price = 0;
        oracle_config.last_price_timestamp = 0;
        oracle_config.last_price_slot = 0;
        oracle_config.enabled = true;
        oracle_config.bump = ctx.bumps.oracle_config;

        emit!(OracleInitialized {
            stablecoin_mint: oracle_config.stablecoin_mint,
            switchboard_feed: oracle_config.switchboard_feed,
            base_currency: oracle_config.base_currency.clone(),
            quote_currency: oracle_config.quote_currency.clone(),
        });

        Ok(())
    }

    /// Read and cache the latest price from the Switchboard aggregator.
    /// The aggregator account data is read and parsed to extract the current price.
    pub fn get_price(ctx: Context<GetPrice>) -> Result<()> {
        let oracle_config = &mut ctx.accounts.oracle_config;

        require!(oracle_config.enabled, OracleError::OracleDisabled);

        // Read Switchboard aggregator data
        let feed_data = ctx.accounts.switchboard_feed.try_borrow_data()?;

        // Parse Switchboard V2 aggregator result
        // Layout: discriminator (8) + various fields, result at offset 112
        // result: { mantissa: i128 (16 bytes), scale: u32 (4 bytes) }
        require!(feed_data.len() >= 132, OracleError::InvalidFeedData);

        let mantissa = i128::from_le_bytes(
            feed_data[112..128]
                .try_into()
                .map_err(|_| OracleError::InvalidFeedData)?,
        );
        let scale = u32::from_le_bytes(
            feed_data[128..132]
                .try_into()
                .map_err(|_| OracleError::InvalidFeedData)?,
        );

        // Convert to fixed-point with 9 decimal places
        // price = mantissa / 10^scale * 10^9
        let price_decimals: u32 = 9;
        let price = if scale <= price_decimals {
            let multiplier = 10i128.pow(price_decimals - scale);
            (mantissa * multiplier) as u64
        } else {
            let divisor = 10i128.pow(scale - price_decimals);
            (mantissa / divisor) as u64
        };

        require!(price > 0, OracleError::InvalidPrice);

        // Check staleness
        let clock = Clock::get()?;
        let current_slot = clock.slot;

        // Read the last updated slot from the aggregator (offset 216 in Switchboard V2)
        if feed_data.len() >= 224 {
            let last_updated_slot = u64::from_le_bytes(
                feed_data[216..224]
                    .try_into()
                    .map_err(|_| OracleError::InvalidFeedData)?,
            );

            let slots_since_update = current_slot.saturating_sub(last_updated_slot);
            require!(
                slots_since_update <= oracle_config.max_staleness_slots,
                OracleError::StalePriceData
            );
        }

        oracle_config.last_price = price;
        oracle_config.last_price_timestamp = clock.unix_timestamp;
        oracle_config.last_price_slot = current_slot;

        emit!(PriceUpdated {
            stablecoin_mint: oracle_config.stablecoin_mint,
            price,
            slot: current_slot,
            timestamp: clock.unix_timestamp,
        });

        Ok(())
    }

    /// Mint tokens using the oracle price for non-USD peg conversion.
    /// tokens_out = collateral_amount * 10^9 / oracle_price
    pub fn mint_with_oracle(
        ctx: Context<MintWithOracle>,
        collateral_amount: u64,
        min_tokens_out: u64,
    ) -> Result<()> {
        let oracle_config = &ctx.accounts.oracle_config;

        require!(oracle_config.enabled, OracleError::OracleDisabled);
        require!(oracle_config.last_price > 0, OracleError::InvalidPrice);

        // Check price freshness
        let clock = Clock::get()?;
        let slots_since_update = clock.slot.saturating_sub(oracle_config.last_price_slot);
        require!(
            slots_since_update <= oracle_config.max_staleness_slots,
            OracleError::StalePriceData
        );

        // Calculate tokens to mint based on oracle price
        let price_decimals: u64 = 1_000_000_000; // 10^9
        let tokens_out = (collateral_amount as u128)
            .checked_mul(price_decimals as u128)
            .ok_or(OracleError::MathOverflow)?
            .checked_div(oracle_config.last_price as u128)
            .ok_or(OracleError::MathOverflow)? as u64;

        require!(tokens_out >= min_tokens_out, OracleError::SlippageExceeded);

        // CPI to sss-token program to mint
        let mint_ix = anchor_lang::solana_program::instruction::Instruction {
            program_id: ctx.accounts.sss_token_program.key(),
            accounts: vec![
                anchor_lang::solana_program::instruction::AccountMeta::new_readonly(
                    ctx.accounts.minter.key(),
                    true,
                ),
                anchor_lang::solana_program::instruction::AccountMeta::new(
                    ctx.accounts.stablecoin_config.key(),
                    false,
                ),
                anchor_lang::solana_program::instruction::AccountMeta::new(
                    ctx.accounts.role_config.key(),
                    false,
                ),
                anchor_lang::solana_program::instruction::AccountMeta::new(
                    ctx.accounts.stablecoin_mint.key(),
                    false,
                ),
                anchor_lang::solana_program::instruction::AccountMeta::new(
                    ctx.accounts.recipient_token_account.key(),
                    false,
                ),
                anchor_lang::solana_program::instruction::AccountMeta::new_readonly(
                    ctx.accounts.token_program.key(),
                    false,
                ),
            ],
            data: {
                let mut data = anchor_lang::solana_program::hash::hash(b"global:mint_tokens")
                    .to_bytes()[..8]
                    .to_vec();
                data.extend_from_slice(&tokens_out.to_le_bytes());
                data
            },
        };

        anchor_lang::solana_program::program::invoke(
            &mint_ix,
            &[
                ctx.accounts.minter.to_account_info(),
                ctx.accounts.stablecoin_config.to_account_info(),
                ctx.accounts.role_config.to_account_info(),
                ctx.accounts.stablecoin_mint.to_account_info(),
                ctx.accounts.recipient_token_account.to_account_info(),
                ctx.accounts.token_program.to_account_info(),
            ],
        )?;

        emit!(OracleMint {
            stablecoin_mint: oracle_config.stablecoin_mint,
            minter: ctx.accounts.minter.key(),
            collateral_amount,
            tokens_minted: tokens_out,
            oracle_price: oracle_config.last_price,
        });

        Ok(())
    }

    /// Update oracle configuration (authority only).
    pub fn update_oracle(
        ctx: Context<UpdateOracle>,
        max_staleness_slots: Option<u64>,
        max_confidence_interval: Option<u64>,
        enabled: Option<bool>,
    ) -> Result<()> {
        let oracle_config = &mut ctx.accounts.oracle_config;

        if let Some(staleness) = max_staleness_slots {
            oracle_config.max_staleness_slots = staleness;
        }

        if let Some(confidence) = max_confidence_interval {
            oracle_config.max_confidence_interval = confidence;
        }

        if let Some(enabled) = enabled {
            oracle_config.enabled = enabled;
        }

        emit!(OracleUpdated {
            stablecoin_mint: oracle_config.stablecoin_mint,
            enabled: oracle_config.enabled,
            max_staleness_slots: oracle_config.max_staleness_slots,
        });

        Ok(())
    }
}

// ── Instruction Accounts ────────────────────────────────────────────

#[derive(Accounts)]
pub struct InitializeOracle<'info> {
    #[account(
        init,
        payer = authority,
        space = OracleConfig::LEN,
        seeds = [b"oracle", stablecoin_mint.key().as_ref()],
        bump,
    )]
    pub oracle_config: Account<'info, OracleConfig>,

    /// CHECK: Validated by seed derivation.
    pub stablecoin_mint: UncheckedAccount<'info>,

    /// CHECK: Validated by the oracle at price read time.
    pub switchboard_feed: UncheckedAccount<'info>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct GetPrice<'info> {
    #[account(
        mut,
        seeds = [b"oracle", oracle_config.stablecoin_mint.as_ref()],
        bump = oracle_config.bump,
        has_one = switchboard_feed @ OracleError::InvalidFeed,
    )]
    pub oracle_config: Account<'info, OracleConfig>,

    /// CHECK: Validated by has_one constraint.
    pub switchboard_feed: UncheckedAccount<'info>,
}

#[derive(Accounts)]
pub struct MintWithOracle<'info> {
    #[account(
        seeds = [b"oracle", oracle_config.stablecoin_mint.as_ref()],
        bump = oracle_config.bump,
    )]
    pub oracle_config: Account<'info, OracleConfig>,

    /// CHECK: Passed through to sss-token CPI.
    #[account(mut)]
    pub stablecoin_config: UncheckedAccount<'info>,

    /// CHECK: Passed through to sss-token CPI.
    #[account(mut)]
    pub role_config: UncheckedAccount<'info>,

    /// CHECK: The stablecoin mint, passed through to sss-token CPI.
    #[account(mut)]
    pub stablecoin_mint: UncheckedAccount<'info>,

    /// CHECK: Recipient token account, passed through to sss-token CPI.
    #[account(mut)]
    pub recipient_token_account: UncheckedAccount<'info>,

    #[account(mut)]
    pub minter: Signer<'info>,

    /// CHECK: sss-token program for CPI.
    pub sss_token_program: UncheckedAccount<'info>,

    /// CHECK: Token-2022 program.
    pub token_program: UncheckedAccount<'info>,
}

#[derive(Accounts)]
pub struct UpdateOracle<'info> {
    #[account(
        mut,
        seeds = [b"oracle", oracle_config.stablecoin_mint.as_ref()],
        bump = oracle_config.bump,
        has_one = authority @ OracleError::Unauthorized,
    )]
    pub oracle_config: Account<'info, OracleConfig>,

    pub authority: Signer<'info>,
}

// ── Events ──────────────────────────────────────────────────────────

#[event]
pub struct OracleInitialized {
    pub stablecoin_mint: Pubkey,
    pub switchboard_feed: Pubkey,
    pub base_currency: String,
    pub quote_currency: String,
}

#[event]
pub struct PriceUpdated {
    pub stablecoin_mint: Pubkey,
    pub price: u64,
    pub slot: u64,
    pub timestamp: i64,
}

#[event]
pub struct OracleMint {
    pub stablecoin_mint: Pubkey,
    pub minter: Pubkey,
    pub collateral_amount: u64,
    pub tokens_minted: u64,
    pub oracle_price: u64,
}

#[event]
pub struct OracleUpdated {
    pub stablecoin_mint: Pubkey,
    pub enabled: bool,
    pub max_staleness_slots: u64,
}
