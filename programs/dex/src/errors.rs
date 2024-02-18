use anchor_lang::prelude::*;

#[error_code]
pub enum DexProgramError {
    #[msg("Insufficient funds to swap")]
    InsufficientFunds,

    #[msg("Duplicate tokens are not allowed")]
    DuplicateTokenNotAllowed,

    #[msg("Failed to add liquidity")]
    FailedToAddLiquidity,

    #[msg("Overflow or underflow occured")]
    OverflowOrUnderflowOccurred,
}
