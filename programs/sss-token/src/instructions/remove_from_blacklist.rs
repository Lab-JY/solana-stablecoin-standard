use anchor_lang::prelude::*;

use crate::constants::*;
use crate::error::StablecoinError;
use crate::events::AddressUnblacklisted;
use crate::state::*;

#[derive(Accounts)]
#[instruction(address: Pubkey)]
pub struct RemoveFromBlacklist<'info> {
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
        mut,
        close = blacklister,
        seeds = [BLACKLIST_SEED, stablecoin_config.mint.as_ref(), address.as_ref()],
        bump = blacklist_entry.bump,
    )]
    pub blacklist_entry: Account<'info, BlacklistEntry>,
}

pub fn handler(ctx: Context<RemoveFromBlacklist>, address: Pubkey) -> Result<()> {
    let config = &ctx.accounts.stablecoin_config;

    require!(!config.paused, StablecoinError::Paused);

    // Feature gate: compliance must be enabled
    require!(
        config.is_compliance_enabled(),
        StablecoinError::ComplianceNotEnabled
    );

    // Verify caller is blacklister
    let blacklister_key = ctx.accounts.blacklister.key();
    require!(
        blacklister_key == ctx.accounts.role_config.blacklister
            || blacklister_key == ctx.accounts.role_config.master_authority,
        StablecoinError::Unauthorized
    );

    // blacklist_entry is closed via Anchor `close` constraint above

    emit!(AddressUnblacklisted {
        mint: config.mint,
        address,
        by: blacklister_key,
    });

    Ok(())
}
