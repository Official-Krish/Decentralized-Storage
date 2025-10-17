use pinocchio::{account_info::AccountInfo, msg, program_error::ProgramError, pubkey::Pubkey, ProgramResult};
use borsh::{BorshDeserialize, BorshSerialize};

use crate::{helpers::next_account, state::{EpochRecord, EpochStatus, MinerAccount}};

pub fn challenge_proof(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    epoch_id: u128,
    evidence_hash: [u8; 32],
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let challenger = next_account(accounts_iter)?;
    let epoch_account = next_account(accounts_iter)?;
    let miner_account = next_account(accounts_iter)?;

    if !challenger.is_signer() {
        msg!("Challenger must sign");
        return Err(ProgramError::MissingRequiredSignature);
    }

    let mut epoch_data = epoch_account.try_borrow_mut_data()?;
    let mut epoch: EpochRecord = EpochRecord::try_from_slice(&epoch_data).map_err(|_| {
        msg!("Failed to deserialize epoch record");
        ProgramError::InvalidAccountData
    })?;

    if epoch.epoch_id != epoch_id {
        msg!("Epoch mismatch");
        return Err(ProgramError::InvalidArgument);
    }

    if epoch.status != EpochStatus::Submitted {
        msg!("Epoch not in submitted state");
        return Err(ProgramError::InvalidArgument);
    }

    epoch.status = EpochStatus::Challenged;
    epoch.serialize(&mut &mut epoch_data[..]).map_err(|_| {
        msg!("Failed to serialize epoch record");
        ProgramError::InvalidAccountData
    })?;

    let mut miner_data = miner_account.try_borrow_mut_data()?;
    let mut miner_acc = MinerAccount::try_from_slice(&miner_data).map_err(|_| {
        msg!("Failed to deserialize miner account");
        ProgramError::InvalidAccountData
    })?;
    miner_acc.reputation = miner_acc.reputation.saturating_sub(1);
    miner_acc.serialize(&mut &mut miner_data[..]).map_err(|_| {
        msg!("Failed to serialize miner account");
        ProgramError::InvalidAccountData
    })?;

    msg!(&format!("EVENT:EpochChallenged:{}", epoch_id));
    Ok(())
}