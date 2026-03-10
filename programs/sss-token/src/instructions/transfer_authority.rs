use anchor_lang::prelude::*;

use crate::constants::*;
use crate::error::StablecoinError;
use crate::events::{AuthorityTransferAccepted, AuthorityTransferProposed};
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
        seeds = [ROLES_SEED, stablecoin_config.key().as_ref()],
        bump = role_config.bump,
    )]
    pub role_config: Account<'info, RoleConfig>,
}

/// Step 1: Current authority proposes a new authority. Sets pending_authority.
pub fn handler(ctx: Context<TransferAuthority>, new_authority: Pubkey) -> Result<()> {
    let authority_key = ctx.accounts.authority.key();
    let role_config = &ctx.accounts.role_config;

    require!(
        authority_key == role_config.master_authority,
        StablecoinError::Unauthorized
    );

    let config = &mut ctx.accounts.stablecoin_config;
    config.pending_authority = Some(new_authority);

    emit!(AuthorityTransferProposed {
        mint: config.mint,
        current_authority: authority_key,
        pending_authority: new_authority,
    });

    Ok(())
}

#[derive(Accounts)]
pub struct AcceptAuthority<'info> {
    pub new_authority: Signer<'info>,

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

/// Step 2: Pending authority accepts the transfer. Moves authority.
pub fn accept_handler(ctx: Context<AcceptAuthority>) -> Result<()> {
    let config = &mut ctx.accounts.stablecoin_config;

    let pending = config
        .pending_authority
        .ok_or(StablecoinError::NoPendingAuthority)?;

    let new_authority_key = ctx.accounts.new_authority.key();
    require!(
        new_authority_key == pending,
        StablecoinError::NotPendingAuthority
    );

    let old_authority = config.authority;
    config.authority = new_authority_key;
    config.pending_authority = None;

    let role_config = &mut ctx.accounts.role_config;
    role_config.master_authority = new_authority_key;

    emit!(AuthorityTransferAccepted {
        mint: config.mint,
        old_authority,
        new_authority: new_authority_key,
    });

    Ok(())
}
