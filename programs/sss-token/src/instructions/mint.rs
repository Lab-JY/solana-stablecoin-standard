use anchor_lang::prelude::*;
use anchor_spl::token_interface::{self, Mint, MintTo, TokenAccount, TokenInterface};

use crate::constants::*;
use crate::error::StablecoinError;
use crate::events::TokensMinted;
use crate::state::*;

#[derive(Accounts)]
pub struct MintTokens<'info> {
    #[account(mut)]
    pub minter: Signer<'info>,

    #[account(
        mut,
        seeds = [STABLECOIN_SEED, mint.key().as_ref()],
        bump = stablecoin_config.bump,
    )]
    pub stablecoin_config: Account<'info, StablecoinConfig>,

    #[account(
        mut,
        seeds = [ROLES_SEED, stablecoin_config.key().as_ref()],
        bump = role_config.bump,
    )]
    pub role_config: Account<'info, RoleConfig>,

    #[account(
        mut,
        constraint = mint.key() == stablecoin_config.mint,
    )]
    pub mint: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        token::mint = mint,
    )]
    pub recipient_token_account: InterfaceAccount<'info, TokenAccount>,

    pub token_program: Interface<'info, TokenInterface>,
}

pub fn handler(ctx: Context<MintTokens>, amount: u64) -> Result<()> {
    let config = &ctx.accounts.stablecoin_config;
    require!(!config.paused, StablecoinError::Paused);

    // Find minter in role config
    let role_config = &mut ctx.accounts.role_config;
    let minter_key = ctx.accounts.minter.key();
    let minter_info = role_config
        .minters
        .iter_mut()
        .find(|m| m.address == minter_key)
        .ok_or(StablecoinError::MinterNotFound)?;

    // Check quota
    require!(
        minter_info.remaining_quota() >= amount,
        StablecoinError::MinterQuotaExceeded
    );

    // CPI: mint_to via PDA signer
    let mint_key = ctx.accounts.mint.key();
    let seeds = &[STABLECOIN_SEED, mint_key.as_ref(), &[config.bump]];
    let signer_seeds = &[&seeds[..]];

    let cpi_accounts = MintTo {
        mint: ctx.accounts.mint.to_account_info(),
        to: ctx.accounts.recipient_token_account.to_account_info(),
        authority: ctx.accounts.stablecoin_config.to_account_info(),
    };
    let cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        cpi_accounts,
        signer_seeds,
    );
    token_interface::mint_to(cpi_ctx, amount)?;

    // Update minter stats
    minter_info.minted = minter_info
        .minted
        .checked_add(amount)
        .ok_or(StablecoinError::MathOverflow)?;

    // Update global stats
    let config = &mut ctx.accounts.stablecoin_config;
    config.total_minted = config
        .total_minted
        .checked_add(amount)
        .ok_or(StablecoinError::MathOverflow)?;

    emit!(TokensMinted {
        mint: ctx.accounts.mint.key(),
        recipient: ctx.accounts.recipient_token_account.key(),
        amount,
        minter: minter_key,
    });

    Ok(())
}
