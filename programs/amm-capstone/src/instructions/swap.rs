use anchor_lang::prelude::*;
use anchor_spl::token::{
    self,
    Token,
    TokenAccount,
    Transfer,
};

use crate::state::*;
use crate::constants::*;
use crate::errors::AmmError as ErrorCode;

#[derive(Accounts)]
pub struct Swap<'info> {

    #[account(mut)]
    pub user: Signer<'info>,

    #[account(mut)]
    pub pool: Account<'info, Pool>,

    #[account(
        mut,
        constraint = user_input.owner == user.key(),
    )]
    pub user_input: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = user_output.owner == user.key(),
    )]
    pub user_output: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = vault_a.key() == pool.vault_a,
        constraint = vault_a.owner == vault_authority.key(),
        constraint = vault_a.mint == pool.token_a_mint
    )]
    pub vault_a: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = vault_b.key() == pool.vault_b,
        constraint = vault_b.owner == vault_authority.key(),
        constraint = vault_b.mint == pool.token_b_mint
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

pub fn handler(
    ctx: Context<Swap>,
    amount_in: u64,
    min_amount_out: u64,
) -> Result<()> {

    require!(amount_in > 0, ErrorCode::InvalidAmount);

    let pool = &mut ctx.accounts.pool;

    // Detect direction
    let is_a_to_b = ctx.accounts.user_input.mint == pool.token_a_mint;

    let (reserve_in, reserve_out) = if is_a_to_b {
        (pool.reserve_a, pool.reserve_b)
    } else {
        (pool.reserve_b, pool.reserve_a)
    };

    // ---- Fee Calculation ----
    let fee = (amount_in as u128)
        .checked_mul(pool.fee_bps as u128)
        .unwrap()
        / 10_000;

    let amount_in_after_fee = amount_in - fee as u64;

    // ---- Constant Product ----
    let k = (reserve_in as u128)
        .checked_mul(reserve_out as u128)
        .unwrap();

    let new_reserve_in = reserve_in + amount_in_after_fee;

    let new_reserve_out =
        k.checked_div(new_reserve_in as u128).unwrap() as u64;

    let amount_out = reserve_out - new_reserve_out;

    require!(amount_out >= min_amount_out, ErrorCode::SlippageExceeded);

    // ---- Transfer Input (User → Vault) ----
    let cpi_ctx_in = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.user_input.to_account_info(),
            to: if is_a_to_b {
                ctx.accounts.vault_a.to_account_info()
            } else {
                ctx.accounts.vault_b.to_account_info()
            },
            authority: ctx.accounts.user.to_account_info(),
        },
    );

    token::transfer(cpi_ctx_in, amount_in)?;

    let bump = ctx.bumps.vault_authority;
    let bump_seed = [bump];
    // ---- Transfer Output (Vault → User) ----
    let seeds: &[&[u8]] = &[VAULT_AUTH_SEED, &bump_seed];
    let signer_seeds = &[seeds];

    let cpi_ctx_out = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: if is_a_to_b {
                ctx.accounts.vault_b.to_account_info()
            } else {
                ctx.accounts.vault_a.to_account_info()
            },
            to: ctx.accounts.user_output.to_account_info(),
            authority: ctx.accounts.vault_authority.to_account_info(),
        },
        signer_seeds,
    );

    token::transfer(cpi_ctx_out, amount_out)?;

    // ---- Update Reserves ----
    if is_a_to_b {
        pool.reserve_a += amount_in_after_fee;
        pool.reserve_b -= amount_out;
    } else {
        pool.reserve_b += amount_in_after_fee;
        pool.reserve_a -= amount_out;
    }

    Ok(())
}