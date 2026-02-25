use anchor_lang::prelude::*;
use anchor_spl::token::{
     TokenAccount,
      Token,
       Mint, };

use crate::state::*;
use crate::constants::*;

#[derive(Accounts)]
pub struct RemoveLiquidity<'info> {
    pub user: Signer<'info>,

    #[account(mut)]
    pub pool: Account<'info, Pool>,

    #[account(mut)]
    pub user_lp: Account<'info, TokenAccount>,
    #[account(mut)]
    pub lp_mint: Account<'info, Mint>,
    #[account(
        mut,
        constraint = user_token_a.mint == pool.token_a_mint
    )]
    pub user_token_a: Account<'info, TokenAccount>,
    #[account(
        mut,
        constraint = user_token_b.mint == pool.token_b_mint
    )]
    pub user_token_b: Account<'info, TokenAccount>,

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

    #[account(
        seeds = [VAULT_AUTH_SEED],
        bump
    )]

    /// CHECK: This is a PDA authority derived from VAULT_AUTH_SEED.
/// It does not store data and is only used as a signing authority
/// for token vault transfers.

    pub vault_authority: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
}