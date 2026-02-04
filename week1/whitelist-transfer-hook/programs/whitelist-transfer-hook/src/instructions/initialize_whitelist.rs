use anchor_lang::prelude::*;

use crate::state::Whitelist;

#[derive(Accounts)]
pub struct InitializeWhitelist<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(
        init,
        payer = admin,
        space = 8 + Whitelist::INIT_SPACE , // 8 bytes for discriminator, 
        seeds = [b"whitelist"],
        bump
    )]
    pub whitelist_pda: Account<'info, Whitelist>,
    pub system_program: Program<'info, System>,
}

impl<'info> InitializeWhitelist<'info> {
    pub fn initialize_whitelist(&mut self, bumps: InitializeWhitelistBumps) -> Result<()> {
        // Initialize the whitelist with the key of the admin
        self.whitelist_pda.set_inner(Whitelist {
            key: self.admin.key(),
            bump: bumps.whitelist_pda,
        });

        Ok(())
    }
}
