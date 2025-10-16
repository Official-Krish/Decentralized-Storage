use solana_program::pubkey::Pubkey;
use borsh::{BorshSerialize, BorshDeserialize};

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
pub struct DataProof {
    pub uploader: Pubkey,
    pub merkle_root: [u8; 32],
    pub total_chunks: u32,
    pub created_at: i64,
    pub proof_id: u128,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, PartialEq)]
pub enum ChallengeStatus {
    Open,
    Solved,
    Expired,
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct Challenge {
    pub proof_id: u128,
    pub chunk_index: u32,
    pub expires_at: i64,
    pub solver: Option<Pubkey>,
    pub reward: u64,
    pub status: ChallengeStatus,
    pub challenge_id: u128,
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct MinerStake {
    pub miner: Pubkey,
    pub staked_amount: u64,
    pub reputation_score: u32,
    pub pending_rewards: u64,
    pub unstake_available_at: i64,
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub enum RewardInstruction {
    Initialize {
        decay_numerator: u64,
        decay_denom: u64,
        emission_cap: u64,
    },
    RegisterProof {
        merkle_root: [u8; 32],
        total_chunks: u32,
        proof_id: u128,
    },
    CreateChallenge {
        proof_id: u128,
    },
    SubmitProof {
        challenge_id: u128,
        proof_hash: [u8; 32],
    },
    ClaimRewards {},
    StakeTokens { amount: u64 },
    UnstakeTokens { amount: u64 },
    SlashMiner { amount: u64 },
}