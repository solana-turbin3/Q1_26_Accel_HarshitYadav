use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Too early to take from the escrow")]
    Locked,
}