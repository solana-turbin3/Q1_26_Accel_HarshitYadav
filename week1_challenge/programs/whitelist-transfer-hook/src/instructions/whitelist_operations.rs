use anchor_lang::prelude::*;

use crate::state::{whitelist::Whitelist, Vault};

#[derive(Accounts)]
#[instruction(user:Pubkey)]
pub struct WhitelistOperations<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        has_one = admin ,
        seeds = [b"vault" , admin.key().as_ref()],
        bump
    )]
    pub vault: Account<'info, Vault>,

    #[account(
        init_if_needed,
        payer = admin ,
        space = 8 + Whitelist::INIT_SPACE,
        seeds = [b"whitelist" ,admin.key.as_ref() , user.as_ref()],
        bump,
    )]
    pub whitelist: Account<'info, Whitelist>,
    pub system_program: Program<'info, System>,
}

impl<'info> WhitelistOperations<'info> {
    pub fn add_to_whitelist(
        &mut self,
        address: Pubkey,
        bumps: &WhitelistOperationsBumps,
    ) -> Result<()> {
        self.whitelist.set_inner(Whitelist {
            user_key: address,
            is_whitelisted: true,
            bump: bumps.whitelist,
        });
        Ok(())
    }

    pub fn remove_from_whitelist(
        &mut self,
        address: Pubkey,
        bumps: &WhitelistOperationsBumps,
    ) -> Result<()> {
        self.whitelist.set_inner(Whitelist {
            user_key: address,
            is_whitelisted: false,
            bump: bumps.whitelist,
        });
        Ok(())
    }
}
