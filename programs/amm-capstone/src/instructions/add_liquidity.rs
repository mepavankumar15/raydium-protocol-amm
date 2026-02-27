use anchor_lang::prelude::*;
use anchor_spl::token::{
    Token,
    TokenAccount,
    Mint,
    Transfer,
    MintTo,
    transfer,
    mint_to,
};

use crate::state::*;
use crate::constants::*;



#[derive(Accounts)]
pub struct AddLiquidity<'info> {

    #[account(mut)] 
    pub user: Signer<'info>,

    #[account(mut)]
    pub pool: Account<'info, Pool>,

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
        constraint = vault_a.mint == pool.token_a_mint,
        constraint = vault_a.owner == vault_authority.key()
    )]
    pub vault_a: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = vault_b.mint == pool.token_b_mint,
        constraint = vault_b.owner == vault_authority.key()
    )]
    pub vault_b: Account<'info, TokenAccount>,

    #[account(mut)]
    pub lp_mint: Account<'info, Mint>,

    #[account(mut)]
    pub user_lp: Account<'info, TokenAccount>,

    /// CHECK: PDA authority
    #[account(
    seeds = [VAULT_AUTH_SEED],
    bump
    )]
    pub vault_authority: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
}