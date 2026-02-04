use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Whitelist {
    pub user : Pubkey,
    pub bump : u8,
}
