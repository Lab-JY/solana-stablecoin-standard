use anchor_lang::prelude::*;

#[error_code]
pub enum TransferHookError {
    #[msg("Sender is blacklisted")]
    BlacklistedSender,
    #[msg("Receiver is blacklisted")]
    BlacklistedReceiver,
    #[msg("Source account is not currently transferring")]
    NotTransferring,
}
