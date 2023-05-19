use std::convert::TryInto;

use borsh::BorshDeserialize;
use solana_program::{
    program_error::ProgramError,
    msg, pubkey::{Pubkey, PUBKEY_BYTES}
};

use crate::{
    error::OnchainAccountError
};

pub enum OnchainAccountInstruction {
    CreateOnchainAccount {
        id: u8,
        name: String,
    },
    Hello,
}

#[derive(BorshDeserialize)]
struct OnchainAccountPayload {
    id: u8,
    name: String,
}

impl OnchainAccountInstruction {
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {

        let (&variant, rest) = input
            .split_first()
            .ok_or(ProgramError::InvalidInstructionData)?;
        
        Ok(match variant {

            0 => Self::Hello,


            // Create OnChainAccount
            1 => {
                let payload = OnchainAccountPayload::try_from_slice(rest).unwrap();
                Self::CreateOnchainAccount {
                id: payload.id,
                name: payload.name,
                }
            },
            
            _ => return Err(ProgramError::InvalidInstructionData),

        })
    }


    fn unpack_u64(input: &[u8]) -> Result<(u64, &[u8]), ProgramError> {
        if input.len() < 8 {
            msg!("u64 cannot be unpacked");
            return Err(OnchainAccountError::InstructionUnpackError.into());
        }
        let (bytes, rest) = input.split_at(8);
        let value = bytes
            .get(..8)
            .and_then(|slice| slice.try_into().ok())
            .map(u64::from_le_bytes)
            .ok_or(OnchainAccountError::InstructionUnpackError)?;
        Ok((value, rest))
    }

    fn unpack_u8(input: &[u8]) -> Result<(u8, &[u8]), ProgramError> {
        if input.is_empty() {
            msg!("u8 cannot be unpacked");
            return Err(OnchainAccountError::InstructionUnpackError.into());
        }
        let (bytes, rest) = input.split_at(1);
        let value = bytes
            .get(..1)
            .and_then(|slice| slice.try_into().ok())
            .map(u8::from_le_bytes)
            .ok_or(OnchainAccountError::InstructionUnpackError)?;
        Ok((value, rest))
    }

    fn unpack_bytes32(input: &[u8]) -> Result<(&[u8; 32], &[u8]), ProgramError> {
        if input.len() < 32 {
            msg!("32 bytes cannot be unpacked");
            return Err(OnchainAccountError::InstructionUnpackError.into());
        }
        let (bytes, rest) = input.split_at(32);
        Ok((
            bytes
                .try_into()
                .map_err(|_| OnchainAccountError::InstructionUnpackError)?,
            rest,
        ))
    }

    fn unpack_pubkey(input: &[u8]) -> Result<(Pubkey, &[u8]), ProgramError> {
        if input.len() < PUBKEY_BYTES {
            msg!("Pubkey cannot be unpacked");
            return Err(OnchainAccountError::InstructionUnpackError.into());
        }
        let (key, rest) = input.split_at(PUBKEY_BYTES);
        let pk = Pubkey::new(key);
        Ok((pk, rest))
    }
}
