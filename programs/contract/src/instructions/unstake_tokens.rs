use borsh::{BorshDeserialize, BorshSerialize};
use pinocchio::{account_info::AccountInfo, msg, program_error::ProgramError, pubkey::Pubkey, sysvars::{clock::Clock, Sysvar}, ProgramResult};
use crate::{constants::GLOBAL_SEED, helpers::{global_pda, next_account}, state::MinerAccount};
use pinocchio_token::instructions::{Transfer};
use pinocchio::{seeds, instruction::Signer};

pub fn unstake_tokens(program_id: &Pubkey, accounts: &[AccountInfo], amount: u64) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let miner = next_account(accounts_iter)?;
    let miner_account = next_account(accounts_iter)?;
    let stake_vault = next_account(accounts_iter)?;
    let miner_token_account = next_account(accounts_iter)?;
    let _token_program = next_account(accounts_iter)?;
    let global_account = next_account(accounts_iter)?;

    if !miner.is_signer() {
        msg!("Miner must sign");
        return Err(ProgramError::MissingRequiredSignature);
    }

    let mut miner_data = miner_account.try_borrow_mut_data()?;
    let mut miner_acc = MinerAccount::try_from_slice(&miner_data).map_err(|_| {
        msg!("Failed to deserialize miner account");
        ProgramError::InvalidAccountData
    })?;

    let clock = Clock::get()?;
    if clock.unix_timestamp < miner_acc.unstake_ts {
        msg!("Cooldown period not elapsed");
        return Err(ProgramError::Custom(4));
    }

    if miner_acc.stake < amount {
        msg!("Insufficient stake");
        return Err(ProgramError::InsufficientFunds);
    }

    let (global_pda, bump) = global_pda(program_id);
    if *global_account.key() != global_pda {
        msg!("Global PDA mismatch");
        return Err(ProgramError::InvalidArgument);
    }

    let seed_bump = bump.clone();

    let seed_bump_arr = [seed_bump];
    let seeds = seeds!(GLOBAL_SEED, &seed_bump_arr);
    let signer = Signer::from(&seeds);

    Transfer {
        from: stake_vault,
        to: miner_token_account,
        authority: global_account,
        amount,
    }.invoke_signed(&[signer])?;

    miner_acc.stake = miner_acc.stake.saturating_sub(amount);
    miner_acc.serialize(&mut &mut miner_data[..]).map_err(|_| {
        msg!("Failed to serialize miner account");
        ProgramError::InvalidAccountData
    })?;

    msg!(&format!("EVENT:Unstaked:{:?}:{}", miner.key(), amount));
    Ok(())
}