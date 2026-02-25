use anchor_lang::prelude::*;

#[error_code]
pub enum AmmError {
    #[msg("Slippage exceeded")]
    SlippageExceeded,
    #[msg("Invariant violation")]
    InvariantViolation,
    #[msg("Insufficient liquidity")]
    InsufficientLiquidity,
    #[msg("Invalid amount")]
    InvalidAmount,
    #[msg("Pool has no liquidity")]
    PoolEmpty,
    #[msg("Math overflow")]
    Overflow,
}