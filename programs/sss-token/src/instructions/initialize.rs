use anchor_lang::prelude::*;
use anchor_lang::solana_program::{program::invoke_signed, system_instruction};
use anchor_spl::token_interface::TokenInterface;
use spl_token_2022::{
    extension::{
        default_account_state::instruction as default_account_state_ix,
        metadata_pointer::instruction as metadata_pointer_ix,
        transfer_hook::instruction as transfer_hook_ix, ExtensionType,
    },
    instruction as token_ix,
    state::AccountState,
};

use crate::constants::*;
use crate::error::StablecoinError;
use crate::events::StablecoinInitialized;
use crate::state::*;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct InitializeParams {
    pub name: String,
    pub symbol: String,
    pub uri: String,
    pub decimals: u8,
    pub enable_permanent_delegate: bool,
    pub enable_transfer_hook: bool,
    pub default_account_frozen: bool,
}

#[derive(Accounts)]
#[instruction(params: InitializeParams)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    /// CHECK: Mint account, initialized via CPI to Token-2022
    #[account(mut)]
    pub mint: Signer<'info>,

    #[account(
        init,
        payer = authority,
        space = StablecoinConfig::LEN,
        seeds = [STABLECOIN_SEED, mint.key().as_ref()],
        bump,
    )]
    pub stablecoin_config: Account<'info, StablecoinConfig>,

    #[account(
        init,
        payer = authority,
        space = RoleConfig::LEN,
        seeds = [ROLES_SEED, stablecoin_config.key().as_ref()],
        bump,
    )]
    pub role_config: Account<'info, RoleConfig>,

    /// CHECK: Transfer hook program (optional, required for SSS-2)
    pub transfer_hook_program: Option<UncheckedAccount<'info>>,

    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<Initialize>, params: InitializeParams) -> Result<()> {
    require!(
        params.name.len() <= MAX_NAME_LEN,
        StablecoinError::NameTooLong
    );
    require!(
        params.symbol.len() <= MAX_SYMBOL_LEN,
        StablecoinError::SymbolTooLong
    );
    require!(params.uri.len() <= MAX_URI_LEN, StablecoinError::UriTooLong);

    let mint = &ctx.accounts.mint;
    let authority = &ctx.accounts.authority;
    let token_program = &ctx.accounts.token_program;
    let system_program = &ctx.accounts.system_program;

    // Determine extensions to enable
    let mut extension_types = vec![ExtensionType::MetadataPointer];

    if params.enable_permanent_delegate {
        extension_types.push(ExtensionType::PermanentDelegate);
    }

    if params.enable_transfer_hook {
        extension_types.push(ExtensionType::TransferHook);
    }

    if params.default_account_frozen {
        extension_types.push(ExtensionType::DefaultAccountState);
    }

    // Token-2022 validates mint size at InitializeMint2 time against currently
    // initialized extensions only (fixed-size). Metadata is variable-size and is
    // appended later by the metadata initialize CPI.
    let mint_space =
        ExtensionType::try_calculate_account_len::<spl_token_2022::state::Mint>(&extension_types)?;

    let lamports = Rent::get()?.minimum_balance(mint_space);

    // 1. Create mint account
    invoke_signed(
        &system_instruction::create_account(
            authority.key,
            &mint.key(),
            lamports,
            mint_space as u64,
            &token_program.key(),
        ),
        &[
            authority.to_account_info(),
            mint.to_account_info(),
            system_program.to_account_info(),
        ],
        &[],
    )?;

    // 2. Initialize MetadataPointer (self-referencing)
    invoke_signed(
        &metadata_pointer_ix::initialize(
            &token_program.key(),
            &mint.key(),
            Some(authority.key()),
            Some(mint.key()),
        )?,
        &[mint.to_account_info()],
        &[],
    )?;

    // 3. Initialize PermanentDelegate (SSS-2)
    if params.enable_permanent_delegate {
        let stablecoin_config_key = ctx.accounts.stablecoin_config.key();
        invoke_signed(
            &token_ix::initialize_permanent_delegate(
                &token_program.key(),
                &mint.key(),
                &stablecoin_config_key,
            )?,
            &[mint.to_account_info()],
            &[],
        )?;
    }

    // 4. Initialize TransferHook (SSS-2)
    if params.enable_transfer_hook {
        let hook_program = ctx
            .accounts
            .transfer_hook_program
            .as_ref()
            .ok_or(StablecoinError::ComplianceNotEnabled)?;

        invoke_signed(
            &transfer_hook_ix::initialize(
                &token_program.key(),
                &mint.key(),
                Some(authority.key()),
                Some(hook_program.key()),
            )?,
            &[mint.to_account_info()],
            &[],
        )?;
    }

    // 5. Initialize DefaultAccountState (SSS-2 optional)
    if params.default_account_frozen {
        invoke_signed(
            &default_account_state_ix::initialize_default_account_state(
                &token_program.key(),
                &mint.key(),
                &AccountState::Frozen,
            )?,
            &[mint.to_account_info()],
            &[],
        )?;
    }

    // 6. Initialize mint — config PDA is mint authority and freeze authority
    let stablecoin_config_key = ctx.accounts.stablecoin_config.key();
    invoke_signed(
        &token_ix::initialize_mint2(
            &token_program.key(),
            &mint.key(),
            &stablecoin_config_key,
            Some(&stablecoin_config_key),
            params.decimals,
        )?,
        &[mint.to_account_info()],
        &[],
    )?;

    // 7. Set stablecoin config state
    let config = &mut ctx.accounts.stablecoin_config;
    config.authority = authority.key();
    config.mint = mint.key();
    config.name = params.name.clone();
    config.symbol = params.symbol.clone();
    config.uri = params.uri.clone();
    config.decimals = params.decimals;
    config.paused = false;
    config.total_minted = 0;
    config.total_burned = 0;
    config.enable_permanent_delegate = params.enable_permanent_delegate;
    config.enable_transfer_hook = params.enable_transfer_hook;
    config.default_account_frozen = params.default_account_frozen;
    config.transfer_hook_program = ctx.accounts.transfer_hook_program.as_ref().map(|p| p.key());
    config.pending_authority = None;
    config.bump = ctx.bumps.stablecoin_config;
    config._reserved = [0u8; 31];

    // 8. Set role config state — authority gets all roles initially
    let roles = &mut ctx.accounts.role_config;
    roles.stablecoin = config.key();
    roles.master_authority = authority.key();
    roles.pauser = authority.key();
    roles.minters = vec![];
    roles.burners = vec![];
    roles.blacklister = authority.key();
    roles.seizer = authority.key();
    roles.bump = ctx.bumps.role_config;
    roles._reserved = [0u8; 64];

    // 9. Emit event
    let preset = if params.enable_permanent_delegate && params.enable_transfer_hook {
        "SSS-2"
    } else {
        "SSS-1"
    };

    emit!(StablecoinInitialized {
        mint: mint.key(),
        authority: authority.key(),
        name: params.name,
        symbol: params.symbol,
        decimals: params.decimals,
        preset: preset.to_string(),
    });

    Ok(())
}
