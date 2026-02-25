use anchor_lang::prelude::*;
use anchor_spl::token::{
    Token,
    TokenAccount,
};

use crate::state::*;
use crate::constants::*;

#[derive(Accounts)]
pub struct Swap<'info> {

    #[account(mut)]
    pub user: Signer<'info>,

    #[account(mut)]
    pub pool: Account<'info, Pool>,

    #[account(mut)]
    pub user_input: Account<'info, TokenAccount>,

    #[account(mut)]
    pub user_output: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = vault_a.mint == vault_authority.key()
    )]
    pub vault_a: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = vault_b.mint == vault_authority.key()
    )]
    pub vault_b: Account<'info, TokenAccount>,

    /// CHECK: PDA authority
    #[account(
        seeds = [VAULT_AUTH_SEED],
        bump
    )]
    pub vault_authority: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,

    #[account(
        mut,
        seeds = [TREASURY_SEED],
        bump
    )]
    pub treasury: Account<'info, Treasury>,
}