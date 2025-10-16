
use borsh::BorshDeserialize;
use pinocchio::{
  entrypoint, msg, pubkey::Pubkey, ProgramResult
};
use solana_program::{
  account_info::AccountInfo,
};
use crate::{state::RewardInstruction};
mod state;
mod instructions;

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

    RewardInstruction::RegisterProof {
      merkle_root,
      total_chunks,
      proof_id,
    } => instructions::register_proof(program_id, accounts, merkle_root, total_chunks, proof_id),

    RewardInstruction::CreateChallenge { proof_id } => 
      instructions::create_challenge(program_id, accounts, proof_id),

    RewardInstruction::SubmitProof {
      challenge_id,
      proof_hash,
    } => instructions::submit_proof(program_id, accounts, challenge_id, proof_hash),

    RewardInstruction::ClaimRewards {} => instructions::claim_reward(program_id, accounts),

    RewardInstruction::StakeTokens { amount } => instructions::stake_tokens(program_id, accounts, amount),

    RewardInstruction::UnstakeTokens { amount } => {
      instructions::unstake_tokens(program_id, accounts, amount)
    }

    RewardInstruction::SlashMiner { amount } => instructions::slash_miner(program_id, accounts, amount),
  }
}