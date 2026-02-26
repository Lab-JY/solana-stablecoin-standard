use anchor_lang::prelude::*;
use anchor_lang::system_program;
use anchor_spl::token_interface::Mint;
use spl_tlv_account_resolution::{
    account::ExtraAccountMeta, seeds::Seed, state::ExtraAccountMetaList,
};
use spl_transfer_hook_interface::instruction::ExecuteInstruction;

/// PDA seeds used by the sss-token program for blacklist entries
const BLACKLIST_SEED: &[u8] = b"blacklist";

/// SSS-token program ID — the program that owns BlacklistEntry accounts
const SSS_TOKEN_PROGRAM_ID: Pubkey = pubkey!("AhZamuppxULmpM9QGXcZJ9ZR3fvQbDd4gPsxLtDoMQmE");

#[derive(Accounts)]
pub struct InitializeExtraAccountMetaList<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    /// CHECK: ExtraAccountMetas PDA — validated by seeds
    #[account(
        mut,
        seeds = [b"extra-account-metas", mint.key().as_ref()],
        bump,
    )]
    pub extra_account_meta_list: UncheckedAccount<'info>,

    pub mint: InterfaceAccount<'info, Mint>,

    pub system_program: Program<'info, System>,
}

pub fn handler_initialize(ctx: Context<InitializeExtraAccountMetaList>) -> Result<()> {
    // Define the extra accounts that the transfer hook needs:
    // 1. sss-token program account
    // 2. Sender's BlacklistEntry PDA (from sss-token program)
    // 3. Receiver's BlacklistEntry PDA (from sss-token program)
    //
    // The sender/receiver owner keys are extracted from the token account data:
    // In a Token-2022 account, the owner field is at bytes 32..64.
    //
    // NOTE: ExtraAccountMetaList resolves entries in-order against the current
    // instruction accounts. Therefore, the external PDA entries must reference
    // a program index that already exists at resolution time.

    let extra_account_metas = vec![
        // sss-token program (needed to derive BlacklistEntry PDAs below)
        ExtraAccountMeta::new_with_pubkey(&SSS_TOKEN_PROGRAM_ID, false, false)?,
        // Sender BlacklistEntry PDA: seeds = ["blacklist", mint, sender_owner]
        // sender_owner is extracted from source token account (account index 0) at offset 32, length 32
        ExtraAccountMeta::new_external_pda_with_seeds(
            5, // sss-token program index after appending the entry above
            &[
                Seed::Literal {
                    bytes: BLACKLIST_SEED.to_vec(),
                },
                Seed::AccountKey { index: 1 }, // mint account
                Seed::AccountData {
                    account_index: 0, // source token account
                    data_index: 32,
                    length: 32,
                },
            ],
            false, // is_signer
            false, // is_writable
        )?,
        // Receiver BlacklistEntry PDA: seeds = ["blacklist", mint, receiver_owner]
        // receiver_owner is extracted from destination token account (account index 2) at offset 32, length 32
        ExtraAccountMeta::new_external_pda_with_seeds(
            5, // sss-token program index after appending the entry above
            &[
                Seed::Literal {
                    bytes: BLACKLIST_SEED.to_vec(),
                },
                Seed::AccountKey { index: 1 }, // mint account
                Seed::AccountData {
                    account_index: 2, // destination token account
                    data_index: 32,
                    length: 32,
                },
            ],
            false, // is_signer
            false, // is_writable
        )?,
    ];

    // Calculate required account size
    let account_size = ExtraAccountMetaList::size_of(extra_account_metas.len())?;
    let lamports = Rent::get()?.minimum_balance(account_size);

    let mint_key = ctx.accounts.mint.key();
    let signer_seeds: &[&[u8]] = &[
        b"extra-account-metas",
        mint_key.as_ref(),
        &[ctx.bumps.extra_account_meta_list],
    ];

    // Create the ExtraAccountMetas PDA
    system_program::create_account(
        CpiContext::new_with_signer(
            ctx.accounts.system_program.to_account_info(),
            system_program::CreateAccount {
                from: ctx.accounts.payer.to_account_info(),
                to: ctx.accounts.extra_account_meta_list.to_account_info(),
            },
            &[signer_seeds],
        ),
        lamports,
        account_size as u64,
        &crate::id(),
    )?;

    // Initialize the account data with the extra account metas
    ExtraAccountMetaList::init::<ExecuteInstruction>(
        &mut ctx.accounts.extra_account_meta_list.try_borrow_mut_data()?,
        &extra_account_metas,
    )?;

    Ok(())
}
