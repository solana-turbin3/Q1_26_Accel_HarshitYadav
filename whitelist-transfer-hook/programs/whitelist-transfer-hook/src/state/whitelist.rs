use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Whitelist {
    pub key: Pubkey,
    pub bump: u8,
}
