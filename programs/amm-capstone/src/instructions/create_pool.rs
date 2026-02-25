use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token};
use crate::state::pool::*;
use crate::constants::*;

#[derive(Accounts)]
pub struct CreatePool<'info> {

    #[account(mut)]
    pub payer: Signer<'info>,

    /// Token A mint
    pub token_a_mint: Account<'info, Mint>,

    /// Token B mint
    pub token_b_mint: Account<'info, Mint>,

    /// Pool PDA
    #[account(
        init,
        payer = payer,
        seeds = [
            POOL_SEED,
            token_a_mint.key().as_ref(),
            token_b_mint.key().as_ref()
        ],
        bump,
        space = 8 + Pool::LEN
    )]
    pub pool: Account<'info, Pool>,

    /// LP token mint
    #[account(
        init,
        payer = payer,
        mint::decimals = 6,
        mint::authority = vault_authority
    )]
    pub lp_mint: Account<'info, Mint>,

    /// PDA authority controlling vaults
    /// CHECK: PDA authority
    #[account(
        seeds = [VAULT_AUTH_SEED],
        bump
    )]
    pub vault_authority: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}