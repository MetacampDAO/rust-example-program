use solana_program::{program_error::ProgramError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum OnchainAccountError {
  #[error("Invalid seeds for PDA")]
  InvalidPDA,

  #[error("Account is not initialized")]
  UninitializedAccount,

  #[error("Invalid rating")]
  InvalidRating,

  #[error("Invalid data length")]
  InvalidDataLength,

  #[error("Accounts do not match")]
  IncorrectAccountError,

  /// Invalid instruction data passed in.
  #[error("Failed to unpack instruction data")]
  InstructionUnpackError,

}

impl From<OnchainAccountError> for ProgramError {
    fn from(e: OnchainAccountError) -> Self {
        ProgramError::Custom(e as u32)
    }
}