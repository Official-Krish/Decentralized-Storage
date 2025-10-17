use solana_program::pubkey::Pubkey;
use crate::constants::{CHALLENGE_SEED, GLOBAL_STATE_SEED, MINER_STAKE_SEED, PROOF_SEED, REWARD_VAULT_SEED, TAPE_MINT_SEED};

fn global_state_pda(program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[GLOBAL_STATE_SEED], program_id)
}

fn proof_pda(program_id: &Pubkey, uploader: &Pubkey, proof_id: u128) -> (Pubkey, u8) {
    let id_bytes = proof_id.to_le_bytes().to_vec();
    Pubkey::find_program_address(&[PROOF_SEED, uploader.as_ref(), &id_bytes], program_id)
}

fn challenge_pda(program_id: &Pubkey, proof_id: u128, nonce: u64) -> (Pubkey, u8) {
    let id_bytes = proof_id.to_le_bytes().to_vec();
    let nonce_bytes = nonce.to_le_bytes();
    Pubkey::find_program_address(&[CHALLENGE_SEED, &id_bytes, &nonce_bytes], program_id)
}

fn miner_stake_pda(program_id: &Pubkey, miner: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[MINER_STAKE_SEED, miner.as_ref()], program_id)
}

fn tape_mint_pda(program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[TAPE_MINT_SEED], program_id)
}

fn reward_vault_pda(program_id: &Pubkey, tape_mint: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[REWARD_VAULT_SEED, tape_mint.as_ref()], program_id)
}