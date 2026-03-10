use anchor_lang::prelude::*;
use anchor_spl::token_interface::Mint;

use crate::error::TransferHookError;

/// PDA seed used by sss-token program for stablecoin configs.
const STABLECOIN_SEED: &[u8] = b"stablecoin";
/// sss-token program ID.
const SSS_TOKEN_PROGRAM_ID: Pubkey = pubkey!("AhZamuppxULmpM9QGXcZJ9ZR3fvQbDd4gPsxLtDoMQmE");

/// The discriminator for the BlacklistEntry account from sss-token program.
/// This is the first 8 bytes of sha256("account:BlacklistEntry").
/// Computed at init time to stay in sync with Anchor's derivation.
fn blacklist_entry_discriminator() -> [u8; 8] {
    let hash = anchor_lang::solana_program::hash::hash(b"account:BlacklistEntry");
    let mut disc = [0u8; 8];
    disc.copy_from_slice(&hash.to_bytes()[..8]);
    disc
}

#[derive(Accounts)]
pub struct TransferHook<'info> {
    /// Source token account
    /// CHECK: validated by Token-2022 before invoking hook
    pub source_token_account: UncheckedAccount<'info>,

    pub mint: InterfaceAccount<'info, Mint>,

    /// Destination token account
    /// CHECK: validated by Token-2022 before invoking hook
    pub destination_token_account: UncheckedAccount<'info>,

    /// Owner/delegate of source token account
    /// CHECK: validated by Token-2022 before invoking hook
    pub owner: UncheckedAccount<'info>,

    /// CHECK: ExtraAccountMetas PDA
    #[account(
        seeds = [b"extra-account-metas", mint.key().as_ref()],
        bump,
    )]
    pub extra_account_meta_list: UncheckedAccount<'info>,

    /// CHECK: Program ID is validated with address constraint.
    /// Needed by ExtraAccountMetas PDA resolution and delegate PDA derivation.
    #[account(address = SSS_TOKEN_PROGRAM_ID)]
    pub sss_token_program: UncheckedAccount<'info>,

    /// Sender's BlacklistEntry PDA from sss-token program.
    /// If the account exists and has data, the sender is blacklisted.
    /// CHECK: PDA derived by ExtraAccountMetas resolution; we only read data to check existence.
    pub sender_blacklist_entry: UncheckedAccount<'info>,

    /// Receiver's BlacklistEntry PDA from sss-token program.
    /// If the account exists and has data, the receiver is blacklisted.
    /// CHECK: PDA derived by ExtraAccountMetas resolution; we only read data to check existence.
    pub receiver_blacklist_entry: UncheckedAccount<'info>,
}

pub fn handler(ctx: Context<TransferHook>, _amount: u64) -> Result<()> {
    // Allow seizure transfers where the authority/delegate is the stablecoin PDA.
    if is_stablecoin_delegate(&ctx)? {
        return Ok(());
    }

    // Check if sender is blacklisted
    // A blacklisted address has a BlacklistEntry PDA that is initialized (has data)
    let sender_bl = &ctx.accounts.sender_blacklist_entry;
    if is_blacklisted(sender_bl)? {
        return Err(TransferHookError::BlacklistedSender.into());
    }

    // Check if receiver is blacklisted
    let receiver_bl = &ctx.accounts.receiver_blacklist_entry;
    if is_blacklisted(receiver_bl)? {
        return Err(TransferHookError::BlacklistedReceiver.into());
    }

    Ok(())
}

fn is_stablecoin_delegate(ctx: &Context<TransferHook>) -> Result<bool> {
    let mint_key = ctx.accounts.mint.key();
    let (expected_delegate, _) = Pubkey::find_program_address(
        &[STABLECOIN_SEED, mint_key.as_ref()],
        &ctx.accounts.sss_token_program.key(),
    );
    Ok(ctx.accounts.owner.key() == expected_delegate)
}

/// Check if a BlacklistEntry account is initialized (meaning the address is blacklisted).
/// We check: account has data, data length > 8 (discriminator), and discriminator matches.
fn is_blacklisted(account: &UncheckedAccount) -> Result<bool> {
    let data = account.try_borrow_data()?;

    // If account has no data or is empty, not blacklisted
    if data.len() < 8 {
        return Ok(false);
    }

    // Check if the discriminator matches BlacklistEntry
    if data[0..8] == blacklist_entry_discriminator() {
        return Ok(true);
    }

    Ok(false)
}
