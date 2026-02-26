use anchor_lang::prelude::*;

pub mod error;
pub mod instructions;

use instructions::*;

declare_id!("Gf5xP5YMRdhb7jRGiDsZW2guwwRMi4RQt4b5r44VPhTU");

#[program]
pub mod sss_transfer_hook {
    use super::*;

    pub fn initialize_extra_account_meta_list(
        ctx: Context<InitializeExtraAccountMetaList>,
    ) -> Result<()> {
        instructions::initialize_extra_account_meta::handler_initialize(ctx)
    }

    // SPL Transfer Hook `Execute` discriminator:
    // sha256("spl-transfer-hook-interface:execute")[..8]
    #[instruction(discriminator = [105, 37, 101, 197, 75, 251, 102, 26])]
    pub fn transfer_hook(ctx: Context<TransferHook>, amount: u64) -> Result<()> {
        instructions::transfer_hook::handler(ctx, amount)
    }
}
