use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{Mint, Token, TokenAccount, transfer_checked, TransferChecked};
use anchor_spl::token_interface::TokenInterface;
use crate::{assert_non_zero, assert_not_expired, assert_not_locked};
use crate::state::Config;

#[derive(Accounts)]
pub struct Swap<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    pub mint_x: InterfaceAccount<'info, Mint>,
    pub mint_y: InterfaceAccount<'info, Mint>,
    #[account(
    mut,
    associated_token::mint = mint_x,
    associated_token::authority = auth
    )]
    pub vault_x: InterfaceAccount<'info, TokenAccount>,
    #[account(
    mut,
    associated_token::mint = mint_y,
    associated_token::aithority = auth
    )]
    pub vault_y: InterfaceAccount<'info, TokenAccount>,
    #[account(
    init_if_needed,
    payer = user,
    associated_token::mint = mint_x,
    associated_token::authority = user
    )]
    pub user_vault_x: InterfaceAccount<'info, TokenAccount>,
    #[account(
    mut,
    payer = user,
    associated_token::mint = mint_y,
    associated_token::authority = user
    )]
    pub user_vault_y: InterfaceAccount<'info, TokenAccount>,
    #[account(
    seeds = [b"auth"],
    bump = congif.auth_bump
    )]
    pub auth: UncheckedAccount<'info>,
    #[account(
    init_if_needed,
    payer = user,
    seeds = [b"config", config.seed.to_le_bytes().key().as_ref()],
    bump = config.config_bump
    )]
    pub config: Account<'info, Config>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

impl<'info> Swap<'info> {
    pub fn swap(
        &mut self,
        is_x: bool,
        amount: u64,
        expiration: i64,
    ) -> Result<()> {
        assert_not_locked!(self.config.locked);
        assert_not_expired!(expiration);
        assert_non_zero!([amount]);

        //
    }

    pub fn deposit(
        &mut self,
        is_x: bool,
        amount: u64,
    ) -> Result<()> {
        let (from, to, mint, decimals) = match is_x {
            true => (
                self.user_vault_x.to_account_info(),
                self.vault_x.to_account_info(),
                self.mint_x.to_account_info(),
                self.mint_x.decimals
            ),
            false => (
                self.user_vault_y.to_account_info(),
                self.vault_y.to_account_info(),
                self.mint_y.to_account_info(),
                self.mint_y.decimals
            )
        };

        let cpi_account = TransferChecked {
            from,
            mint,
            to,
            authority: self.user.to_account_info(),
        };

        let ctx = CpiContext::new(
            self.token_program.to_account_info(),
            cpi_account,
        );

        transfer_checked(ctx, amount, decimals)
    }

    pub fn withdraw(
        &mut self,
        is_x: bool,
        amount: u64,
    ) -> Result<()> {
        let (from, to, mint, decimals) = match is_x {
            true => (
                self.vault_x.to_account_info(),
                self.user_vault_x.to_account_info(),
                self.mint_x.to_account_info(),
                self.mint_x.decimals
            ),
            false => (
                self.vault_y.to_account_info(),
                self.user_vault_y.to_account_info(),
                self.mint_y.to_account_info(),
                self.mint_y.decimals
            )
        };

        let seeds = &[
            &b"auth"[..],
            &[self.config.auth_bump],
        ];

        let signer_seeds = &[&seeds[..]];

        let cpi_account = TransferChecked {
            from,
            mint,
            to,
            authority: self.auth.to_account_info(),
        };

        let ctx = CpiContext::new_with_signer(self.token_program.to_account_info(), cpi_account, signer_seeds);

        transfer_checked(ctx, amount, decimals)
    }
}