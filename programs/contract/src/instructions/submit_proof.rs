use borsh::{BorshDeserialize, BorshSerialize};
use pinocchio::{msg, program_error::ProgramError, pubkey::Pubkey, sysvars::{clock::Clock, Sysvar}, ProgramResult};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
};

use crate::state::{Challenge, ChallengeStatus, MinerStake};

pub fn submit_proof(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    challenge_id: u128,
    proof_hash: [u8; 32]
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let miner = next_account_info(account_info_iter).map_err(|_| {
        ProgramError::InvalidAccountData
    })?;

    let challenge_account = next_account_info(account_info_iter).map_err(|_| {
        ProgramError::InvalidAccountData
    })?;

    let data_proof_account = next_account_info(account_info_iter).map_err(|_| {
        ProgramError::InvalidAccountData
    })?;

    let miner_stake_account = next_account_info(account_info_iter).map_err(|_| {
        ProgramError::InvalidAccountData
    })?;

    if !miner.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let mut challenge: Challenge = Challenge::try_from_slice(&challenge_account.data.borrow())
        .map_err(|_| ProgramError::InvalidAccountData)?;

    if challenge.challenge_id != challenge_id {
        msg!("challenge id mismatch");
        return Err(ProgramError::InvalidAccountData);
    }

    if challenge.status != ChallengeStatus::Open {
        msg!("challenge is not open");
        return Err(ProgramError::InvalidAccountData);
    }

    let now = Clock::get()?.unix_timestamp;
    if now > challenge.expires_at {
        msg!("challenge has expired");
        return Err(ProgramError::InvalidAccountData);
    }

    // TODO: must verify merkle inclusion path here using provided proof path.

    challenge.solver = Some(*miner.key);
    challenge.status = ChallengeStatus::Solved;
    challenge.serialize(&mut *challenge_account.data.borrow_mut())
        .map_err(|_| ProgramError::InvalidAccountData)?;

    let mut miner_stake: MinerStake = MinerStake::try_from_slice(&miner_stake_account.data.borrow()).map_err(|_| ProgramError::InvalidAccountData)?;

    miner_stake.pending_rewards = miner_stake.pending_rewards.saturating_add(challenge.reward);

    miner_stake.serialize(&mut *miner_stake_account.data.borrow_mut())
        .map_err(|_| ProgramError::InvalidAccountData)?;

    msg!(&format!("challenge {} solved by {}: reward {}", challenge_id, miner.key, challenge.reward));
    Ok(())
}