use anchor_lang::{prelude::*, system_program};

use crate::state::{
    whitelist::{self, Whitelist},
    whitelist_users::WhitelistUsers,
};
#[derive(Accounts)]
pub struct AddToWhitelist<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(
        mut , 
        seeds = [b"whitelist"] ,
        bump = whitelist_pda.bump
    )]
    pub whitelist_pda: Account<'info, Whitelist>,

    /// CHECK: the address to be whitelisted
    pub user: UncheckedAccount<'info>,

    #[account(
        init , 
        space = 8 + WhitelistUsers::INIT_SPACE ,
        payer = admin ,
        seeds = [b"whitelist_users" , user.key().as_ref()] ,
        bump
    )]
    pub whitelist_users: Account<'info, WhitelistUsers>,
    pub system_program: Program<'info, System>,
}

pub fn add_to_whitelist(ctx: Context<AddToWhitelist>) -> Result<()> {
    let whitelist_users = &mut ctx.accounts.whitelist_users;
    whitelist_users.user = ctx.accounts.user.key();
    whitelist_users.bump = ctx.bumps.whitelist_users;
    msg!("User {} added to whitelist", ctx.accounts.user.key());
    Ok(())
}


#[derive(Accounts)]
pub struct RemoveFromWhitelist<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(
        mut , 
        seeds = [b"whitelist"] ,
        bump = whitelist_pda.bump
    )]
    pub whitelist_pda: Account<'info, Whitelist>,

    /// CHECK: the address to be whitelisted
    pub user: UncheckedAccount<'info>,

    #[account(
        mut , 
        close = admin ,
        seeds = [b"whitelist_users" , user.key().as_ref()] ,
        bump
    )]
    pub whitelist_users: Account<'info, WhitelistUsers>,
    pub system_program: Program<'info, System>,
}

pub fn remove_from_whitelist(ctx: Context<RemoveFromWhitelist>) -> Result<()> {
    // for closing a pda , just do close = admin  
    msg!("User {} removed from whitelist", ctx.accounts.user.key());
    Ok(())
}
