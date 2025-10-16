use borsh::{BorshDeserialize, BorshSerialize};
use pinocchio::{msg, program_error::ProgramError, pubkey::Pubkey, sysvars::{clock::Clock, Sysvar}, ProgramResult};
use solana_program::{account_info::{next_account_info, AccountInfo}, program::invoke_signed};

use crate::state::MinerStake;

pub fn stake_tokens(program_id: &Pubkey, accounts: &[AccountInfo], amount: u64) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let miner = next_account_info(account_info_iter).map_err(|_| ProgramError::NotEnoughAccountKeys)?;
    let miner_token_acc = next_account_info(account_info_iter).map_err(|_| ProgramError::NotEnoughAccountKeys)?;
    let miner_stake_account = next_account_info(account_info_iter).map_err(|_| ProgramError::NotEnoughAccountKeys)?;
    let token_program = next_account_info(account_info_iter).map_err(|_| ProgramError::NotEnoughAccountKeys)?;
    let stake_vault_account = next_account_info(account_info_iter).map_err(|_| ProgramError::NotEnoughAccountKeys)?;


    if !miner.is_signer {
    return Err(ProgramError::MissingRequiredSignature);
    }


    let ix = spl_token::instruction::transfer(
        token_program.key,
        miner_token_acc.key,
        stake_vault_account.key,
        miner.key,
        &[],
        amount,
    ).map_err(|_| ProgramError::InvalidInstructionData)?;


    invoke_signed(&ix, &[miner_token_acc.clone(), stake_vault_account.clone(), token_program.clone()], &[])
        .map_err(|_| ProgramError::Custom(2))?;


    let mut miner_stake = MinerStake::try_from_slice(&miner_stake_account.data.borrow())
        .map_err(|_| ProgramError::InvalidAccountData)?;

    miner_stake.staked_amount = miner_stake.staked_amount.saturating_add(amount);
    miner_stake.unstake_available_at = Clock::get()?.unix_timestamp + 3600;
    miner_stake.serialize(&mut *miner_stake_account.data.borrow_mut()).map_err(|_| ProgramError::InvalidAccountData)?;

    msg!(&format!("miner {} staked {}", miner.key, amount));
    Ok(())
}