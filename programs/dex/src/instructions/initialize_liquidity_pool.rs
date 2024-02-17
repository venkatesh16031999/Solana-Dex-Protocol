use anchor_lang::prelude::*;

use crate::state::*;

pub fn initialize_liquidity_pool(ctx: Context<InitializeLiquidityPool>) -> Result<()> {
    let pool = &mut ctx.accounts.pool;

    pool.set_inner(LiquidityPool::new(ctx.bumps.pool));

    Ok(())
}

#[derive(Accounts)]
pub struct InitializeLiquidityPool<'info> {
    #[account(
        init,
        space = LiquidityPool::INITIAL_ACCOUNT_SIZE,
        payer = payer,
        seeds = [LiquidityPool::POOL_SEED_PREFIX.as_bytes()],
        bump
    )]
    pub pool: Account<'info, LiquidityPool>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub system_program: Program<'info, System>,
}
