use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
    pub struct WhitelistUsers {
        pub key: Pubkey,
        pub bump: u8,
    }
// the idea is , ki if pda exists , the pub key is whitelisted
