use borsh::{BorshDeserialize, BorshSerialize};
use pinocchio::{account_info::AccountInfo, msg, program_error::ProgramError, pubkey::Pubkey, sysvars::{clock::Clock, Sysvar}, ProgramResult};
use crate::{helpers::next_account, state::MinerAccount};
use pinocchio_token::instructions::{Transfer};

pub fn stake_tokens(program_id: &Pubkey, accounts: &[AccountInfo], amount: u64) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let miner = next_account(accounts_iter)?;
    let miner_token_account = next_account(accounts_iter)?;
    let stake_vault = next_account(accounts_iter)?;
    let miner_account = next_account(accounts_iter)?;
    let _token_program = next_account(accounts_iter)?;

    if !miner.is_signer() {
        msg!("Miner must sign");
        return Err(ProgramError::MissingRequiredSignature);
    }

    Transfer {
        from: miner_token_account,
        to: stake_vault,
        authority: miner,
        amount,
    }.invoke()?;

    let mut miner_data = miner_account.try_borrow_mut_data()?;
    let mut miner_acc = MinerAccount::try_from_slice(&miner_data).map_err(|_| {
        msg!("Failed to deserialize miner account");
        ProgramError::InvalidAccountData
    })?;
    miner_acc.stake = miner_acc.stake.saturating_add(amount);
    
    let clock = Clock::get()?;
    miner_acc.unstake_ts = clock.unix_timestamp + 3600;
    miner_acc.serialize(&mut &mut miner_data[..]).map_err(|_| {
        msg!("Failed to serialize miner account");
        ProgramError::InvalidAccountData
    })?;

    msg!(&format!("EVENT:Staked:{:?}:{}", miner.key(), amount));
    Ok(())
}