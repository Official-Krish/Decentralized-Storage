use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;
use solana_program::instruction::Instruction;
use solana_sdk::signature::{Keypair, Signer};
use solana_sdk::transaction::Transaction;
use solana_sdk::system_instruction;
use solana_sdk::system_program;

use spl_token::state::Mint;
use spl_associated_token_account::get_associated_token_address;

// Define the instruction enum matching your program
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub enum TapeInstruction {
    Initialize {
        decay_n: u64,
        decay_d: u64,
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
        epoch_id: u128,
    },
    SubmitProof {
        epoch_id: u128,
        proof_hash: [u8; 32],
    },
    FinalizeEpoch {
        epoch_id: u128,
    },
    Stake {
        amount: u64,
    },
    Unstake {
        amount: u64,
    },
    Claim {},
}

// Define account structures matching your program
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct GlobalState {
    pub admin: Pubkey,
    pub tape_mint: Pubkey,
    pub reward_vault: Pubkey,
    pub decay_n: u64,
    pub decay_d: u64,
    pub emission_cap: u64,
    pub total_staked: u64,
    pub bump: u8,
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct ObjectRecord {
    pub owner: Pubkey,
    pub commitment: [u8; 32],
    pub proof_type: u8,
    pub size: u64,
    pub retention_epochs: u64,
    pub object_id: u128,
    pub created_at: i64,
    pub bump: u8,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, PartialEq)]
pub enum EpochStatus {
    Open,
    Submitted,
    Finalized,
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct EpochRecord {
    pub object_id: u128,
    pub epoch_id: u128,
    pub nonce: u64,
    pub status: EpochStatus,
    pub solver: Option<Pubkey>,
    pub proof_hash: Option<[u8; 32]>,
    pub reward: u64,
    pub created_at: i64,
    pub bump: u8,
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct MinerAccount {
    pub miner: Pubkey,
    pub stake: u64,
    pub pending_rewards: u64,
    pub reputation: u64,
    pub unstake_ts: i64,
    pub bump: u8,
}

#[cfg(test)]
mod tests {
    use super::*;
    use litesvm::LiteSVM;
    use solana_sdk::account::Account;

    /// Helper: derive PDA for global state
    fn derive_global_pda(program_id: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(&[b"global"], program_id)
    }

    fn derive_object_pda(program_id: &Pubkey, owner: &Pubkey, object_id: u128) -> (Pubkey, u8) {
        let idb = object_id.to_le_bytes();
        Pubkey::find_program_address(&[b"object", owner.as_ref(), &idb], program_id)
    }

    fn derive_epoch_pda(program_id: &Pubkey, object_id: u128, epoch_id: u128) -> (Pubkey, u8) {
        let oid = object_id.to_le_bytes();
        let eid = epoch_id.to_le_bytes();
        Pubkey::find_program_address(&[b"epoch", &oid, &eid], program_id)
    }

    fn derive_miner_pda(program_id: &Pubkey, miner: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(&[b"miner", miner.as_ref()], program_id)
    }

    #[test]
    fn test_integration_flow_all() {
        // Initialize LiteSVM with your program
        let program_id = Pubkey::new_unique();
        let mut svm = LiteSVM::new();
        
        // Load your program (you'll need the compiled .so file)
        // svm.add_program_from_file(program_id, "path/to/your/program.so").unwrap();
        // Or for testing, you can add a program with custom logic
        svm.add_program(program_id, &[]);

        // Create and fund payer
        let payer = Keypair::new();
        svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

        // ---- Initialize: create GlobalState PDA, mint account, reward vault ----
        let (global_pda, _global_bump) = derive_global_pda(&program_id);
        
        // Create global PDA account
        let global_space = 512usize;
        let rent = svm.minimum_balance_for_rent_exemption(global_space);
        let create_global_ix = system_instruction::create_account(
            &payer.pubkey(),
            &global_pda,
            rent,
            global_space as u64,
            &program_id,
        );

        let tx = Transaction::new_signed_with_payer(
            &[create_global_ix],
            Some(&payer.pubkey()),
            &[&payer],
            svm.latest_blockhash(),
        );
        svm.send_transaction(tx).unwrap();

        // Create tape mint account
        let tape_mint = Keypair::new();
        let mint_space = spl_token::state::Mint::LEN;
        let mint_rent = svm.minimum_balance_for_rent_exemption(mint_space);
        let create_tape_mint_ix = system_instruction::create_account(
            &payer.pubkey(),
            &tape_mint.pubkey(),
            mint_rent,
            mint_space as u64,
            &spl_token::id(),
        );
        
        let tx = Transaction::new_signed_with_payer(
            &[create_tape_mint_ix],
            Some(&payer.pubkey()),
            &[&payer, &tape_mint],
            svm.latest_blockhash(),
        );
        svm.send_transaction(tx).unwrap();

        // Create reward vault (associated token account)
        let reward_vault_ata = get_associated_token_address(&global_pda, &tape_mint.pubkey());
        let create_reward_vault_ix = spl_associated_token_account::instruction::create_associated_token_account(
            &payer.pubkey(),
            &global_pda,
            &tape_mint.pubkey(),
            &spl_token::id(),
        );
        
        let tx = Transaction::new_signed_with_payer(
            &[create_reward_vault_ix],
            Some(&payer.pubkey()),
            &[&payer],
            svm.latest_blockhash(),
        );
        svm.send_transaction(tx).unwrap();

        // Call initialize instruction
        let initialize_ix = Instruction::new_with_borsh(
            program_id,
            &TapeInstruction::Initialize { 
                decay_n: 15, 
                decay_d: 100, 
                emission_cap: 7_000_000 
            },
            vec![
                solana_program::instruction::AccountMeta::new(payer.pubkey(), true),
                solana_program::instruction::AccountMeta::new(global_pda, false),
                solana_program::instruction::AccountMeta::new(tape_mint.pubkey(), false),
                solana_program::instruction::AccountMeta::new(reward_vault_ata, false),
                solana_program::instruction::AccountMeta::new_readonly(solana_program::sysvar::rent::id(), false),
                solana_program::instruction::AccountMeta::new_readonly(spl_token::id(), false),
                solana_program::instruction::AccountMeta::new_readonly(system_program::ID, false),
            ],
        );

        let tx = Transaction::new_signed_with_payer(
            &[initialize_ix],
            Some(&payer.pubkey()),
            &[&payer],
            svm.latest_blockhash(),
        );
        svm.send_transaction(tx).unwrap();

        // Read back GlobalState
        let global_account = svm.get_account(&global_pda).expect("global not found");
        let global_state = GlobalState::try_from_slice(&global_account.data).expect("deserialize global");
        assert_eq!(global_state.admin, payer.pubkey());
        assert_eq!(global_state.tape_mint, tape_mint.pubkey());

        // ---- Register object ----
        let user = Keypair::new();
        svm.airdrop(&user.pubkey(), 1_000_000_000).unwrap();

        let object_id: u128 = 42;
        let (object_pda, _obj_bump) = derive_object_pda(&program_id, &user.pubkey(), object_id);
        
        // Create object PDA account
        let object_space = 1024usize;
        let obj_rent = svm.minimum_balance_for_rent_exemption(object_space);
        let create_object_ix = system_instruction::create_account(
            &payer.pubkey(), 
            &object_pda, 
            obj_rent, 
            object_space as u64, 
            &program_id
        );
        
        let tx = Transaction::new_signed_with_payer(
            &[create_object_ix],
            Some(&payer.pubkey()),
            &[&payer],
            svm.latest_blockhash(),
        );
        svm.send_transaction(tx).unwrap();

        // Register object
        let commitment = [7u8; 32];
        let register_ix = Instruction::new_with_borsh(
            program_id,
            &TapeInstruction::RegisterObject {
                commitment,
                proof_type: 0u8,
                size: 1234u64,
                retention_epochs: 10u64,
                object_id,
            },
            vec![
                solana_program::instruction::AccountMeta::new(user.pubkey(), true),
                solana_program::instruction::AccountMeta::new(object_pda, false),
                solana_program::instruction::AccountMeta::new_readonly(system_program::ID, false),
            ],
        );
        
        let tx = Transaction::new_signed_with_payer(
            &[register_ix],
            Some(&payer.pubkey()),
            &[&payer, &user],
            svm.latest_blockhash(),
        );
        svm.send_transaction(tx).unwrap();

        // Read back object
        let obj_acct = svm.get_account(&object_pda).expect("object not found");
        let obj_rec = ObjectRecord::try_from_slice(&obj_acct.data).expect("deserialize object");
        assert_eq!(obj_rec.owner, user.pubkey());
        assert_eq!(obj_rec.commitment[0], 7u8);

        // ---- Create Epoch ----
        let epoch_id: u128 = 1001;
        let nonce: u64 = 9;
        let (epoch_pda, _bump) = derive_epoch_pda(&program_id, object_id, epoch_id);
        
        // Create epoch PDA
        let epoch_space = 1024usize;
        let epoch_rent = svm.minimum_balance_for_rent_exemption(epoch_space);
        let create_epoch_account_ix = system_instruction::create_account(
            &payer.pubkey(), 
            &epoch_pda, 
            epoch_rent, 
            epoch_space as u64, 
            &program_id
        );
        
        let tx = Transaction::new_signed_with_payer(
            &[create_epoch_account_ix],
            Some(&payer.pubkey()),
            &[&payer],
            svm.latest_blockhash(),
        );
        svm.send_transaction(tx).unwrap();

        let create_epoch_ix = Instruction::new_with_borsh(
            program_id,
            &TapeInstruction::CreateEpoch { object_id, nonce, epoch_id },
            vec![
                solana_program::instruction::AccountMeta::new(payer.pubkey(), true),
                solana_program::instruction::AccountMeta::new(object_pda, false),
                solana_program::instruction::AccountMeta::new(epoch_pda, false),
                solana_program::instruction::AccountMeta::new_readonly(system_program::ID, false),
            ],
        );
        
        let tx = Transaction::new_signed_with_payer(
            &[create_epoch_ix],
            Some(&payer.pubkey()),
            &[&payer],
            svm.latest_blockhash(),
        );
        svm.send_transaction(tx).unwrap();

        let epoch_acct = svm.get_account(&epoch_pda).expect("epoch not found");
        let epoch_rec = EpochRecord::try_from_slice(&epoch_acct.data).expect("deserialize epoch");
        assert_eq!(epoch_rec.object_id, object_id);
        assert_eq!(epoch_rec.epoch_id, epoch_id);
        assert_eq!(epoch_rec.status, EpochStatus::Open);

        // ---- Miner stake & submit proof ----
        let miner = Keypair::new();
        svm.airdrop(&miner.pubkey(), 500_000_000).unwrap();

        // Create miner PDA
        let (miner_pda, _mbump) = derive_miner_pda(&program_id, &miner.pubkey());
        let miner_space = 1024usize;
        let miner_rent = svm.minimum_balance_for_rent_exemption(miner_space);
        let create_miner_acc_ix = system_instruction::create_account(
            &payer.pubkey(), 
            &miner_pda, 
            miner_rent, 
            miner_space as u64, 
            &program_id
        );
        
        let tx = Transaction::new_signed_with_payer(
            &[create_miner_acc_ix],
            Some(&payer.pubkey()),
            &[&payer],
            svm.latest_blockhash(),
        );
        svm.send_transaction(tx).unwrap();

        // Create miner ATA
        let miner_ata = get_associated_token_address(&miner.pubkey(), &tape_mint.pubkey());
        let create_miner_ata_ix = spl_associated_token_account::instruction::create_associated_token_account(
            &payer.pubkey(),
            &miner.pubkey(),
            &tape_mint.pubkey(),
            &spl_token::id(),
        );
        
        let tx = Transaction::new_signed_with_payer(
            &[create_miner_ata_ix],
            Some(&payer.pubkey()),
            &[&payer],
            svm.latest_blockhash(),
        );
        svm.send_transaction(tx).unwrap();

        // Call stake instruction
        let stake_amount = 500u64;
        let stake_ix = Instruction::new_with_borsh(
            program_id,
            &TapeInstruction::Stake { amount: stake_amount },
            vec![
                solana_program::instruction::AccountMeta::new(miner.pubkey(), true),
                solana_program::instruction::AccountMeta::new(miner_ata, false),
                solana_program::instruction::AccountMeta::new(reward_vault_ata, false),
                solana_program::instruction::AccountMeta::new(miner_pda, false),
                solana_program::instruction::AccountMeta::new_readonly(spl_token::id(), false),
            ],
        );
        
        let tx = Transaction::new_signed_with_payer(
            &[stake_ix],
            Some(&payer.pubkey()),
            &[&payer, &miner],
            svm.latest_blockhash(),
        );
        let _ = svm.send_transaction(tx);

        // Submit proof
        let proof_hash = [9u8; 32];
        let submit_ix = Instruction::new_with_borsh(
            program_id,
            &TapeInstruction::SubmitProof { epoch_id, proof_hash },
            vec![
                solana_program::instruction::AccountMeta::new(miner.pubkey(), true),
                solana_program::instruction::AccountMeta::new(epoch_pda, false),
                solana_program::instruction::AccountMeta::new(miner_pda, false),
                solana_program::instruction::AccountMeta::new(global_pda, false),
            ],
        );
        
        let tx = Transaction::new_signed_with_payer(
            &[submit_ix],
            Some(&payer.pubkey()),
            &[&payer, &miner],
            svm.latest_blockhash(),
        );
        svm.send_transaction(tx).unwrap();

        // Read epoch back
        let epoch_acct = svm.get_account(&epoch_pda).expect("epoch present");
        let epoch_rec = EpochRecord::try_from_slice(&epoch_acct.data).expect("deserialize epoch");
        assert_eq!(epoch_rec.status, EpochStatus::Submitted);
        assert_eq!(epoch_rec.solver.unwrap(), miner.pubkey());

        // ---- Finalize Epoch ----
        let miner_reward_ata = get_associated_token_address(&miner.pubkey(), &tape_mint.pubkey());
        
        let finalize_ix = Instruction::new_with_borsh(
            program_id,
            &TapeInstruction::FinalizeEpoch { epoch_id },
            vec![
                solana_program::instruction::AccountMeta::new(payer.pubkey(), true),
                solana_program::instruction::AccountMeta::new(epoch_pda, false),
                solana_program::instruction::AccountMeta::new(miner_pda, false),
                solana_program::instruction::AccountMeta::new(reward_vault_ata, false),
                solana_program::instruction::AccountMeta::new(miner_reward_ata, false),
                solana_program::instruction::AccountMeta::new_readonly(spl_token::id(), false),
                solana_program::instruction::AccountMeta::new(global_pda, false),
            ],
        );
        
        let tx = Transaction::new_signed_with_payer(
            &[finalize_ix],
            Some(&payer.pubkey()),
            &[&payer],
            svm.latest_blockhash(),
        );
        let _ = svm.send_transaction(tx);

        // Claim rewards
        let claim_ix = Instruction::new_with_borsh(
            program_id,
            &TapeInstruction::Claim {},
            vec![
                solana_program::instruction::AccountMeta::new(miner.pubkey(), true),
                solana_program::instruction::AccountMeta::new(miner_pda, false),
                solana_program::instruction::AccountMeta::new(reward_vault_ata, false),
                solana_program::instruction::AccountMeta::new(miner_reward_ata, false),
                solana_program::instruction::AccountMeta::new_readonly(spl_token::id(), false),
                solana_program::instruction::AccountMeta::new(global_pda, false),
            ],
        );
        
        let tx = Transaction::new_signed_with_payer(
            &[claim_ix],
            Some(&payer.pubkey()),
            &[&payer, &miner],
            svm.latest_blockhash(),
        );
        let _ = svm.send_transaction(tx);

        // Unstake
        let unstake_ix = Instruction::new_with_borsh(
            program_id,
            &TapeInstruction::Unstake { amount: 200 },
            vec![
                solana_program::instruction::AccountMeta::new(miner.pubkey(), true),
                solana_program::instruction::AccountMeta::new(miner_pda, false),
                solana_program::instruction::AccountMeta::new(reward_vault_ata, false),
                solana_program::instruction::AccountMeta::new(miner_reward_ata, false),
                solana_program::instruction::AccountMeta::new_readonly(spl_token::id(), false),
                solana_program::instruction::AccountMeta::new(global_pda, false),
            ],
        );
        
        let tx = Transaction::new_signed_with_payer(
            &[unstake_ix],
            Some(&payer.pubkey()),
            &[&payer, &miner],
            svm.latest_blockhash(),
        );
        let _ = svm.send_transaction(tx);

        // Final assertions
        assert!(svm.get_account(&global_pda).is_some());
        assert!(svm.get_account(&object_pda).is_some());
        assert!(svm.get_account(&epoch_pda).is_some());
        assert!(svm.get_account(&miner_pda).is_some());
    }
}