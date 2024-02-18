use crate::errors::DexProgramError;
use crate::helpers::convert_to_float;
use anchor_lang::prelude::*;
use anchor_lang::system_program;
use anchor_spl::token::{self, Mint, Token, TokenAccount};
use std::cmp;
use std::ops::Mul;

#[account]
pub struct LiquidityPool {
    pub token_one: Pubkey,
    pub token_two: Pubkey,
    pub shares: u64,
    pub total_supply: u64,
    pub reserve_one: u64,
    pub reserve_two: u64,
    pub bump: u8,
}

impl LiquidityPool {
    pub const POOL_SEED_PREFIX: &'static str = "liquidity_pool";

    // Discriminator (8) + Pubkey (32) + Pubkey (32) + Shares (8) + Bump (1)
    pub const ACCOUNT_SIZE: usize = 8 + 32 + 32 + 8 + 1;

    pub fn generate_seed(token_one: Pubkey, token_two: Pubkey) -> String {
        if token_one > token_two {
            format!("{}{}", token_one.to_string(), token_two.to_string())
        } else {
            format!("{}{}", token_two.to_string(), token_one.to_string())
        }
    }

    pub fn new(token_one: Pubkey, token_two: Pubkey, bump: u8) -> Self {
        Self {
            token_one: token_one,
            token_two: token_two,
            shares: 0_u64,
            total_supply: 0_u64,
            reserve_one: 0_u64,
            reserve_two: 0_u64,
            bump: bump,
        }
    }
}

pub trait LiquidityPoolAccount<'info> {
    fn grant_shares(&mut self, shares: u64) -> Result<()>;
    fn remove_shares(&mut self, shares: u64) -> Result<()>;
    fn update_reserves(&mut self, reserve_one: u64, reserve_two: u64) -> Result<()>;

    fn add_liquidity(
        &mut self,
        token_one_accounts: (
            &mut Account<'info, Mint>,
            &mut Account<'info, TokenAccount>,
            &mut Account<'info, TokenAccount>,
        ),
        token_two_accounts: (
            &mut Account<'info, Mint>,
            &mut Account<'info, TokenAccount>,
            &mut Account<'info, TokenAccount>,
        ),
        amount_one: u64,
        amount_two: u64,
        authority: &Signer<'info>,
        token_program: &Program<'info, Token>,
    ) -> Result<()>;

    fn transfer_token_from_pool(
        &self,
        from: &Account<'info, TokenAccount>,
        to: &Account<'info, TokenAccount>,
        amount: u64,
        token_program: &Program<'info, Token>,
    ) -> Result<()>;

    fn transfer_token_to_pool(
        &self,
        from: &Account<'info, TokenAccount>,
        to: &Account<'info, TokenAccount>,
        amount: u64,
        authority: &Signer<'info>,
        token_program: &Program<'info, Token>,
    ) -> Result<()>;

    fn transfer_sol_to_pool(
        &self,
        from: &Signer<'info>,
        amount: u64,
        system_program: &Program<'info, System>,
    ) -> Result<()>;

    fn transfer_sol_from_pool(
        &self,
        to: &AccountInfo<'info>,
        amount: u64,
        system_program: &Program<'info, System>,
    ) -> Result<()>;
}

impl<'info> LiquidityPoolAccount<'info> for Account<'info, LiquidityPool> {
    fn grant_shares(&mut self, shares: u64) -> Result<()> {
        self.shares = self
            .shares
            .checked_sub(shares)
            .ok_or(DexProgramError::OverflowOrUnderflowOccurred)?;

        self.total_supply = self
            .total_supply
            .checked_sub(shares)
            .ok_or(DexProgramError::OverflowOrUnderflowOccurred)?;

        Ok(())
    }

    fn remove_shares(&mut self, shares: u64) -> Result<()> {
        self.shares = self
            .shares
            .checked_sub(shares)
            .ok_or(DexProgramError::OverflowOrUnderflowOccurred)?;

        self.total_supply = self
            .total_supply
            .checked_sub(shares)
            .ok_or(DexProgramError::OverflowOrUnderflowOccurred)?;

        Ok(())
    }

    fn update_reserves(&mut self, reserve_one: u64, reserve_two: u64) -> Result<()> {
        self.reserve_one = reserve_one;
        self.reserve_two = reserve_two;

        Ok(())
    }

    fn add_liquidity(
        &mut self,
        token_one_accounts: (
            &mut Account<'info, Mint>,
            &mut Account<'info, TokenAccount>,
            &mut Account<'info, TokenAccount>,
        ),
        token_two_accounts: (
            &mut Account<'info, Mint>,
            &mut Account<'info, TokenAccount>,
            &mut Account<'info, TokenAccount>,
        ),
        amount_one: u64,
        amount_two: u64,
        authority: &Signer<'info>,
        token_program: &Program<'info, Token>,
    ) -> Result<()> {
        let mut shares_to_allocate = 0_u64;

        if self.total_supply == 0 {
            let sqrt_shares = (convert_to_float(amount_one, token_one_accounts.0.decimals)
                .mul(convert_to_float(amount_two, token_two_accounts.0.decimals)))
            .sqrt();

            shares_to_allocate = sqrt_shares as u64;
        } else {
            let mul_value = amount_one
                .checked_mul(self.total_supply)
                .ok_or(DexProgramError::OverflowOrUnderflowOccurred)?;
            let shares_one = mul_value
                .checked_div(self.reserve_one)
                .ok_or(DexProgramError::OverflowOrUnderflowOccurred)?;

            let mul_value = amount_two
                .checked_mul(self.total_supply)
                .ok_or(DexProgramError::OverflowOrUnderflowOccurred)?;
            let shares_two = mul_value
                .checked_div(self.reserve_two)
                .ok_or(DexProgramError::OverflowOrUnderflowOccurred)?;

            shares_to_allocate = cmp::min(shares_one, shares_two);
        }

        if shares_to_allocate <= 0 {
            return err!(DexProgramError::FailedToAddLiquidity);
        }

        self.grant_shares(shares_to_allocate)?;

        let new_reserves_one = self
            .reserve_one
            .checked_add(amount_one)
            .ok_or(DexProgramError::OverflowOrUnderflowOccurred)?;
        let new_reserves_two = self
            .reserve_two
            .checked_add(amount_two)
            .ok_or(DexProgramError::OverflowOrUnderflowOccurred)?;

        self.update_reserves(new_reserves_one, new_reserves_two)?;

        self.transfer_token_to_pool(
            token_one_accounts.2,
            token_one_accounts.1,
            amount_one,
            authority,
            token_program,
        )?;

        self.transfer_token_to_pool(
            token_two_accounts.2,
            token_two_accounts.1,
            amount_two,
            authority,
            token_program,
        )?;

        Ok(())
    }

    fn transfer_token_from_pool(
        &self,
        from: &Account<'info, TokenAccount>,
        to: &Account<'info, TokenAccount>,
        amount: u64,
        token_program: &Program<'info, Token>,
    ) -> Result<()> {
        token::transfer(
            CpiContext::new_with_signer(
                token_program.to_account_info(),
                token::Transfer {
                    from: from.to_account_info(),
                    to: to.to_account_info(),
                    authority: self.to_account_info(),
                },
                &[&[
                    LiquidityPool::POOL_SEED_PREFIX.as_bytes(),
                    LiquidityPool::generate_seed(self.token_one.key(), self.token_two.key())
                        .as_bytes(),
                    &[self.bump],
                ]],
            ),
            amount,
        )?;

        Ok(())
    }

    fn transfer_token_to_pool(
        &self,
        from: &Account<'info, TokenAccount>,
        to: &Account<'info, TokenAccount>,
        amount: u64,
        authority: &Signer<'info>,
        token_program: &Program<'info, Token>,
    ) -> Result<()> {
        token::transfer(
            CpiContext::new(
                token_program.to_account_info(),
                token::Transfer {
                    from: from.to_account_info(),
                    to: to.to_account_info(),
                    authority: authority.to_account_info(),
                },
            ),
            amount,
        )?;

        Ok(())
    }

    fn transfer_sol_from_pool(
        &self,
        to: &AccountInfo<'info>,
        amount: u64,
        system_program: &Program<'info, System>,
    ) -> Result<()> {
        let pool_account_info = self.to_account_info();

        system_program::transfer(
            CpiContext::new_with_signer(
                system_program.to_account_info(),
                system_program::Transfer {
                    from: pool_account_info,
                    to: to.clone(),
                },
                &[&[
                    LiquidityPool::POOL_SEED_PREFIX.as_bytes(),
                    LiquidityPool::generate_seed(self.token_one.key(), self.token_two.key())
                        .as_bytes(),
                    &[self.bump],
                ]],
            ),
            amount,
        )?;

        Ok(())
    }

    fn transfer_sol_to_pool(
        &self,
        from: &Signer<'info>,
        amount: u64,
        system_program: &Program<'info, System>,
    ) -> Result<()> {
        let pool_account_info = self.to_account_info();

        system_program::transfer(
            CpiContext::new(
                system_program.to_account_info(),
                system_program::Transfer {
                    from: from.to_account_info(),
                    to: pool_account_info,
                },
            ),
            amount,
        )?;
        Ok(())
    }
}
