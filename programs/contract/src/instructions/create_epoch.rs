use borsh::{BorshDeserialize, BorshSerialize};
use pinocchio::{account_info::AccountInfo, msg, program_error::ProgramError, pubkey::Pubkey, sysvars::{clock::Clock, Sysvar}, ProgramResult};

use crate::{helpers::{epoch_pda, next_account}, state::{EpochRecord, EpochStatus, ObjectRecord}};

pub fn create_epoch(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    object_id: u128,
    nonce: u64,
    epoch_id: u128,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let caller = next_account(accounts_iter)?;
    let object_account = next_account(accounts_iter)?;
    let epoch_account = next_account(accounts_iter)?;

    if !caller.is_signer() {
        msg!("Caller must sign");
        return Err(ProgramError::MissingRequiredSignature);
    }

    let object_data = object_account.try_borrow_data()?;
    let _object: ObjectRecord = ObjectRecord::try_from_slice(&object_data).map_err(|_| {
        msg!("Failed to deserialize object record");
        ProgramError::InvalidAccountData
    })?;

    let clock = Clock::get()?;
    let deadline = clock.unix_timestamp + 120;

    let (expected_epoch, bump) = epoch_pda(program_id, object_id, epoch_id);
    if expected_epoch != *epoch_account.key() {
        msg!("Epoch PDA mismatch");
        return Err(ProgramError::InvalidArgument);
    }

    let epoch_record = EpochRecord {
        object_id,
        epoch_id,
        nonce,
        deadline_ts: deadline,
        solver: None,
        proof_hash: [0u8; 32],
        status: EpochStatus::Open,
        reward: 1_000_000,
        bump,
    };

    let mut data = epoch_account.try_borrow_mut_data()?;
    epoch_record.serialize(&mut &mut data[..]).map_err(|_| {
        msg!("Failed to serialize epoch record");
        ProgramError::InvalidAccountData
    })?;

    msg!(&format!("EVENT:EpochCreated:{}:{}", epoch_id, nonce));
    Ok(())
}