use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
};
use pinocchio::{msg, program_error::ProgramError, pubkey::Pubkey, sysvars::{clock::Clock, Sysvar}, ProgramResult};

use crate::state::{Challenge, DataProof};

pub fn create_challenge(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    proof_id: u128,
) -> ProgramResult {

    let account_info_iter = &mut accounts.iter();
    let caller = next_account_info(account_info_iter).map_err(|_| ProgramError::NotEnoughAccountKeys)?;
    let data_proof_acc = next_account_info(account_info_iter).map_err(|_| ProgramError::NotEnoughAccountKeys)?;
    let challenge_acc = next_account_info(account_info_iter).map_err(|_| ProgramError::NotEnoughAccountKeys)?;
    let system_program = next_account_info(account_info_iter).map_err(|_| ProgramError::NotEnoughAccountKeys)?;
    

    let proof: DataProof = DataProof::try_from_slice(&data_proof_acc.data.borrow())
        .map_err(|_| ProgramError::InvalidAccountData)?;

    let now = Clock::get()?.unix_timestamp as u64;
    let chunk_index = (now % (proof.total_chunks as u64)) as u32;

    let challenge = Challenge {
        proof_id,
        chunk_index,
        expires_at: (now + 300) as i64,
        solver: None,
        reward: 1_000_000,
        status: crate::state::ChallengeStatus::Open,
        challenge_id: now as u128 ^ proof_id, 
    };

    challenge.serialize(&mut *challenge_acc.data.borrow_mut())
        .map_err(|_| ProgramError::AccountDataTooSmall)?;

    msg!(&format!("Challenge created: {:?}", challenge));
    Ok(())
}