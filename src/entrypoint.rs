use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    msg,
    pubkey::Pubkey,
    entrypoint::ProgramResult,
};

use crate::processor;


// Declare and export the program's entrypoint
entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    msg!("Hello Solana!");
    msg!(
        "process_instruction: {}: {} accounts, data={:?}",
        program_id,
        accounts.len(),
        instruction_data
    );
    processor::process_instruction(program_id, accounts, instruction_data)?;

    Ok(())
}