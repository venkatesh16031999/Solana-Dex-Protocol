use anchor_lang::prelude::*;

pub mod errors;
pub mod instructions;
pub mod state;

use crate::instructions::*;

declare_id!("HHtpy5cez4guhvwoXVCZzo8EUce6ouJyXaxZ7r9CVR24");

#[program]
pub mod dex {
    use super::*;

    pub fn initialize_liquidity_pool(ctx: Context<InitializeLiquidityPool>) -> Result<()> {
        instructions::initialize_liquidity_pool(ctx)
    }

    pub fn add_liquidity(ctx: Context<AddLiquidity>) -> Result<()> {
        instructions::add_liquidity(ctx)
    }

    pub fn remove_liquidity(ctx: Context<RemoveLiquidity>) -> Result<()> {
        instructions::remove_liquidity(ctx)
    }

    pub fn swap(ctx: Context<Swap>) -> Result<()> {
        instructions::swap(ctx)
    }
}
