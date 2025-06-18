use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    program::{invoke_signed},
    pubkey::Pubkey,
    system_instruction,
    sysvar::{rent::Rent, Sysvar},
};

entrypoint!(process_instruction);

#[derive(BorshSerialize, BorshDeserialize, Debug)]
struct OnChainData {
    count: u32,
}

fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    _instruction_data: &[u8],
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let pda_account = next_account_info(account_info_iter)?;      // The PDA
    let user_account = next_account_info(account_info_iter)?;     // Signer
    let system_program = next_account_info(account_info_iter)?;   // System Program

    // Derive PDA and bump
    let (pda, bump) = Pubkey::find_program_address(&[user_account.key.as_ref(), b"user"], program_id);

    // Check PDA is correct
    assert_eq!(&pda, pda_account.key);

    // Calculate rent-exempt minimum
    let rent = Rent::get()?;
    let space = std::mem::size_of::<OnChainData>();
    let lamports = rent.minimum_balance(space);

    let create_account_ix = system_instruction::create_account(
        user_account.key,
        pda_account.key,
        lamports,
        space as u64,
        program_id,
    );

    let seeds: &[&[u8]] = &[user_account.key.as_ref(), b"user", &[bump]];
    invoke_signed(
        &create_account_ix,
        &[user_account.clone(), pda_account.clone(), system_program.clone()],
        &[seeds],
    )?;

    // Optionally: initialize data in the account
    let data = OnChainData { count: 0 };
    data.serialize(&mut &mut pda_account.data.borrow_mut()[..])?;

    Ok(())
}
