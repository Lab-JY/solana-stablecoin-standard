use anchor_lang::prelude::*;
use anchor_spl::token_interface::{self, FreezeAccount, Mint, TokenAccount, TokenInterface};

use crate::constants::*;
use crate::error::StablecoinError;
use crate::events;
use crate::state::*;

#[derive(Accounts)]
pub struct FreezeTokenAccount<'info> {
    pub authority: Signer<'info>,

    #[account(
        seeds = [STABLECOIN_SEED, mint.key().as_ref()],
        bump = stablecoin_config.bump,
    )]
    pub stablecoin_config: Account<'info, StablecoinConfig>,

    #[account(
        seeds = [ROLES_SEED, stablecoin_config.key().as_ref()],
        bump = role_config.bump,
    )]
    pub role_config: Account<'info, RoleConfig>,

    #[account(
        constraint = mint.key() == stablecoin_config.mint,
    )]
    pub mint: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        token::mint = mint,
    )]
    pub token_account: InterfaceAccount<'info, TokenAccount>,

    pub token_program: Interface<'info, TokenInterface>,
}

pub fn handler(ctx: Context<FreezeTokenAccount>) -> Result<()> {
    let authority_key = ctx.accounts.authority.key();
    let role_config = &ctx.accounts.role_config;

    // Must be authority or pauser
    require!(
        authority_key == role_config.master_authority || authority_key == role_config.pauser,
        StablecoinError::Unauthorized
    );

    let mint_key = ctx.accounts.mint.key();
    let seeds = &[
        STABLECOIN_SEED,
        mint_key.as_ref(),
        &[ctx.accounts.stablecoin_config.bump],
    ];
    let signer_seeds = &[&seeds[..]];

    let cpi_accounts = FreezeAccount {
        account: ctx.accounts.token_account.to_account_info(),
        mint: ctx.accounts.mint.to_account_info(),
        authority: ctx.accounts.stablecoin_config.to_account_info(),
    };
    let cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        cpi_accounts,
        signer_seeds,
    );
    token_interface::freeze_account(cpi_ctx)?;

    emit!(events::AccountFrozen {
        mint: mint_key,
        account: ctx.accounts.token_account.key(),
        by: authority_key,
    });

    Ok(())
}
