use borsh::{BorshDeserialize, BorshSerialize};
use pinocchio::{account_info::AccountInfo, instruction::Signer, msg, program_error::ProgramError, pubkey::Pubkey, seeds, ProgramResult};
use crate::{constants::GLOBAL_SEED, helpers::{global_pda, next_account}, state::{EpochRecord, EpochStatus}};
use pinocchio_token::instructions::{Transfer};

pub fn finalize_epoch(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    epoch_id: u128,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let caller = next_account(accounts_iter)?;
    let epoch_account = next_account(accounts_iter)?;
    let _miner_account = next_account(accounts_iter)?;
    let reward_vault = next_account(accounts_iter)?;
    let miner_token_account = next_account(accounts_iter)?;
    let token_program = next_account(accounts_iter)?;
    let global_account = next_account(accounts_iter)?;

    if !caller.is_signer() {
        msg!("Caller must sign");
        return Err(ProgramError::MissingRequiredSignature);
    }

    let mut epoch_data = epoch_account.try_borrow_mut_data()?;
    let mut epoch: EpochRecord = EpochRecord::try_from_slice(&epoch_data).map_err(|_| {
        msg!("Failed to deserialize epoch data");
        ProgramError::InvalidAccountData
    })?;

    if epoch.epoch_id != epoch_id {
        msg!("Epoch mismatch");
        return Err(ProgramError::InvalidArgument);
    }

    if epoch.status == EpochStatus::Challenged {
        msg!("Epoch under dispute");
        return Err(ProgramError::Custom(2));
    }

    if epoch.status != EpochStatus::Submitted {
        msg!("Epoch not submitted");
        return Err(ProgramError::InvalidArgument);
    }

    let (global_pda, bump) = global_pda(program_id);
    if *global_account.key() != global_pda {
        msg!("Global PDA mismatch");
        return Err(ProgramError::InvalidArgument);
    }

    let amount = epoch.reward;
    let seed_bump = bump.clone();

    let seed_bump_arr = [seed_bump];
    let seeds = seeds!(GLOBAL_SEED, &seed_bump_arr);
    let signer = Signer::from(&seeds);

    // Use pinocchio token transfer
    Transfer {
        from: reward_vault,
        to: miner_token_account,
        authority: global_account,
        amount,
    }.invoke_signed(&[signer])?;

    epoch.status = EpochStatus::Finalized;
    epoch.serialize(&mut &mut epoch_data[..]).map_err(|_| {
        msg!("Failed to serialize epoch data");
        ProgramError::InvalidAccountData
    })?;

    msg!(&format!("EVENT:EpochFinalized:{}:{}", epoch_id, amount));
    Ok(())
}
