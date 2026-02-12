use anchor_lang::prelude::*;

use crate::state::UserAccount;

#[derive(Accounts)]
pub struct VrfCallback<'info> {
    #[account(address = ephemeral_vrf_sdk::consts::VRF_PROGRAM_IDENTITY)]
    pub vrf_program_identity: Signer<'info>,
    #[account(mut)]
    pub user_account: Account<'info, UserAccount>,
}

impl<'info> VrfCallback<'info> {
    pub fn callback(&mut self, randomness: [u8; 32]) -> Result<()> {
        let rand_value = ephemeral_vrf_sdk::rnd::random_u64(&randomness);

        self.user_account.data = rand_value;

        Ok(())
    }
}