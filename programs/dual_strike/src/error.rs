use solana_program::program_error::ProgramError;
use thiserror::Error;

#[derive(Error, Debug, Copy, Clone)]
pub enum DualStrikeError {
    #[error("Invalid Instruction Discriminator")]
    InvalidInstruction = 0,

    #[error("Insufficient Funds for Atomic Swap")]
    InsufficientFunds = 1,

    #[error("Bonding Curve Slippage Exceeded — Triggering Jito Error 26 RAM Discard")]
    SlippageExceededDiscard = 26, // 0x1a - Custom error 26 that causes Jito bundle to drop in RAM

    #[error("Unauthorized Signer")]
    UnauthorizedSigner = 27,

    #[error("Hard Time Stop Liquidation Triggered")]
    TimeStopLiquidation = 28,
}

impl From<DualStrikeError> for ProgramError {
    fn from(e: DualStrikeError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
