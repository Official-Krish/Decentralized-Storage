use borsh::{BorshSerialize, BorshDeserialize};
use pinocchio::pubkey::Pubkey;

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct GlobalState {
    pub admin: Pubkey,
    pub reward_mint: Pubkey,
    pub reward_vault: Pubkey, 
    pub total_minted: u64,
    pub emission_cap: u64, // remaining cap / current cap
    pub decay_numerator: u64, // represent decay as fraction (numerator/denom)
    pub decay_denom: u64,
    pub last_decay_at: i64,
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub enum ProofType {
    CompactHash,
    Snark, // SNARK/STARK style compact proofs
    Other(u8),
}

#[derive(BorshSerialize, BorshDeserialize, Debug, PartialEq)]
pub enum EpochStatus {
    Open,
    Submitted,
    Challenged,
    Finalized,
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct ObjectRecord {
    pub owner: Pubkey,
    pub commitment: [u8; 32], // commitment/descriptor (CID or hash)
    pub proof_type: ProofType,
    pub size: u64,
    pub created_ts: i64,
    pub retention_epochs: u64,
    pub bump: u8,
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct EpochRecord {
    pub object_id: u128,
    pub epoch_id: u128,
    pub nonce: u64,
    pub deadline_ts: i64,
    pub solver: Option<Pubkey>,
    pub proof_hash: [u8; 32], // hash of proof blob
    pub status: EpochStatus,
    pub reward: u64,
    pub bump: u8,
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct MinerAccount {
    pub miner: Pubkey,
    pub stake: u64,
    pub pending_rewards: u64,
    pub reputation: u32,
    pub unstake_ts: i64,
    pub bump: u8,
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub enum RewardInstruction {
    Initialize {
        decay_numerator: u64,
        decay_denom: u64,
        emission_cap: u64,
    },
    RegisterObject {
        commitment: [u8; 32],
        proof_type: u8,
        size: u64,
        retention_epochs: u64,
        object_id: u128,
    },
    CreateEpoch { 
        object_id: u128, 
        nonce: u64, 
        epoch_id: u128 
    },
    SubmitProof { 
        epoch_id: u128, 
        proof_hash: [u8; 32] 
    },
    ChallengeProof { 
        epoch_id: u128, 
        evidence_hash: [u8; 32] 
    },
    FinalizeEpoch { 
        epoch_id: u128 
    },
    Stake { 
        amount: u64 
    },
    Unstake { 
        amount: u64 
    },
    Slash { 
        miner: Pubkey, 
        amount: u64 
    },
}