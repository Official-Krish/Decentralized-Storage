use borsh::BorshSerialize;
use pinocchio::{account_info::AccountInfo, msg, program_error::ProgramError, pubkey::Pubkey, sysvars::{clock::Clock, Sysvar}, ProgramResult};

use crate::{helpers::{next_account, object_pda}, state::{ObjectRecord, ProofType}};

pub fn register_object(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    commitment: [u8; 32],
    proof_type: u8,
    size: u64,
    retention_epochs: u64,
    object_id: u128,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let owner = next_account(accounts_iter)?;
    let object_account = next_account(accounts_iter)?;

    if !owner.is_signer() {
        msg!("Owner must sign");
        return Err(ProgramError::MissingRequiredSignature);
    }

    let (expected_object, bump) = object_pda(program_id, owner.key(), object_id);
    if expected_object != *object_account.key() {
        msg!("Object PDA mismatch");
        return Err(ProgramError::InvalidArgument);
    }

    let proof_type_enum = match proof_type {
        0 => ProofType::CompactHash,
        1 => ProofType::Snark,
        x => ProofType::Other(x),
    };

    let clock = Clock::get()?;
    let object_record = ObjectRecord {
        owner: *owner.key(),
        commitment,
        proof_type: proof_type_enum,
        size,
        created_ts: clock.unix_timestamp,
        retention_epochs,
        bump,
    };

    let mut data = object_account.try_borrow_mut_data()?;
    object_record.serialize(&mut &mut data[..]).map_err(|_| {
        msg!("Failed to serialize object record");
        ProgramError::InvalidAccountData
    })?;

    msg!(&format!("EVENT:ObjectRegistered:{}", object_id));
    Ok(())
}