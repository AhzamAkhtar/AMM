use anchor_lang::prelude::*;

mod state;
use state::*;
mod contexts;
use contexts::*;

mod errors;
mod helpers;
pub use errors::AmmError;

declare_id!("DpvM21fb8QHhdi4wSsSvK2mP5zc5oKqNLVH4QHqdEG2z");

#[program]
pub mod anchor_amm_2023 {
    use super::*;

    pub fn initialize(
        ctx: Context<Initialize>,
        seed: u64,
        fee: u16, // Fee as basis points
        authority: Option<Pubkey> // Update authority (if required)
    ) -> Result<()> {
        // Initialise our AMM config
        ctx.accounts.init(&ctx.bumps, seed, fee, authority)
    }

    pub fn deposit(
        ctx: Context<Deposit>,
        amount: u64, // Amount of LP token to claim
        max_x: u64, // Max amount of X we are willing to deposit
        max_y: u64, // Max amount of Y we are willing to deposit
        expiration: i64,
    ) -> Result<()> {
        // Deposit liquidity to swap
        ctx.accounts.deposit(amount, max_x, max_y, expiration)
    }
}