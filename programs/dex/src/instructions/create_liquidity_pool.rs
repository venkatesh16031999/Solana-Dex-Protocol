use crate::{errors::DexProgramError, state::*};
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};

pub fn create_liquidity_pool(ctx: Context<CreateLiquidityPool>) -> Result<()> {
    let pool = &mut ctx.accounts.pool;

    pool.set_inner(LiquidityPool::new(
        ctx.accounts.mint_token_one.key(),
        ctx.accounts.mint_token_two.key(),
        ctx.bumps.pool,
    ));

    Ok(())
}

#[derive(Accounts)]
pub struct CreateLiquidityPool<'info> {
    #[account(
        init,
        space = LiquidityPool::ACCOUNT_SIZE,
        payer = payer,
        seeds = [LiquidityPool::POOL_SEED_PREFIX.as_bytes(), LiquidityPool::generate_seed(mint_token_one.key(), mint_token_two.key()).as_bytes()],
        bump
    )]
    pub pool: Account<'info, LiquidityPool>,

    #[account(
        constraint = !mint_token_one.key().eq(&mint_token_two.key()) @ DexProgramError::DuplicateTokenNotAllowed
    )]
    pub mint_token_one: Account<'info, Mint>,

    #[account(
        constraint = !mint_token_one.key().eq(&mint_token_two.key()) @ DexProgramError::DuplicateTokenNotAllowed
    )]
    pub mint_token_two: Account<'info, Mint>,

    #[account(
        init,
        payer = payer,
        associated_token::mint = mint_token_one,
        associated_token::authority = pool
    )]
    pub pool_token_account_one: Account<'info, TokenAccount>,

    #[account(
        init,
        payer = payer,
        associated_token::mint = mint_token_two,
        associated_token::authority = pool
    )]
    pub pool_token_account_two: Account<'info, TokenAccount>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}
