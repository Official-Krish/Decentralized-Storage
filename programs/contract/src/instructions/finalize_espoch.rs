use borsh::BorshSerialize;
use pinocchio::{account_info::AccountInfo, msg, program_error::ProgramError, pubkey::Pubkey, sysvars::{clock::Clock, Sysvar}, ProgramResult};

use crate::{helpers::{global_pda, next_account}, state::GlobalState};

pub fn finalize_espoch(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    epoch_id: u128 
) -> ProgramResult {
    Ok(())
}