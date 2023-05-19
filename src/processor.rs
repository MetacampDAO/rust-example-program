use solana_program::{
    entrypoint::ProgramResult,
    pubkey::Pubkey,
    msg,
    account_info::{next_account_info, AccountInfo},
    system_instruction,
    sysvar::{
      rent::Rent,
      Sysvar,
    },
    program::{invoke_signed},
    borsh::try_from_slice_unchecked,
    program_error::ProgramError,
};


use std::convert::TryInto;
use borsh::BorshSerialize;

use crate::{
  instruction::OnchainAccountInstruction,
  state::OnchainAccountState,
  error::OnchainAccountError,
};

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8]
  ) -> ProgramResult {

    let instruction = OnchainAccountInstruction::unpack(instruction_data)?;

    match instruction {

      // Just say hello :)
      OnchainAccountInstruction::Hello{} => {
        msg!("Instruction: Hello");
        process_hello_world()
      }

      // Create on-chain account
      OnchainAccountInstruction::CreateOnchainAccount { id, name} => {
        msg!("Instruction: Add Onchain Account");
        process_create_onchain_account(program_id, accounts, id, name)
      }
      
    }
  }

  pub fn process_hello_world(

  ) -> ProgramResult {
    msg!("Hello Solana!");
    Ok(())
  }
  

  pub fn process_create_onchain_account(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    id: u8,
    name: String,
  ) -> ProgramResult {
    msg!("Creating Onchain Account...");
    msg!("Id: {}", id.to_string());
    msg!("Name: {}", name);

    let account_info_iter = &mut accounts.iter();

    let initializer = next_account_info(account_info_iter)?;
    let pda_account = next_account_info(account_info_iter)?;
    let system_program = next_account_info(account_info_iter)?;

    // Checking if initializer is signer
    if !initializer.is_signer {
      msg!("Missing required signature");
    return Err(ProgramError::MissingRequiredSignature)
    }

    let (pda, bump_seed) = Pubkey::find_program_address(&[initializer.key.as_ref(), id.to_le_bytes().as_ref()], program_id);
    

    let id_seed = &id.to_le_bytes();
    msg!("Id Seed: {:?}", &id_seed);
    msg!("Derived PDA: {}", pda);
    msg!("Passed PDA: {}", pda_account.key);

    // Checking if right pda is passed
    if pda != *pda_account.key {
      msg!("Invalid seeds for PDA");
      return Err(ProgramError::InvalidArgument)
    }

    // Checking if rating is a valid number
    if id > 15 || id < 0 {
      msg!("Invalid Id");
      return Err(OnchainAccountError::InvalidRating.into())
    }


    let account_len = 1000;
    // Checking if account data is within accepted length
    let total_len: usize = 1 + 1 + (4 + name.len()) + 32;
    if total_len > 1000 {
      msg!("Data length is larger than 1000 bytes");
      return Err(OnchainAccountError::InvalidDataLength.into())
    }

    let rent = Rent::get()?;
    let rent_lamports = rent.minimum_balance(account_len);

    invoke_signed(
      &system_instruction::create_account(
        initializer.key,
        pda_account.key,
        rent_lamports,
        account_len.try_into().unwrap(),
        program_id,
      ),
      &[initializer.clone(), pda_account.clone(), system_program.clone()],
      &[&[initializer.key.as_ref(), id.to_le_bytes().as_ref(), &[bump_seed]]],
    )?;


    msg!("Unpacking state account ...");
    let mut account_data = try_from_slice_unchecked::<OnchainAccountState>(&pda_account.data.borrow()).unwrap();
    msg!("Borrowed account data");

    account_data.id = id;
    account_data.name = name;
    account_data.creator = *initializer.key;
    account_data.is_initialized = true;

    msg!("serializing account");
    account_data.serialize(&mut &mut pda_account.data.borrow_mut()[..])?;
    msg!("state account serialized");

    Ok(())
}



// Inside processor.rs
#[cfg(test)]
mod tests {
  use {
    super::*,
    assert_matches::*,
    solana_program::{
        instruction::{AccountMeta, Instruction},
        system_program::ID as SYSTEM_PROGRAM_ID,
    },
    solana_program_test::*,
    solana_sdk::{
        signature::Signer,
        transaction::Transaction
    },
  };

  // First unit test
  #[tokio::test]
  async fn test_create_onchain_account_instruction() {
    let program_id = Pubkey::new_unique();
    let (mut banks_client, payer, recent_blockhash) = ProgramTest::new(
        "test-program",
        program_id,
        processor!(process_instruction),
    )
    .start()
    .await;

    // Create Onchain PDA
    const id: u8 = 3;
    let name: String = "Test".to_owned();


    let (onchain_account_pda, _bump_seed) =
      Pubkey::find_program_address(&[payer.pubkey().as_ref(), &id.to_le_bytes().as_ref()], &program_id);

    // Concat data to single buffer
    let mut data_vec = vec![0];

    
    data_vec.append(
        &mut (TryInto::<u32>::try_into(id.to_string().len()).unwrap().to_le_bytes())
            .try_into()
            .unwrap(),
    );
    data_vec.append(&mut id.to_string().into_bytes());
    data_vec.push(id);

    data_vec.append(
        &mut (TryInto::<u32>::try_into(name.len())
            .unwrap()
            .to_le_bytes())
        .try_into()
        .unwrap(),
    );
    data_vec.append(&mut name.into_bytes());

    // Create transaction object with instructions, accounts, and input data
    let mut transaction = Transaction::new_with_payer(
        &[
        Instruction {
            program_id: program_id,
            accounts: vec![
                AccountMeta::new_readonly(payer.pubkey(), true),
                AccountMeta::new(onchain_account_pda, false),
                AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
            ],
            data: data_vec,
        },
        ],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[&payer], recent_blockhash);

    // Process transaction and compare the result
    assert_matches!(banks_client.process_transaction(transaction).await, Ok(_));
    


  }

}