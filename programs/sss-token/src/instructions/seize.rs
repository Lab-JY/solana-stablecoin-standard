use anchor_lang::prelude::*;
use anchor_lang::solana_program::program::invoke_signed;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

use crate::constants::*;
use crate::error::StablecoinError;
use crate::events::TokensSeized;
use crate::state::*;

#[derive(Accounts)]
pub struct Seize<'info> {
    pub seizer: Signer<'info>,

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

    /// The account to seize tokens from
    #[account(
        mut,
        token::mint = mint,
    )]
    pub from_token_account: InterfaceAccount<'info, TokenAccount>,

    /// The treasury/destination to receive seized tokens
    #[account(
        mut,
        token::mint = mint,
    )]
    pub to_token_account: InterfaceAccount<'info, TokenAccount>,

    pub token_program: Interface<'info, TokenInterface>,
}

pub fn handler<'info>(ctx: Context<'_, '_, '_, 'info, Seize<'info>>, amount: u64) -> Result<()> {
    let config = &ctx.accounts.stablecoin_config;

    // Feature gate: compliance must be enabled (permanent delegate required)
    require!(
        config.is_compliance_enabled(),
        StablecoinError::ComplianceNotEnabled
    );

    // Verify caller is seizer
    let seizer_key = ctx.accounts.seizer.key();
    require!(
        seizer_key == ctx.accounts.role_config.seizer
            || seizer_key == ctx.accounts.role_config.master_authority,
        StablecoinError::Unauthorized
    );

    // Use permanent delegate (stablecoin_config PDA) to transfer tokens
    let mint_key = ctx.accounts.mint.key();
    let seeds = &[STABLECOIN_SEED, mint_key.as_ref(), &[config.bump]];
    let signer_seeds = &[&seeds[..]];

    // CPI: transfer_checked using permanent delegate authority
    let mut ix = spl_token_2022::instruction::transfer_checked(
        &ctx.accounts.token_program.key(),
        &ctx.accounts.from_token_account.key(),
        &ctx.accounts.mint.key(),
        &ctx.accounts.to_token_account.key(),
        &ctx.accounts.stablecoin_config.key(), // permanent delegate
        &[],
        amount,
        ctx.accounts.mint.decimals,
    )?;

    let mut cpi_account_infos = vec![
        ctx.accounts.from_token_account.to_account_info(),
        ctx.accounts.mint.to_account_info(),
        ctx.accounts.to_token_account.to_account_info(),
        ctx.accounts.stablecoin_config.to_account_info(),
    ];

    if config.enable_transfer_hook {
        // Forward transfer-hook related accounts from the outer instruction.
        // Expected order (CLI/SDK): validation PDA, sender blacklist PDA, receiver
        // blacklist PDA, sss-token program, transfer-hook program.
        for account in ctx.remaining_accounts {
            if account.is_writable {
                ix.accounts
                    .push(anchor_lang::solana_program::instruction::AccountMeta::new(
                        *account.key,
                        account.is_signer,
                    ));
            } else {
                ix.accounts.push(
                    anchor_lang::solana_program::instruction::AccountMeta::new_readonly(
                        *account.key,
                        account.is_signer,
                    ),
                );
            }
            cpi_account_infos.push(account.clone());
        }
    }

    invoke_signed(
        &ix,
        &cpi_account_infos,
        signer_seeds,
    )?;

    emit!(TokensSeized {
        mint: mint_key,
        from: ctx.accounts.from_token_account.key(),
        to: ctx.accounts.to_token_account.key(),
        amount,
        by: seizer_key,
    });

    Ok(())
}
