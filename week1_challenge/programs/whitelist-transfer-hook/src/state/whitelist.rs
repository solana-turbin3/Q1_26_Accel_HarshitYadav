use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Whitelist {
    pub user_key: Pubkey,
    pub is_whitelisted: bool,
    pub bump: u8,
}
