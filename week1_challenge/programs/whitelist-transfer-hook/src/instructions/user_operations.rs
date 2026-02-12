// use anchor_lang::prelude::*;

// use crate::state::{User, Vault, whitelist::Whitelist};

// #[derive(Accounts)]
// #[instruction(user:Pubkey)]
// pub struct WhitelistOperations<'info> {
//     #[account(mut)]
//     pub user: Signer<'info>,

//     #[account(
//         init_if_needed ,
//         payer = user ,
//         space = 8 + User::INIT_SPACE ,
//         seeds = [b"user-config" , user.key().as_ref()] ,
//         bump ,
//     )]
//     pub user_account: Account<'info, User>,

//     #[account(
//         init_if_needed,
//         payer = admin ,
//         space = 8 + Whitelist::INIT_SPACE,
//         seeds = [b"whitelist" ,admin.key.as_ref() , user.as_ref()],
//         bump,
//     )]
//     pub whitelist: Account<'info, Whitelist>,
//     pub system_program: Program<'info, System>,
// }

// impl<'info> WhitelistOperations<'info> {
//     pub fn add_to_whitelist(
//         &mut self,
//         address: Pubkey,
//         bumps: &WhitelistOperationsBumps,
//     ) -> Result<()> {
//         self.whitelist.set_inner(Whitelist {
//             user_key: address,
//             is_whitelisted: true,
//             bump: bumps.whitelist,
//         });
//         Ok(())
//     }

//     pub fn remove_from_whitelist(
//         &mut self,
//         address: Pubkey,
//         bumps: &WhitelistOperationsBumps,
//     ) -> Result<()> {
//         self.whitelist.set_inner(Whitelist {
//             user_key: address,
//             is_whitelisted: false,
//             bump: bumps.whitelist,
//         });
//         Ok(())
//     }
// }
