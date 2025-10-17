use borsh::{BorshDeserialize, BorshSerialize};
use pinocchio::{account_info::AccountInfo, msg, program_error::ProgramError, pubkey::Pubkey, sysvars::{clock::Clock, Sysvar}, ProgramResult};

use crate::{helpers::{miner_pda, next_account}, state::{EpochRecord, EpochStatus, GlobalState, MinerAccount}};

pub fn submit_proof(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    epoch_id: u128,
    proof_hash: [u8; 32],
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let miner = next_account(accounts_iter)?;
    let epoch_account = next_account(accounts_iter)?;
    let miner_account = next_account(accounts_iter)?;
    let global_account = next_account(accounts_iter)?;

    if !miner.is_signer() {
        msg!("Miner must sign");
        return Err(ProgramError::MissingRequiredSignature);
    }

    let mut epoch_data = epoch_account.try_borrow_mut_data()?;
    let mut epoch: EpochRecord = EpochRecord::try_from_slice(&epoch_data).map_err(|_| {
        msg!("Failed to deserialize epoch record");
        ProgramError::InvalidAccountData
    })?;

    if epoch.epoch_id != epoch_id {
        msg!("Epoch ID mismatch");
        return Err(ProgramError::InvalidArgument);
    }

    if epoch.status != EpochStatus::Open {
        msg!("Epoch not open");
        return Err(ProgramError::InvalidArgument);
    }

    let clock = Clock::get()?;
    if clock.unix_timestamp > epoch.deadline_ts {
        epoch.status = EpochStatus::Challenged;
        epoch.serialize(&mut &mut epoch_data[..]).map_err(|_| {
            msg!("Failed to serialize epoch record");
            ProgramError::InvalidAccountData
        })?;
        msg!("Submission too late");
        return Err(ProgramError::Custom(1));
    }

    epoch.solver = Some(*miner.key());
    epoch.proof_hash = proof_hash;
    epoch.status = EpochStatus::Submitted;
    epoch.serialize(&mut &mut epoch_data[..]).map_err(|_| {
        msg!("Failed to serialize epoch record");
        ProgramError::InvalidAccountData
    })?;

    let (expected_miner, _) = miner_pda(program_id, miner.key());
    if expected_miner != *miner_account.key() {
        msg!("Miner account mismatch");
        return Err(ProgramError::InvalidArgument);
    }

    let mut miner_data = miner_account.try_borrow_mut_data().map_err(|_| {
        msg!("Failed to borrow miner account data");
        ProgramError::InvalidAccountData
    })?;
    let mut miner_acc = MinerAccount::try_from_slice(&miner_data).map_err(|_| {
        msg!("Failed to deserialize miner account");
        ProgramError::InvalidAccountData
    })?;
    miner_acc.pending_rewards = miner_acc.pending_rewards.saturating_add(epoch.reward);
    miner_acc.serialize(&mut &mut miner_data[..]).map_err(|_| {
        msg!("Failed to serialize miner account");
        ProgramError::InvalidAccountData
    })?;

    let mut global_data = global_account.try_borrow_mut_data().map_err(|_| {
        msg!("Failed to borrow global account data");
        ProgramError::InvalidAccountData
    })?;
    let mut global_state = GlobalState::try_from_slice(&global_data).map_err(|_| {
        msg!("Failed to deserialize global state");
        ProgramError::InvalidAccountData
    })?;
    if global_state.emission_cap >= epoch.reward {
        global_state.total_minted = global_state.total_minted.saturating_add(epoch.reward);
        global_state.emission_cap = global_state.emission_cap.saturating_sub(epoch.reward);
    }
    global_state.serialize(&mut &mut global_data[..]).map_err(|_| {
        msg!("Failed to serialize global state");
        ProgramError::InvalidAccountData
    })?;

    msg!(&format!("EVENT:EpochSubmitted:{}", epoch_id));
    Ok(())
}