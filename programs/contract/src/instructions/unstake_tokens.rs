use borsh::{BorshDeserialize, BorshSerialize};
use pinocchio::{msg, program_error::ProgramError, pubkey::Pubkey, sysvars::{clock::Clock, Sysvar}, ProgramResult};
use solana_program::{account_info::{next_account_info, AccountInfo}, program::invoke_signed};

use crate::state::MinerStake;

pub fn unstake_tokens(program_id: &Pubkey, accounts: &[AccountInfo], amount: u64) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let miner = next_account_info(account_info_iter).map_err(|_| ProgramError::NotEnoughAccountKeys)?; 
    let miner_stake_account = next_account_info(account_info_iter).map_err(|_| ProgramError::NotEnoughAccountKeys)?; 
    let stake_vault_account = next_account_info(account_info_iter).map_err(|_| ProgramError::NotEnoughAccountKeys)?; 
    let miner_token_acc = next_account_info(account_info_iter).map_err(|_| ProgramError::NotEnoughAccountKeys)?; 
    let token_program = next_account_info(account_info_iter).map_err(|_| ProgramError::NotEnoughAccountKeys)?;


    if !miner.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }


    let mut miner_stake = MinerStake::try_from_slice(&miner_stake_account.data.borrow()).map_err(|_| ProgramError::InvalidAccountData)?;
    let now = Clock::get()?.unix_timestamp;
        if now < miner_stake.unstake_available_at {
        msg!("unstake cooldown not passed");
        return Err(ProgramError::Custom(3));
    }


    if miner_stake.staked_amount < amount {
        msg!("not enough stake");
        return Err(ProgramError::InsufficientFunds);
    }


    let seeds: &[&[u8]] = &[]; 
    let ix = spl_token::instruction::transfer(
        token_program.key,
        stake_vault_account.key,
        miner_token_acc.key,
        stake_vault_account.key,
        &[],
        amount,
    ).map_err(|_| ProgramError::InvalidInstructionData)?;

    invoke_signed(&ix, &[stake_vault_account.clone(), miner_token_acc.clone(), token_program.clone()], &[seeds])
        .map_err(|_| ProgramError::Custom(4))?;


    miner_stake.staked_amount = miner_stake.staked_amount.saturating_sub(amount);
    miner_stake.serialize(&mut *miner_stake_account.data.borrow_mut()).map_err(|_| ProgramError::AccountDataTooSmall)?;


    msg!(&format!("miner {} unstaked {}", miner.key, amount));
    Ok(())
}