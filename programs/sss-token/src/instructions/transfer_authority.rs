use anchor_lang::prelude::*;

use crate::constants::*;
use crate::error::StablecoinError;
use crate::events::AuthorityTransferred;
use crate::state::*;

#[derive(Accounts)]
pub struct TransferAuthority<'info> {
    pub authority: Signer<'info>,

    #[account(
        mut,
        seeds = [STABLECOIN_SEED, stablecoin_config.mint.as_ref()],
        bump = stablecoin_config.bump,
    )]
    pub stablecoin_config: Account<'info, StablecoinConfig>,

    #[account(
        mut,
        seeds = [ROLES_SEED, stablecoin_config.key().as_ref()],
        bump = role_config.bump,
    )]
    pub role_config: Account<'info, RoleConfig>,
}

pub fn handler(ctx: Context<TransferAuthority>, new_authority: Pubkey) -> Result<()> {
    let authority_key = ctx.accounts.authority.key();
    let role_config = &mut ctx.accounts.role_config;

    require!(
        authority_key == role_config.master_authority,
        StablecoinError::Unauthorized
    );

    let old_authority = role_config.master_authority;

    // Transfer authority on config and roles
    let config = &mut ctx.accounts.stablecoin_config;
    config.authority = new_authority;
    role_config.master_authority = new_authority;

    emit!(AuthorityTransferred {
        mint: config.mint,
        old_authority,
        new_authority,
    });

    Ok(())
}
