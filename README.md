# DECENTRALISED STORAGE: Decentralized Data Proof Network

## Overview
DECENTRALISED STORAGE is a decentralized storage verification layer built entirely on the Solana blockchain.  
It enables users to register, verify, and retrieve data through compact on-chain proofs rather than storing full files directly on-chain.

The system ensures that data stored by off-chain nodes (miners) remains **available**, **untampered**, and **verifiable**, while keeping the blockchain lightweight and inexpensive.

---

## Core Concept
Unlike traditional decentralized storage (which stores large data chunks on-chain or across peers), DECENTRALISED STORAGE stores **only cryptographic proofs** of data integrity on-chain.

Each data object is represented by a *commitment* — a small hash or proof that uniquely identifies the file.  
Miners periodically prove that they still hold the corresponding data by submitting compact proofs to the blockchain.

This creates a verifiable record of **data existence and persistence** without ever uploading the full data to Solana.

---

## System Components

### 1. On-Chain Program
The on-chain smart contract maintains verifiable state and economic logic:
- **Global Registry:** Tracks all registered objects, their commitments, owners, and metadata.
- **Epoch Scheduler:** Defines proof intervals and challenge periods.
- **Proof Verifier:** Validates submitted proofs (compact hash checks or zk-proofs).
- **Reward Engine:** Distributes rewards to miners who successfully complete proof verifications.
- **Stake Manager:** Handles collateral, slashing, and emission control.

The contract is deterministic, auditable, and stores only minimal data required to prove correctness.

---

### 2. Off-Chain Storage Nodes (Miners)
Miners are independent nodes that:
1. Store compressed copies of data files off-chain.
2. Periodically receive or generate *proof challenges*.
3. Compute a proof of availability or integrity for their assigned data.
4. Submit compact proofs to the blockchain for verification.

Miners earn rewards for successful proofs and risk losing stake for dishonest or missing submissions.

---

### 3. Indexer / Listener Service
The indexer is an off-chain service that:
- Listens to blockchain events (proofs, challenges, rewards).
- Maintains a searchable, queryable off-chain mirror of on-chain state.
- Provides APIs to miners, frontends, and monitoring dashboards.

It does **not** store user data; it only synchronizes metadata and proof states for discoverability and coordination.

---

## Data Integrity Model

### 1. Commitments
When a user uploads a file, the uploader:
1. Compresses the file using a deterministic algorithm.
2. Computes a **content hash** (e.g., SHA-256 or BLAKE3) of the compressed bytes.
3. Registers that hash (the *commitment*) on-chain with metadata such as file size, timestamp, and storage policy.

This ensures that:
- The on-chain record uniquely identifies the file.
- Any modification to the file changes its hash and invalidates the commitment.

### 2. Proof of Storage / Availability
Miners periodically prove they still possess the original file by submitting a **storage proof**.

There are two verification options:
- **Compact Hash Proofs:** Miners submit small deterministic hashes based on nonce challenges.  
  The contract checks the hashes match expected commitments.
- **Succinct Cryptographic Proofs (SNARK/STARK):** Miners generate zero-knowledge proofs that demonstrate file possession without revealing data.  
  The verifier checks the proof on-chain.

Each proof is tied to:
- A **commitment** (registered file)
- A **nonce or epoch ID**
- A **timestamp**

This binds the proof to a specific time and data version.

### 3. Challenge and Dispute Process
The system maintains integrity through economic and community mechanisms:
1. **Challenge Creation:** The network (or indexer) issues challenges at fixed intervals.
2. **Proof Submission:** Miners respond with proof-of-storage.
3. **Dispute Window:** Other participants can challenge invalid proofs by submitting counter-evidence.
4. **Resolution:** The contract validates or slashes dishonest miners accordingly.

### 4. Immutable Proof History
Every proof and challenge is stored on-chain as a verifiable event log.  
Anyone can audit the full history of:
- Who stored which data
- When proofs were made
- Whether proofs were accepted or rejected
- How rewards and penalties were applied

This ensures full transparency and tamper-resistance.

---

## Maintaining Data Integrity

DECENTRALISED STORAGE enforces data integrity through a combination of **cryptography**, **consensus**, and **incentives**:

| Layer | Mechanism | Purpose |
|--------|------------|---------|
| **Cryptographic** | File hash commitments & zk-proofs | Detects any tampering or loss of data |
| **Blockchain Consensus** | Solana ledger immutability | Guarantees that proof records cannot be altered |
| **Economic Incentives** | Rewards and slashing | Aligns miner behavior with honest verification |
| **Auditable Logs** | On-chain event history | Enables independent verification and reputation tracking |

Together, these layers make it nearly impossible for any participant to fake or erase data without being detected.

---

## System Flow Summary

1. **Register Object**
   - Uploader compresses file → computes hash → registers commitment on-chain.

2. **Store Data**
   - File is stored by miners off-chain.

3. **Issue Challenge**
   - The network (or indexer) issues periodic proof challenges for registered objects.

4. **Submit Proof**
   - Miners compute and submit compact proofs on-chain.

5. **Verify & Reward**
   - Smart contract verifies proof → mints or transfers reward tokens.

6. **Dispute / Slash**
   - Invalid proofs can be challenged → dishonest miners lose stake.

7. **Retrieve Data**
   - Users request data via miner nodes or gateways using the file’s hash/CID.

---

## Key Properties

- **Verifiable:** Every file and proof can be validated cryptographically.
- **Efficient:** Only small commitments and proofs are stored on-chain.
- **Tamper-Proof:** All actions are logged immutably in the Solana ledger.
- **Decentralized:** No central authority controls storage or verification.
- **Transparent:** Anyone can audit proof history and network performance.

---

## Summary

DECENTRALISED STORAGE transforms decentralized storage from a *data hosting* model into a *data proof* model.  
By storing only verifiable proofs on Solana and using miners to maintain off-chain data, it achieves:

- Scalable and trustless data verification  
- Cost-efficient on-chain integrity tracking  
- Long-term sustainability through aligned incentives  

This architecture ensures that users can always prove their data exists and remains unaltered — without ever relying on a central storage provider.

---
