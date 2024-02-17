use anchor_lang::prelude::*;

#[error_code]
pub enum DexProgramError {
    #[msg("Insufficient funds to swap")]
    InsufficientFunds,
}
