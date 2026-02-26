use anchor_lang::prelude::*;

use crate::constants::*;
use crate::error::StablecoinError;
use crate::events::RolesUpdated;
use crate::state::*;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub enum RoleAction {
    SetPauser { address: Pubkey },
    SetBlacklister { address: Pubkey },
    SetSeizer { address: Pubkey },
    AddBurner { address: Pubkey },
    RemoveBurner { address: Pubkey },
}

#[derive(Accounts)]
pub struct UpdateRoles<'info> {
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

pub fn handler(ctx: Context<UpdateRoles>, action: RoleAction) -> Result<()> {
    let authority_key = ctx.accounts.authority.key();
    let role_config = &mut ctx.accounts.role_config;

    require!(
        authority_key == role_config.master_authority,
        StablecoinError::Unauthorized
    );

    let mint_key = ctx.accounts.stablecoin_config.mint;

    match action {
        RoleAction::SetPauser { address } => {
            role_config.pauser = address;
            emit!(RolesUpdated {
                mint: mint_key,
                role: "pauser".to_string(),
                address,
                by: authority_key,
            });
        }
        RoleAction::SetBlacklister { address } => {
            role_config.blacklister = address;
            emit!(RolesUpdated {
                mint: mint_key,
                role: "blacklister".to_string(),
                address,
                by: authority_key,
            });
        }
        RoleAction::SetSeizer { address } => {
            role_config.seizer = address;
            emit!(RolesUpdated {
                mint: mint_key,
                role: "seizer".to_string(),
                address,
                by: authority_key,
            });
        }
        RoleAction::AddBurner { address } => {
            require!(
                role_config.burners.len() < MAX_BURNERS,
                StablecoinError::MaxBurnersReached
            );
            require!(
                !role_config.burners.contains(&address),
                StablecoinError::BurnerAlreadyExists
            );
            role_config.burners.push(address);
            emit!(RolesUpdated {
                mint: mint_key,
                role: "burner".to_string(),
                address,
                by: authority_key,
            });
        }
        RoleAction::RemoveBurner { address } => {
            let idx = role_config
                .burners
                .iter()
                .position(|b| *b == address)
                .ok_or(StablecoinError::BurnerNotFound)?;
            role_config.burners.remove(idx);
            emit!(RolesUpdated {
                mint: mint_key,
                role: "burner_removed".to_string(),
                address,
                by: authority_key,
            });
        }
    }

    Ok(())
}
