use borsh::{BorshDeserialize, BorshSerialize};
use pinocchio::{msg, program_error::ProgramError, pubkey::Pubkey, ProgramResult};
use solana_program::{account_info::{next_account_info, AccountInfo}, program::invoke_signed};
use spl_token::solana_program::instruction::Instruction;

use crate::state::MinerStake;

pub fn claim_reward(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let miner = next_account_info(account_info_iter).map_err(|_| {
        ProgramError::InvalidAccountData
    })?;

    let miner_stake_account = next_account_info(account_info_iter).map_err(|_| {
        ProgramError::InvalidAccountData
    })?;

    let reward_vault = next_account_info(account_info_iter).map_err(|_| {
        ProgramError::InvalidAccountData
    })?;

    let miner_token_account = next_account_info(account_info_iter).map_err(|_| {
        ProgramError::InvalidAccountData
    })?;

    let token_program = next_account_info(account_info_iter).map_err(|_| {
        ProgramError::InvalidAccountData
    })?;

    if !miner.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let mut miner_stake_data = MinerStake::try_from_slice(&miner_stake_account.data.borrow())
        .map_err(|_| ProgramError::InvalidAccountData)?;

    let amount = miner_stake_data.pending_rewards;
    if amount == 0 {
        return Err(ProgramError::Custom(0));
    }

    //TODO: Transfer from reward vault -> miner token account using spl token transfer via CPI

    let seeds = &[
        b"miner_stake".as_ref(),
        miner.key.as_ref(),
        program_id.as_ref(),
    ];


    let ix: Instruction = spl_token::instruction::transfer(
        token_program.key,
        reward_vault.key,
        miner_token_account.key,
        reward_vault.key,
        &[],
        amount,
    ).map_err(|_| ProgramError::Custom(1))?;

    invoke_signed(
        &ix,
        &[reward_vault.clone(), miner_token_account.clone(), token_program.clone()],
        &[seeds],
    )
    .map_err(|_| ProgramError::Custom(2))?;

    miner_stake_data.pending_rewards = 0;
    miner_stake_data.serialize(&mut *miner_stake_account.data.borrow_mut())
        .map_err(|_| ProgramError::InvalidAccountData)?;

    msg!(&format!("Reward of {} claimed by miner {}", amount, miner.key));
    Ok(())
}