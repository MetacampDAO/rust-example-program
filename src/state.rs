use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{program_pack::{IsInitialized, Sealed}, pubkey::Pubkey};

#[derive(BorshSerialize, BorshDeserialize)]
pub struct OnchainAccountState {
    pub is_initialized: bool,
    pub id: u8,
    pub name: String,
    pub creator: Pubkey
}

impl Sealed for OnchainAccountState {}

impl IsInitialized for OnchainAccountState {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}
