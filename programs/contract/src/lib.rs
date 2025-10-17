#![allow(unexpected_cfgs)]

use borsh::BorshDeserialize;
use pinocchio::{
  account_info::AccountInfo, entrypoint, msg, pubkey::Pubkey, ProgramResult
};
use crate::{state::RewardInstruction};
mod state;
mod instructions;
mod helpers;
mod constants;

entrypoint!(process_instruction);

pub fn process_instruction(
  program_id: &Pubkey,
  accounts: &[AccountInfo],
  instruction_data: &[u8],
) -> ProgramResult {

  let instructions = RewardInstruction::try_from_slice(instruction_data)
  .map_err(|_| {
    msg!("Failed to deserialize instruction data");
    pinocchio::program_error::ProgramError::InvalidInstructionData
  })?;

  match instructions {
    RewardInstruction::Initialize {
      decay_numerator,
      decay_denom,
      emission_cap,
    } => instructions::initialize(program_id, accounts, decay_numerator, decay_denom, emission_cap),

    RewardInstruction::RegisterObject {
      commitment,
      proof_type,
      size,
      retention_epochs,
      object_id,
    } => instructions::register_object(program_id, accounts, commitment, proof_type, size, retention_epochs, object_id),

    RewardInstruction::CreateEpoch { object_id, nonce, epoch_id } =>
      instructions::create_epoch(program_id, accounts, object_id, nonce, epoch_id),

    RewardInstruction::SubmitProof {
      epoch_id,
      proof_hash,
    } => instructions::submit_proof(program_id, accounts, epoch_id, proof_hash),

    RewardInstruction::ChallengeProof {
       epoch_id,
       evidence_hash,
    } => instructions::challenge_proof(program_id, accounts, epoch_id, evidence_hash),

    RewardInstruction::FinalizeEpoch { epoch_id } => instructions::finalize_espoch(program_id, accounts, epoch_id),

    RewardInstruction::Stake { amount } => {
      instructions::stake_tokens(program_id, accounts, amount)
    }

    RewardInstruction::Unstake { amount } => {
      instructions::unstake_tokens(program_id, accounts, amount)
    }

    RewardInstruction::Slash { 
      miner,
      amount,
    } => instructions::slash_miner(program_id, accounts, miner, amount),
  }
}