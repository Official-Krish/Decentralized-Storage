use borsh::BorshSerialize;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
};
use pinocchio::{msg, program_error::ProgramError, pubkey::Pubkey, sysvars::{clock::Clock, Sysvar}, ProgramResult};

use crate::state::GlobalState;

pub fn initialize(
  program_id: &Pubkey,
  accounts: &[AccountInfo],
  decay_numerator: u64,
  decay_denom: u64,
  emission_cap: u64,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let admin = next_account_info(account_info_iter)
        .map_err(|_| ProgramError::NotEnoughAccountKeys)?;
    let global_state_account = next_account_info(account_info_iter)
        .map_err(|_| ProgramError::NotEnoughAccountKeys)?;
    let reward_mint = next_account_info(account_info_iter)
        .map_err(|_| ProgramError::NotEnoughAccountKeys)?;
    let reward_vault = next_account_info(account_info_iter)
        .map_err(|_| ProgramError::NotEnoughAccountKeys)?;

    if !admin.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let global = GlobalState {
        admin: *admin.key,
        reward_mint: *reward_mint.key,
        reward_vault: *reward_vault.key,
        total_minted: 0,
        emission_cap,
        decay_numerator,
        decay_denom,
        last_decay_at: Clock::get()?.unix_timestamp,
    };

    global.serialize(&mut *global_state_account.data.borrow_mut()).map_err(|_| ProgramError::InvalidArgument)?;

    msg!(&format!("Global state initialized"));
    Ok(())
}