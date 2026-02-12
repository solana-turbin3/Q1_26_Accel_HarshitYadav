use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{self, Mint, TokenAccount, TokenInterface ,TransferChecked},
};

use crate::{error::ErrorCode, state::{Vault, Whitelist}};

#[derive(Accounts)]
pub struct DepositVault<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    
    #[account(address = vault.mint_add)]
    pub mint: InterfaceAccount<'info, Mint>,

    #[account(
        mut ,
        associated_token::mint = mint,
        associated_token::authority = user,
        associated_token::token_program = token_program
    )]
    pub user_ata: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut ,
        associated_token::mint = mint,
        associated_token::authority = vault,
        associated_token::token_program = token_program
    )]
    pub vault_ata: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"vault" , vault.admin.as_ref()],
        bump = vault.bump
    )]
    pub vault: Account<'info, Vault>,
    /// CHECK: Required if transfer hook enabled
    pub extra_account_meta_list: UncheckedAccount<'info>,

    /// CHECK: Hook program
    pub hook_program: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

impl<'info> DepositVault<'info> {
    pub fn deposit(&mut self, amount: u64, bumps: &DepositVaultBumps) -> Result<()> {

        let vault_mint = self.vault.mint_add ;
        let mint = self.mint.key();
        require!(vault_mint == mint , ErrorCode::MintMismatch);

        let cpi_accounts = TransferChecked {
            from : self.user_ata.to_account_info() ,
            mint : self.mint.to_account_info() ,
            to : self.vault_ata.to_account_info() ,
            authority : self.user.to_account_info() 
        };  
        let decimals = self.mint.decimals ;
        let cpi_ctx = CpiContext::new(self.token_program.to_account_info() , cpi_accounts).with_remaining_accounts(vec![
            self.extra_account_meta_list.to_account_info(),
            self.hook_program.to_account_info(),
        ]) ;
        token_interface::transfer_checked(cpi_ctx, amount, decimals)?;

        Ok(())
    }
}
