use borsh::BorshSerialize;
use pinocchio::{account_info::AccountInfo, msg, program_error::ProgramError, pubkey::Pubkey, sysvars::{clock::Clock, Sysvar}, ProgramResult};

use crate::{helpers::{global_pda, next_account}, state::GlobalState};

pub fn initialize(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    decay_n: u64,
    decay_d: u64,
    emission_cap: u64,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let admin = next_account(accounts_iter)?;
    let global_account = next_account(accounts_iter)?;
    let tape_mint_account = next_account(accounts_iter)?;
    let reward_vault_account = next_account(accounts_iter)?;

    if !admin.is_signer() {
        msg!("Admin must sign");
        return Err(ProgramError::MissingRequiredSignature);
    }

    let (expected_global, bump) = global_pda(program_id);
    if expected_global != *global_account.key() {
        msg!("Global PDA mismatch");
        return Err(ProgramError::InvalidArgument);
    }

    let clock = Clock::get()?;
    let global_state = GlobalState {
        admin: *admin.key(),
        reward_vault: *reward_vault_account.key(),
        emission_cap,
        reward_mint: *tape_mint_account.key(),
        total_minted: 0,
        decay_numerator: decay_n,
        decay_denom: decay_d,
        last_decay_at: clock.unix_timestamp,
    };

    let mut data = global_account.try_borrow_mut_data()?;
    global_state.serialize(&mut &mut data[..]).map_err(|_| ProgramError::AccountDataTooSmall)?;

    msg!("EVENT:Initialized");
    Ok(())
}
