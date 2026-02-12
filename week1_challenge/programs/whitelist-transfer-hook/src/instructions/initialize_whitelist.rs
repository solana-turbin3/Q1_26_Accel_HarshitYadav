use crate::state::{Vault, Whitelist};
use anchor_lang::prelude::*;

#[derive(Accounts)]
#[instruction(user:Pubkey)]
pub struct InitializeWhitelist<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(
        init ,
        payer = admin ,
        space = 8+Whitelist::INIT_SPACE,
        seeds = [b"whitelist" , admin.key().as_ref() , user.as_ref()],
        bump
    )]
    pub whitelist: Account<'info, Whitelist>,

    #[account(
        has_one = admin ,
        seeds = [b"vault" , admin.key().as_ref()],
        bump,
    )]
    pub vault: Account<'info, Vault>,

    pub system_program: Program<'info, System>,
}

impl<'info> InitializeWhitelist<'info> {
    pub fn initialize_whitelist(
        &mut self,
        user: Pubkey,
        bumps: InitializeWhitelistBumps,
    ) -> Result<()> {
        self.whitelist.set_inner(Whitelist {
            user_key: user,
            is_whitelisted: true,
            bump: bumps.whitelist,
        });
        Ok(())
    }
}
