use std::time::SystemTime;
use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{Mint, TokenAccount, TransferChecked, transfer_checked, Burn, burn};
use anchor_spl::token::spl_token::amount_to_ui_amount;
use anchor_spl::token_interface::TokenInterface;
use constant_product_curve::ConstantProduct;
use crate::{AmmError, assert_non_zero, assert_not_expired, assert_not_locked};
use crate::state::Config;

#[derive(Accounts)]
pub struct Withdraw<'info> {
    pub user: Signer<'info>,
    pub mint_x: InterfaceAccount<'info, Mint>,
    pub mint_y: InterfaceAccount<'info, Mint>,
    #[account(
    mut,
    seeds = [b"lp", config.key().as_ref()],
    bump = config.lp_bump
    )]
    pub mint_lp: InterfaceAccount<'info, Mint>,
    #[account(
    mut,
    associated_token::mint = mint_x,
    associated_token::authority = auth
    )]
    pub vault_x: InterfaceAccount<'info, TokenAccount>,
    #[account(
    mut,
    associated_token::mint = mint_y,
    associated_token::authoeity = auth
    )]
    pub vault_y: InterfaceAccount<'info, TokenAccount>,
    #[account(
    init_if_needed,
    payer = user
    associated_token::mint = mint_x,
    associated_token::authority = user
    )]
    pub user_vault_x: InterfaceAccount<'info, TokenAccount>,
    #[account(
    init_if_needed,
    payer = user
    associated_token::mint = mint_y,
    associated_token::authority = user
    )]
    pub user_vault_y: InterfaceAccount<'info, TokenAccount>,
    #[account(
    mut,
    associated_token::mint = mint_lp,
    associated_token::authority = user
    )]
    pub user_vault_lp: InterfaceAccount<'info, TokenAccount>,
    ///CHECK : pda used just for signing purposes
    #[account(
    seeds = [b"auth"]
    bump = config.auth_bump
    )]
    pub auth: UncheckedAccount<'info>,
    #[account(
    has_one = mint_x,
    has_one = mint_y,
    seeds = [
    b"config",
    config.seed.to_le_bytes().as_ref()
    ],
    bump = config.config_bump
    )]
    pub config: Account<'info, Config>,
    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

impl<'info> Withdraw<'info> {
    pub fn withdraw(
        &mut self,
        amount : u64,
        min_x : u64,
        min_y : u64,
        expiration : i64
    ) -> Result<()> {
        assert_not_locked!(self.config.locked);
        assert_not_expired!(expiration);
        assert_non_zero!([amount,min_x,min_y]);

        let amounts = ConstantProduct::xy_withdraw_amounts_from_l(
            self.vault_x.amount,
            self.vault_y.amount,
            self.mint_lp.supply,
            amount,
            6
        ).map_err(AmmError::from)?;

        require!(min_x <= amounts.x && min_y <= amounts.y, AmmError::SlippageExceeded); //min_x chota hi hoga amount.x se ofcource
        self.withdraw_tokens(true, amounts.x)?;
        self.withdraw_tokens(false, amounts.y)?;
        self.burn_lp_tokens(amount)
    }

    pub fn withdraw_tokens(
        &mut self,
        is_x: bool,
        amount: u64,
    ) -> Result<()> {
        let (from, to, mint, decimals) = match is_x {
            true => (self.vault_x.to_account_info(),
                     self.user_vault_x.to_account_info(),
                     self.mint_x.to_account_info(),
                     self.mint_x.decimals
            ),
            false => (self.vault_y.to_account_info(),
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

        let cpi_accounts = TransferChecked {
            from,
            mint,
            to,
            authority: self.auth.to_account_info(),
        };
        let ctx = CpiContext::new_with_signer(self.token_program.to_account_info(), cpi_accounts, signer_seeds);
        transfer_checked(
            ctx,
            amount,
            decimals,
        )
    }
    pub fn burn_lp_tokens(
        &mut self,
        amount: u64,
    ) -> Result<()> {
        let cpi_accounts = Burn {
            mint: self.mint_lp.to_account_info(),
            from: self.user_vault_lp.to_account_info(),
            authority: self.user.to_account_info(),
        };
        let ctx = CpiContext::new(
            self.token_program.to_account_info(),
            cpi_accounts,
        );
        burn(ctx, amount)
    }
}