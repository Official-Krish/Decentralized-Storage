use borsh::{BorshDeserialize, BorshSerialize};
use pinocchio::{account_info::AccountInfo, msg, program_error::ProgramError, pubkey::Pubkey, ProgramResult};
use crate::{helpers::next_account, state::{GlobalState, MinerAccount}};

pub fn slash_miner(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    _miner_pub: Pubkey,
    amount: u64,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let admin = next_account(accounts_iter)?;
    let miner_account = next_account(accounts_iter)?;
    let global_account = next_account(accounts_iter)?;

    if !admin.is_signer() {
        msg!("Admin must sign");
        return Err(ProgramError::MissingRequiredSignature);
    }

    let global_data = global_account.try_borrow_data()?;
    let global_state = GlobalState::try_from_slice(&global_data).map_err(|_| {
        msg!("Failed to deserialize global state");
        ProgramError::InvalidAccountData
    })?;

    if global_state.admin != *admin.key() {
        msg!("Not authorized admin");
        return Err(ProgramError::IllegalOwner);
    }

    let mut miner_data = miner_account.try_borrow_mut_data()?;
    let mut miner_acc = MinerAccount::try_from_slice(&miner_data).map_err(|_| {
        msg!("Failed to deserialize miner account");
        ProgramError::InvalidAccountData
    })?;

    miner_acc.stake = miner_acc.stake.saturating_sub(amount);
    miner_acc.reputation = miner_acc.reputation.saturating_sub(10);
    miner_acc.serialize(&mut &mut miner_data[..]).map_err(|_| {
        msg!("Failed to serialize miner account");
        ProgramError::InvalidAccountData
    })?;

    msg!(&format!("EVENT:MinerSlashed:{}", amount));
    Ok(())
}