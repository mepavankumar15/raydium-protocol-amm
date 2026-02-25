use anchor_lang::prelude::*;
use anchor_spl::token::{
    transfer,
    mint_to,
    Transfer,
    MintTo,
};

pub mod instructions;
pub mod state;
pub mod math;
pub mod errors;
pub mod constants;
pub mod events;
pub mod utils;

use instructions::*;
use anchor_spl::token::{Burn , burn};
use crate::constants::*;
use crate::math::*;
use crate::events::*;
use errors::AmmError;

declare_id!("F6SEFWxhBryxMrCwdD9jzaL2MZe6dtZBqzEwCRFsXodf");

#[program]
pub mod amm_capstone {
    use super::*;

    // ------------------------------------------------
    // CREATE POOL
    // ------------------------------------------------
    pub fn create_pool(ctx: Context<CreatePool>) -> Result<()> {
        let pool = &mut ctx.accounts.pool;

        pool.token_a_mint = ctx.accounts.token_a_mint.key();
        pool.token_b_mint = ctx.accounts.token_b_mint.key();
        pool.vault_authority = ctx.accounts.vault_authority.key();
        pool.lp_mint = ctx.accounts.lp_mint.key();

        pool.reserve_a = 0;
        pool.reserve_b = 0;
        pool.fee_bps = 30; // 0.3%

        Ok(())
    }

    // ------------------------------------------------
    // ADD LIQUIDITY
    // ------------------------------------------------
    pub fn add_liquidity(
        ctx: Context<AddLiquidity>,
        amount_a: u64,
        amount_b: u64,
    ) -> Result<()> {

        let pool = &mut ctx.accounts.pool;
        require!(amount_a > 0, AmmError::InvalidAmount);
        require!(amount_b > 0, AmmError::InvalidAmount);

        if pool.reserve_a > 0 && pool.reserve_b > 0 {
        let expected_b =
            (amount_a as u128 * pool.reserve_b as u128)
            / pool.reserve_a as u128;

        require!(
            amount_b as u128 >= expected_b,
            AmmError::SlippageExceeded
            );
        }
        // Transfer Token A
        transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.user_token_a.to_account_info(),
                    to: ctx.accounts.vault_a.to_account_info(),
                    authority: ctx.accounts.user.to_account_info(),
                },
            ),
            amount_a,
        )?;

        // Transfer Token B
        transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.user_token_b.to_account_info(),
                    to: ctx.accounts.vault_b.to_account_info(),
                    authority: ctx.accounts.user.to_account_info(),
                },
            ),
            amount_b,
        )?;

        // PDA signer
        let bump = ctx.bumps.vault_authority;

        let signer_seeds: &[&[&[u8]]] = &[&[
            VAULT_AUTH_SEED,
            &[bump],
        ]];

        // Mint LP tokens
        mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                MintTo {
                    mint: ctx.accounts.lp_mint.to_account_info(),
                    to: ctx.accounts.user_lp.to_account_info(),
                    authority: ctx.accounts.vault_authority.to_account_info(),
                },
                signer_seeds,
            ),
            amount_a.min(amount_b),
        )?;

        pool.reserve_a += amount_a;
        pool.reserve_b += amount_b;

        Ok(())
    }

    // ------------------------------------------------
    // SWAP (CORE AMM ENGINE)
    // ------------------------------------------------
    pub fn swap(
        ctx: Context<Swap>,
        amount_in: u64,
        min_out: u64,
    ) -> Result<()> {

    let pool = &mut ctx.accounts.pool;

    // ---------------- Pool Safety ----------------
    require!(
        pool.reserve_a > 0 && pool.reserve_b > 0,
        AmmError::PoolEmpty
    );

    require!(amount_in > 0, AmmError::InvalidAmount);

    // ---------------- Fee Split ----------------
    let total_fee =
        amount_in * pool.fee_bps as u64 / 10_000;

    let protocol_fee = total_fee / 5;
    let lp_fee = total_fee - protocol_fee;

    let effective_input = amount_in - total_fee;

    // ---------------- Price Calculation ----------------
    let amount_out = get_amount_out(
        effective_input,
        pool.reserve_a,
        pool.reserve_b,
        0,
    );

    require!(
        amount_out >= min_out,
        AmmError::SlippageExceeded
    );

    // ---------------- Transfer input ----------------
    transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.user_input.to_account_info(),
                to: ctx.accounts.vault_a.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
            },
        ),
        amount_in,
    )?;

    // ---------------- PDA signer ----------------
    let bump = ctx.bumps.vault_authority;

    let signer_seeds: &[&[&[u8]]] = &[&[
        VAULT_AUTH_SEED,
        &[bump],
    ]];

    // ---------------- Transfer output ----------------
    transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.vault_b.to_account_info(),
                to: ctx.accounts.user_output.to_account_info(),
                authority: ctx.accounts.vault_authority.to_account_info(),
            },
            signer_seeds,
        ),
        amount_out,
    )?;

    // ---------------- Invariant Check ----------------
    let old_k =
        (pool.reserve_a as u128) *
        (pool.reserve_b as u128);

    let new_reserve_a = pool.reserve_a
        .checked_add(effective_input + lp_fee)
        .ok_or(AmmError::Overflow)?;

    let new_reserve_b = pool.reserve_b
        .checked_sub(amount_out)
        .ok_or(AmmError::Overflow)?;

    let new_k =
        (new_reserve_a as u128) *
        (new_reserve_b as u128);

    require!(
        new_k >= old_k,
        AmmError::InvariantViolation
    );

    // ---------------- Update Reserves ----------------
    pool.reserve_a = new_reserve_a;
    pool.reserve_b = new_reserve_b;

    // ---------------- Treasury Accounting ----------------
    ctx.accounts.treasury.total_fees_collected =
        ctx.accounts.treasury.total_fees_collected
            .checked_add(protocol_fee)
            .ok_or(AmmError::Overflow)?;

    // ---------------- Emit Event ----------------
    emit!(SwapEvent {
        user: ctx.accounts.user.key(),
        amount_in,
        amount_out,
    });

        Ok(())
    }

    // ------------------------------------------------
    pub fn remove_liquidity(
    ctx: Context<RemoveLiquidity>,
    lp_amount: u64, 
    ) -> Result<()> {

    let pool = &mut ctx.accounts.pool;

    let total_lp_supply = ctx.accounts.lp_mint.supply;

    require!(
        lp_amount <= ctx.accounts.user_lp.amount,
         AmmError::InvalidAmount
        );

    // -----------------------------
    // Calculate proportional share
    // -----------------------------
    let amount_a =
        (pool.reserve_a as u128 * lp_amount as u128
            / total_lp_supply as u128) as u64;

    let amount_b =
        (pool.reserve_b as u128 * lp_amount as u128
            / total_lp_supply as u128) as u64;

    // -----------------------------
    // Burn LP tokens
    // -----------------------------
    burn(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Burn {
                mint: ctx.accounts.lp_mint.to_account_info(),
                from: ctx.accounts.user_lp.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
            },
        ),
        lp_amount,
    )?;

    // -----------------------------
    // PDA signer
    // -----------------------------
    let bump = ctx.bumps.vault_authority;

    let signer_seeds: &[&[&[u8]]] = &[&[
        VAULT_AUTH_SEED,
        &[bump],
    ]];

    // -----------------------------
    // Transfer Token A back
    // -----------------------------
    transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.vault_a.to_account_info(),
                to: ctx.accounts.user_token_a.to_account_info(),
                authority: ctx.accounts.vault_authority.to_account_info(),
            },
            signer_seeds,
        ),
        amount_a,
    )?;

    // -----------------------------
    // Transfer Token B back
    // -----------------------------
    transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.vault_b.to_account_info(),
                to: ctx.accounts.user_token_b.to_account_info(),
                authority: ctx.accounts.vault_authority.to_account_info(),
            },
            signer_seeds,
        ),
        amount_b,
    )?;

    // -----------------------------
    // Update reserves
    // -----------------------------
    pool.reserve_a -= amount_a;
    pool.reserve_b -= amount_b;

    Ok(())
}

    // ------------------------------------------------
    pub fn collect_fees(_ctx: Context<CollectFees>) -> Result<()> {
        Ok(())
    }

    // ------------------------------------------------
    pub fn init_treasury(ctx: Context<InitTreasury>) -> Result<()> {
        let treasury = &mut ctx.accounts.treasury;
        treasury.authority = ctx.accounts.payer.key();
        treasury.total_fees_collected = 0;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}