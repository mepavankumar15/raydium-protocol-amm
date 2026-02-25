use anchor_lang::prelude::*;
use crate::state::*;
use crate::constants::*;

#[derive(Accounts)]
pub struct InitTreasury<'info> {

    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        init,
        payer = payer,
        seeds = [TREASURY_SEED],
        bump,
        space = 8 + Treasury::LEN
    )]
    pub treasury: Account<'info, Treasury>,

    pub system_program: Program<'info, System>,
}