use solana_program::{
    entrypoint::ProgramResult,
    pubkey::Pubkey,
    msg,
    account_info::{next_account_info, AccountInfo},
    system_instruction,
    sysvar::{
      rent::Rent,
      rent::ID as RENT_PROGRAM_ID,
      Sysvar,
    },
    native_token::LAMPORTS_PER_SOL,
    system_program::ID as SYSTEM_PROGRAM_ID,
    program_pack::{IsInitialized, Pack},
    program::{invoke_signed},
    borsh::try_from_slice_unchecked,
    program_error::ProgramError,
};

use spl_token::{
  instruction::{initialize_mint, mint_to},
  ID as TOKEN_PROGRAM_ID,
};

use spl_associated_token_account::get_associated_token_address;

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

      // Create new movie review
      OnchainAccountInstruction::CreateOnchainAccount { id, name} => {
        msg!("Instruction: Add Onchain Account");
        process_create_onchain_account(program_id, accounts, id, name)
      },
      
      // New instruction handled here to initialize the mint account
      OnchainAccountInstruction::InitializeCustomMint => {
        msg!("Instruction: Initialize Mint");
        process_initialize_token_mint(program_id, accounts)
      }

    }
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

    msg!("PDA2 created: {}", pda);

    msg!("unpacking state account");
    let mut account_data = try_from_slice_unchecked::<OnchainAccountState>(&pda_account.data.borrow()).unwrap();
    msg!("borrowed account data");

    account_data.id = id;
    account_data.name = name;
    account_data.creator = *initializer.key;
    account_data.is_initialized = true;

    msg!("serializing account");
    account_data.serialize(&mut &mut pda_account.data.borrow_mut()[..])?;
    msg!("state account serialized");

    Ok(())
}


pub fn process_initialize_token_mint(program_id: &Pubkey, accounts: &[AccountInfo]
) -> ProgramResult {

  let account_info_iter = &mut accounts.iter();

  // The order of accounts is not arbitrary, the client will send them in this order
  
  // Whoever sent in the transaction
  let initializer = next_account_info(account_info_iter)?;
  // Token mint PDA - derived on the client
  let token_mint = next_account_info(account_info_iter)?;
  // Token mint authority (this should be you)
  let mint_auth = next_account_info(account_info_iter)?;
  // System program to create a new account
  let system_program = next_account_info(account_info_iter)?;
  // Solana Token program address
  let token_program = next_account_info(account_info_iter)?;
  // System account to calcuate the rent
  let sysvar_rent = next_account_info(account_info_iter)?;

  // Derive the mint PDA again so we can validate it
  // The seed is just "token_mint"
  let (mint_pda, mint_bump) = Pubkey::find_program_address(&[b"token_mint"], program_id);
  // Derive the mint authority so we can validate it
  // The seed is just "token_auth"
  let (mint_auth_pda, _mint_auth_bump) =
      Pubkey::find_program_address(&[b"token_auth"], program_id);

  msg!("Token mint: {:?}", mint_pda);
  msg!("Mint authority: {:?}", mint_auth_pda);

  // Validate the important accounts passed in
  if mint_pda != *token_mint.key {
      msg!("Incorrect token mint account");
      return Err(OnchainAccountError::IncorrectAccountError.into());
  }

  if *token_program.key != TOKEN_PROGRAM_ID {
      msg!("Incorrect token program");
      return Err(OnchainAccountError::IncorrectAccountError.into());
  }

  if *mint_auth.key != mint_auth_pda {
      msg!("Incorrect mint auth account");
      return Err(OnchainAccountError::IncorrectAccountError.into());
  }

  if *system_program.key != SYSTEM_PROGRAM_ID {
      msg!("Incorrect system program");
      return Err(OnchainAccountError::IncorrectAccountError.into());
  }

  if *sysvar_rent.key != RENT_PROGRAM_ID {
      msg!("Incorrect rent program");
      return Err(OnchainAccountError::IncorrectAccountError.into());
  }

  // Calculate the rent
  let rent = Rent::get()?;
  // We know the size of a mint account is 82 (remember it lol)
  let rent_lamports = rent.minimum_balance(82);

  // Create the token mint PDA
  invoke_signed(
      &system_instruction::create_account(
          initializer.key,
          token_mint.key,
          rent_lamports,
          82, // Size of the token mint account
          token_program.key,
      ),
      // Accounts we're reading from or writing to 
      &[
          initializer.clone(),
          token_mint.clone(),
          system_program.clone(),
      ],
      // Seeds for our token mint account
      &[&[b"token_mint", &[mint_bump]]],
  )?;

  msg!("Created token mint account");

  // Initialize the mint account
  invoke_signed(
      &initialize_mint(
          token_program.key,
          token_mint.key,
          mint_auth.key,
          Option::None, // Freeze authority - we don't want anyone to be able to freeze!
          9, // Number of decimals
      )?,
      // Which accounts we're reading from or writing to
      &[token_mint.clone(), sysvar_rent.clone(), mint_auth.clone()],
      // The seeds for our token mint PDA
      &[&[b"token_mint", &[mint_bump]]],
  )?;

  msg!("Initialized token mint");

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
        transaction::Transaction,
        sysvar::rent::ID as SYSVAR_RENT_ID
    },
    spl_associated_token_account::{
        get_associated_token_address,
        instruction::create_associated_token_account,
    },
    spl_token:: ID as TOKEN_PROGRAM_ID,
  };


  // Inside the the tests modules
  fn create_init_mint_ix(payer: Pubkey, program_id: Pubkey) -> (Pubkey, Pubkey, Instruction) {
    // Derive PDA for token mint authority
    let (mint, _bump_seed) = Pubkey::find_program_address(&[b"token_mint"], &program_id);
    let (mint_auth, _bump_seed) = Pubkey::find_program_address(&[b"token_auth"], &program_id);

    let init_mint_ix = Instruction {
        program_id: program_id,
        accounts: vec![
            AccountMeta::new_readonly(payer, true),
            AccountMeta::new(mint, false),
            AccountMeta::new(mint_auth, false),
            AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
            AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
            AccountMeta::new_readonly(SYSVAR_RENT_ID, false)
        ],
        data: vec![3]
    };

    (mint, mint_auth, init_mint_ix)
  }

  // First unit test
  #[tokio::test]
  async fn test_initialize_mint_instruction() {
      let program_id = Pubkey::new_unique();
      let (mut banks_client, payer, recent_blockhash) = ProgramTest::new(
          "test-program",
          program_id,
          processor!(process_instruction),
      )
      .start()
      .await;

      // Call helper function
      let (_mint, _mint_auth, init_mint_ix) = create_init_mint_ix(payer.pubkey(), program_id);

      // Create transaction object with instructions, accounts, and input data
      let mut transaction = Transaction::new_with_payer(
          &[init_mint_ix,],
          Some(&payer.pubkey()),
      );
      transaction.sign(&[&payer], recent_blockhash);

      // Process transaction and compare the result
      assert_matches!(banks_client.process_transaction(transaction).await, Ok(_));
  }

  // Second unit test
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