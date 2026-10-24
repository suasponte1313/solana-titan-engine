use solana_program::program_error::ProgramError;
use thiserror::Error;

#[derive(Error, Debug, Copy, Clone)]
pub enum TitanSwarmError {
    #[error("Invalid Instruction Discriminator")]
    InvalidInstruction = 0,

    #[error("Swarm Drone Execution Mismatch")]
    DroneMismatch = 1,

    #[error("Bonding Curve Price Calculation Overflow")]
    MathOverflow = 2,

    #[error("Atomic Slippage Exceeded — Triggering Jito Error 26 RAM Discard")]
    SlippageExceededDiscard = 26, // 0x1a

    #[error("Mayhem Agent Dump Window Imminent (<3.0s) — Force Abort")]
    MayhemAgentCollision = 30,

    #[error("Drone Rent Sweep Failed")]
    RentSweepFailed = 31,
}

impl From<TitanSwarmError> for ProgramError {
    fn from(e: TitanSwarmError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
