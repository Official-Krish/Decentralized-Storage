use pinocchio::{account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey};
use crate::constants::{EPOCH_SEED, GLOBAL_SEED, MINER_SEED, OBJECT_SEED, REWARD_VAULT_SEED, TAPE_MINT_SEED};
use pinocchio::pubkey::find_program_address;

pub fn find_pda(seeds: &[&[u8]], program_id: &Pubkey) -> (Pubkey, u8) {
    find_program_address(seeds, program_id)
}

pub fn global_pda(program_id: &Pubkey) -> (Pubkey, u8) {
    find_pda(&[GLOBAL_SEED], program_id)
}

pub fn object_pda(program_id: &Pubkey, owner: &Pubkey, object_id: u128) -> (Pubkey, u8) {
    let id_bytes = object_id.to_le_bytes();
    find_pda(&[OBJECT_SEED, owner.as_ref(), &id_bytes], program_id)
}

pub fn epoch_pda(program_id: &Pubkey, object_id: u128, epoch_id: u128) -> (Pubkey, u8) {
    let oid_bytes = object_id.to_le_bytes();
    let eid_bytes = epoch_id.to_le_bytes();
    find_pda(&[EPOCH_SEED, &oid_bytes, &eid_bytes], program_id)
}

pub fn miner_pda(program_id: &Pubkey, miner: &Pubkey) -> (Pubkey, u8) {
    find_pda(&[MINER_SEED, miner.as_ref()], program_id)
}

pub fn tape_mint_pda(program_id: &Pubkey) -> (Pubkey, u8) {
    find_pda(&[TAPE_MINT_SEED], program_id)
}

pub fn reward_vault_pda(program_id: &Pubkey, mint: &Pubkey) -> (Pubkey, u8) {
    find_pda(&[REWARD_VAULT_SEED, mint.as_ref()], program_id)
}

pub fn next_account<'a>(
    iter: &mut impl Iterator<Item = &'a AccountInfo>,
) -> Result<&'a AccountInfo, ProgramError> {
    iter.next().ok_or(ProgramError::NotEnoughAccountKeys)
}