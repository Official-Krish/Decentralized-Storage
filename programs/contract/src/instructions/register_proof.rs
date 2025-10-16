use borsh::BorshSerialize;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
};
use pinocchio::{msg, program_error::ProgramError, pubkey::Pubkey, sysvars::{clock::Clock, Sysvar}, ProgramResult};
use crate::state::DataProof;

pub fn register_proof(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    merkle_root: [u8; 32],
    total_chunks: u32,
    proof_id: u128,
) -> ProgramResult {
    
    let account_info_iter = &mut accounts.iter();
    let uploader = next_account_info(account_info_iter).map_err(|_| ProgramError::NotEnoughAccountKeys)?;
    let data_proof_account = next_account_info(account_info_iter).map_err(|_| ProgramError::NotEnoughAccountKeys)?;
    let system_program = next_account_info(account_info_iter).map_err(|_| ProgramError::NotEnoughAccountKeys)?;

    if !uploader.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let proof = DataProof {
        uploader: *uploader.key,
        merkle_root,
        total_chunks,
        proof_id,
        created_at: Clock::get()?.unix_timestamp,
    };

    proof.serialize(&mut *data_proof_account.data.borrow_mut()).map_err(|_| ProgramError::AccountDataTooSmall)?;

    msg!(&format!("Data proof registered with ID: {}", proof_id));
    Ok(())
}