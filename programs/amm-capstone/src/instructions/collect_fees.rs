use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct CollectFees<'info> {
    pub signer: Signer<'info>,
}