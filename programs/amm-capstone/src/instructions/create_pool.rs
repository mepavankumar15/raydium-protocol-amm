use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount}; // Import TokenAccount
use crate::state::pool::*;
use crate::constants::*;

#[derive(Accounts)]
pub struct CreatePool<'info> {

    #[account(mut)]
    pub payer: Signer<'info>,

    pub token_a_mint: Account<'info, Mint>,
    pub token_b_mint: Account<'info, Mint>,

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

    // --- ADD THIS: Vault A Initialization ---
    #[account(
        init,
        payer = payer,
        seeds = [
            POOL_SEED,
            token_a_mint.key().as_ref(),
            token_b_mint.key().as_ref(),
            b"vault_a" // Must match the seed in your test
        ],
        bump,
        token::mint = token_a_mint,
        token::authority = vault_authority
    )]
    pub vault_a: Account<'info, TokenAccount>,

    // --- ADD THIS: Vault B Initialization ---
    #[account(
        init,
        payer = payer,
        seeds = [
            POOL_SEED,
            token_a_mint.key().as_ref(),
            token_b_mint.key().as_ref(),
            b"vault_b" // Must match the seed in your test
        ],
        bump,
        token::mint = token_b_mint,
        token::authority = vault_authority
    )]
    pub vault_b: Account<'info, TokenAccount>,

    pub lp_mint: Account<'info, Mint>,

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