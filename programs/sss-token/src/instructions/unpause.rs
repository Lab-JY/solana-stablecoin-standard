use anchor_lang::prelude::*;

use crate::constants::*;
use crate::error::StablecoinError;
use crate::events::StablecoinUnpaused;
use crate::state::*;

#[derive(Accounts)]
pub struct Unpause<'info> {
    pub pauser: Signer<'info>,

    #[account(
        mut,
        seeds = [STABLECOIN_SEED, stablecoin_config.mint.as_ref()],
        bump = stablecoin_config.bump,
    )]
    pub stablecoin_config: Account<'info, StablecoinConfig>,

    #[account(
        seeds = [ROLES_SEED, stablecoin_config.key().as_ref()],
        bump = role_config.bump,
    )]
    pub role_config: Account<'info, RoleConfig>,
}

pub fn handler(ctx: Context<Unpause>) -> Result<()> {
    let pauser_key = ctx.accounts.pauser.key();
    let role_config = &ctx.accounts.role_config;

    require!(
        pauser_key == role_config.pauser || pauser_key == role_config.master_authority,
        StablecoinError::Unauthorized
    );

    let config = &mut ctx.accounts.stablecoin_config;
    config.paused = false;

    emit!(StablecoinUnpaused {
        mint: config.mint,
        by: pauser_key,
    });

    Ok(())
}
