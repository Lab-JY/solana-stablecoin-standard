use anchor_lang::prelude::*;

use crate::constants::*;
use crate::error::StablecoinError;
use crate::events::AddressBlacklisted;
use crate::state::*;

#[derive(Accounts)]
#[instruction(address: Pubkey, reason: String)]
pub struct AddToBlacklist<'info> {
    #[account(mut)]
    pub blacklister: Signer<'info>,

    #[account(
        seeds = [STABLECOIN_SEED, stablecoin_config.mint.as_ref()],
        bump = stablecoin_config.bump,
    )]
    pub stablecoin_config: Account<'info, StablecoinConfig>,

    #[account(
        seeds = [ROLES_SEED, stablecoin_config.key().as_ref()],
        bump = role_config.bump,
    )]
    pub role_config: Account<'info, RoleConfig>,

    #[account(
        init,
        payer = blacklister,
        space = BlacklistEntry::LEN,
        seeds = [BLACKLIST_SEED, stablecoin_config.mint.as_ref(), address.as_ref()],
        bump,
    )]
    pub blacklist_entry: Account<'info, BlacklistEntry>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<AddToBlacklist>, address: Pubkey, reason: String) -> Result<()> {
    let config = &ctx.accounts.stablecoin_config;

    // Feature gate: compliance must be enabled
    require!(
        config.is_compliance_enabled(),
        StablecoinError::ComplianceNotEnabled
    );

    require!(
        reason.len() <= MAX_REASON_LEN,
        StablecoinError::ReasonTooLong
    );

    // Verify caller is blacklister
    let blacklister_key = ctx.accounts.blacklister.key();
    require!(
        blacklister_key == ctx.accounts.role_config.blacklister
            || blacklister_key == ctx.accounts.role_config.master_authority,
        StablecoinError::Unauthorized
    );

    let clock = Clock::get()?;

    // Set blacklist entry
    let entry = &mut ctx.accounts.blacklist_entry;
    entry.stablecoin = config.key();
    entry.address = address;
    entry.reason = reason.clone();
    entry.added_at = clock.unix_timestamp;
    entry.added_by = blacklister_key;
    entry.bump = ctx.bumps.blacklist_entry;

    emit!(AddressBlacklisted {
        mint: config.mint,
        address,
        reason,
        by: blacklister_key,
    });

    Ok(())
}
