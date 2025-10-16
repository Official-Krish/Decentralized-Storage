use borsh::{BorshDeserialize, BorshSerialize};
use pinocchio::{msg, program_error::ProgramError, pubkey::Pubkey, ProgramResult};
use solana_program::{account_info::{next_account_info, AccountInfo}};

use crate::state::MinerStake;

pub fn slash_miner(program_id: &Pubkey, accounts: &[AccountInfo], amount: u64) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let admin = next_account_info(account_info_iter).map_err(|_| ProgramError::NotEnoughAccountKeys)?; 
    let miner_stake_account = next_account_info(account_info_iter).map_err(|_| ProgramError::NotEnoughAccountKeys)?;

    if !admin.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    //TODO: check admin == global.admin
    // TODO: Burn part of miner stake or transfer to reward vault
    let mut miner_stake = MinerStake::try_from_slice(&miner_stake_account.data.borrow()).map_err(|_| ProgramError::InvalidAccountData)?;
    miner_stake.staked_amount = miner_stake.staked_amount.saturating_sub(amount);
    miner_stake.reputation_score = miner_stake.reputation_score.saturating_sub(10);
    miner_stake.serialize(&mut *miner_stake_account.data.borrow_mut()).map_err(|_| ProgramError::AccountDataTooSmall)?;


    msg!(&format!("slashed miner {} by {}", miner_stake.miner, amount));
    Ok(())
}