use anchor_lang::prelude::*;

#[account]
pub struct Treasury {
    pub authority: Pubkey,
    pub total_fees_collected: u64,
}

impl Treasury {
    pub const LEN: usize = 32 + 8;
}