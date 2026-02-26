use anchor_lang::prelude::*;

use crate::constants::*;
use crate::error::StablecoinError;
use crate::events::MinterUpdated;
use crate::state::*;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub enum MinterAction {
    Add { address: Pubkey, quota: u64 },
    Remove { address: Pubkey },
    UpdateQuota { address: Pubkey, new_quota: u64 },
}

#[derive(Accounts)]
pub struct UpdateMinter<'info> {
    pub authority: Signer<'info>,

    #[account(
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

pub fn handler(ctx: Context<UpdateMinter>, action: MinterAction) -> Result<()> {
    let authority_key = ctx.accounts.authority.key();
    let role_config = &mut ctx.accounts.role_config;

    require!(
        authority_key == role_config.master_authority,
        StablecoinError::Unauthorized
    );

    let mint_key = ctx.accounts.stablecoin_config.mint;

    match action {
        MinterAction::Add { address, quota } => {
            require!(
                role_config.minters.len() < MAX_MINTERS,
                StablecoinError::MaxMintersReached
            );
            require!(
                !role_config.minters.iter().any(|m| m.address == address),
                StablecoinError::MinterAlreadyExists
            );

            role_config.minters.push(MinterInfo {
                address,
                quota,
                minted: 0,
            });

            emit!(MinterUpdated {
                mint: mint_key,
                minter: address,
                quota,
                action: "add".to_string(),
            });
        }
        MinterAction::Remove { address } => {
            let idx = role_config
                .minters
                .iter()
                .position(|m| m.address == address)
                .ok_or(StablecoinError::MinterNotFound)?;

            role_config.minters.remove(idx);

            emit!(MinterUpdated {
                mint: mint_key,
                minter: address,
                quota: 0,
                action: "remove".to_string(),
            });
        }
        MinterAction::UpdateQuota { address, new_quota } => {
            let minter = role_config
                .minters
                .iter_mut()
                .find(|m| m.address == address)
                .ok_or(StablecoinError::MinterNotFound)?;

            minter.quota = new_quota;

            emit!(MinterUpdated {
                mint: mint_key,
                minter: address,
                quota: new_quota,
                action: "update_quota".to_string(),
            });
        }
    }

    Ok(())
}
