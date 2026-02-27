// programs/amm-capstone/src/state/pool.rs

use anchor_lang::prelude::*;

#[account]
pub struct Pool {
    pub authority: Pubkey,      // 32
    pub token_a_mint: Pubkey,   // 32
    pub token_b_mint: Pubkey,   // 32
    
    // --- ENSURE THESE EXIST ---
    pub vault_a: Pubkey,        // 32
    pub vault_b: Pubkey,        // 32
    // --------------------------

    pub vault_authority: Pubkey,// 32
    pub lp_mint: Pubkey,        // 32
    pub reserve_a: u64,         // 8
    pub reserve_b: u64,         // 8
    pub fee_bps: u16,           // 2
}

impl Pool {
    // 8 discriminator + 32*6 + 8*2 + 2 = 210 bytes (approx)
    // Ensure you have enough space. A safe buffer is usually:
    pub const LEN: usize = 8 + 32 + 32 + 32 + 32 + 32 + 32 + 8 + 8 + 2 + 64; 
}