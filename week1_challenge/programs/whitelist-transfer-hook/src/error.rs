use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("User is not whitelisted")]
    NotWhitelisted,
    #[msg("Admin key mismatch")]
    AdminMismatch,
    #[msg("Mint mismatch")]
    MintMismatch,
}