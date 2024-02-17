use anchor_lang::prelude::*;
use anchor_lang::system_program;
use anchor_spl::token::{self, Mint, Token, TokenAccount};

#[account]
pub struct LiquidityPool {
    pub assets: Vec<Pubkey>,
    pub bump: u8,
}

impl LiquidityPool {
    pub const POOL_SEED_PREFIX: &'static str = "liquidity_pool";

    pub const INITIAL_ACCOUNT_SIZE: usize = 8 + 4 + 1;

    pub fn new(bump: u8) -> Self {
        Self {
            assets: Vec::new(),
            bump: bump,
        }
    }
}

pub trait LiquidityPoolAccount<'info> {
    fn add_asset(&mut self, asset: Pubkey);

    fn is_asset_exists(&self, key: Pubkey) -> bool;

    fn reallocate(&mut self, new_account_size: usize) -> Result<()>;

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

    fn add_liquidity(
        &mut self,
        deposit_mint_account: &Account<'info, Mint>,
        pool_token_account: &Account<'info, TokenAccount>,
        payer_token_account: &Account<'info, TokenAccount>,
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
    fn add_asset(&mut self, asset: Pubkey) {
        self.assets.push(asset);
    }

    fn add_liquidity(
        &mut self,
        deposit_mint_account: &Account<'info, Mint>,
        pool_token_account: &Account<'info, TokenAccount>,
        payer_token_account: &Account<'info, TokenAccount>,
        amount: u64,
        authority: &Signer<'info>,
        token_program: &Program<'info, Token>,
    ) -> Result<()> {
        if !self.is_asset_exists(deposit_mint_account.key()) {
            self.add_asset(deposit_mint_account.key());
        }

        self.transfer_token_to_pool(
            payer_token_account,
            pool_token_account,
            amount,
            authority,
            token_program,
        )?;
        Ok(())
    }

    fn is_asset_exists(&self, key: Pubkey) -> bool {
        if self.assets.contains(&key) {
            true
        } else {
            false
        }
    }

    fn reallocate(&mut self, new_account_size: usize) -> Result<()> {
        let account_info = self.to_account_info();

        account_info.realloc(new_account_size, false)?;
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
                &[&[LiquidityPool::POOL_SEED_PREFIX.as_bytes(), &[self.bump]]],
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
                &[&[LiquidityPool::POOL_SEED_PREFIX.as_bytes(), &[self.bump]]]
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
