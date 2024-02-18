use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};

use crate::{
    errors::DexProgramError,
    state::{LiquidityPool, LiquidityPoolAccount},
};

pub fn remove_liquidity(ctx: Context<RemoveLiquidity>, shares: u64) -> Result<()> {
    let pool = &mut ctx.accounts.pool;

    let token_one_accounts = (
        &mut ctx.accounts.mint_token_one,
        &mut ctx.accounts.pool_token_account_one,
        &mut ctx.accounts.user_token_account_one,
    );

    let token_two_accounts = (
        &mut ctx.accounts.mint_token_two,
        &mut ctx.accounts.pool_token_account_two,
        &mut ctx.accounts.user_token_account_two,
    );

    pool.remove_liquidity(
        token_one_accounts,
        token_two_accounts,
        shares,
        &ctx.accounts.token_program,
    )?;

    Ok(())
}

#[derive(Accounts)]
pub struct RemoveLiquidity<'info> {
    #[account(
        mut,
        seeds = [LiquidityPool::POOL_SEED_PREFIX.as_bytes(), LiquidityPool::generate_seed(mint_token_one.key(), mint_token_two.key()).as_bytes()],
        bump = pool.bump
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
        mut,
        associated_token::mint = mint_token_one,
        associated_token::authority = pool
    )]
    pub pool_token_account_one: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = mint_token_two,
        associated_token::authority = pool
    )]
    pub pool_token_account_two: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = mint_token_one,
        associated_token::authority = payer,
    )]
    pub user_token_account_one: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = mint_token_two,
        associated_token::authority = payer,
    )]
    pub user_token_account_two: Account<'info, TokenAccount>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}
