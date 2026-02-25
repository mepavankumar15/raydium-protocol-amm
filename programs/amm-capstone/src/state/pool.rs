use anchor_lang::prelude::*;

#[account]
pub struct Pool {
    pub token_a_mint: Pubkey,
    pub token_b_mint: Pubkey,

    pub vault_authority: Pubkey,
    pub vault_a: Pubkey,
    pub vault_b: Pubkey,
    pub lp_mint: Pubkey,

    pub reserve_a: u64,
    pub reserve_b: u64,

    pub fee_bps: u16,
}

impl Pool {
    pub const LEN: usize = 32 + 32 + 32 + 32 + 32 + 32 + 8 + 8 + 2;
    // token A
    // token B
    // authority
    // vault A
    // vault B
    // lp mint
    // reserve A
    // reserve B
    // fee
}