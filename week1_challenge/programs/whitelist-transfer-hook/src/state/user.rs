use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct User {
    pub amount: u64,
    pub mint: Pubkey,
    pub bump: u8,
}
