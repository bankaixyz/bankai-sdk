//! Bankai block representation.

use alloc::vec::Vec;

use alloy_primitives::{keccak256, FixedBytes, Keccak256};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A Bankai block containing verified beacon and execution chain state
///
/// Each Bankai block is the output of an STWO zero-knowledge proof and contains
/// MMR roots for both the beacon chain and execution layer. These roots establish
/// trust for all headers committed in the MMRs.
///
/// This is the foundation of stateless verification - once you have a verified
/// Bankai block, you can trustlessly verify any header in its MMRs.
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct BankaiBlock {
    /// Bankai program version
    pub version: u64,
    /// Program hash that produced this proof
    pub program_hash: FixedBytes<32>,
    /// Hash of previous Bankai block payload
    #[cfg_attr(feature = "serde", serde(alias = "prev_hash"))]
    pub prev_block_hash: FixedBytes<32>,
    /// Bankai MMR root using Keccak hash
    #[cfg_attr(feature = "serde", serde(alias = "mmr_root_keccak"))]
    pub bankai_mmr_root_keccak: FixedBytes<32>,
    /// Bankai MMR root using Poseidon hash
    #[cfg_attr(feature = "serde", serde(alias = "mmr_root_poseidon"))]
    pub bankai_mmr_root_poseidon: FixedBytes<32>,
    /// Bankai block number (sequential)
    pub block_number: u64,
    /// Beacon chain state at this Bankai block
    pub beacon: BeaconClient,
    /// Execution layer state at this Bankai block
    pub execution: ExecutionClient,
    /// Op chains commitment at this Bankai block
    pub op_chains: OpChainsCommitment,
}

/// Canonical Bankai output returned by verification.
///
/// The API envelope is `{ block_hash, block }`.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct BankaiBlockOutput {
    pub block_hash: FixedBytes<32>,
    pub block: BankaiBlock,
}

/// Canonical Bankai output with the full OP chains payload.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct BankaiBlockFullOutput {
    pub block_hash: FixedBytes<32>,
    pub block: BankaiBlockFull,
}

/// Canonical Cairo public output for Bankai OS.
///
/// Current Cairo public output contains only `block_hash`.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct BankaiBlockHashOutput {
    pub block_hash: FixedBytes<32>,
}

/// Beacon chain state in a Bankai block
///
/// Contains the beacon chain's MMR roots and consensus information.
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct BeaconClient {
    /// Latest beacon slot processed
    pub slot_number: u64,
    /// Beacon block root at this slot
    pub header_root: FixedBytes<32>,
    /// Beacon state root at this slot
    pub state_root: FixedBytes<32>,
    /// Last justified beacon slot
    pub justified_height: u64,
    /// Last finalized beacon slot
    pub finalized_height: u64,
    /// Number of validators that signed
    pub num_signers: u64,
    /// MMR root using Keccak hash
    pub mmr_root_keccak: FixedBytes<32>,
    /// MMR root using Poseidon hash
    pub mmr_root_poseidon: FixedBytes<32>,
    /// Current validator root
    pub current_validator_root: FixedBytes<32>,
    /// Next validator root
    pub next_validator_root: FixedBytes<32>,
}

/// Execution layer state in a Bankai block
///
/// Contains the execution chain's MMR roots and block information.
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ExecutionClient {
    /// Latest execution block processed
    pub block_number: u64,
    /// Block hash at this height
    pub header_hash: FixedBytes<32>,
    /// Last justified block height
    pub justified_height: u64,
    /// Last finalized block height
    pub finalized_height: u64,
    /// MMR root using Keccak hash
    pub mmr_root_keccak: FixedBytes<32>,
    /// MMR root using Poseidon hash
    pub mmr_root_poseidon: FixedBytes<32>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct OpChainsCommitment {
    pub root: FixedBytes<32>,
    pub n_clients: u64,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct OpChainClient {
    pub chain_id: u64,
    pub block_number: u64,
    pub header_hash: FixedBytes<32>,
    pub l1_submission_block: u64,
    pub mmr_root_keccak: FixedBytes<32>,
    pub mmr_root_poseidon: FixedBytes<32>,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct BankaiBlockFull {
    /// Bankai program version
    pub version: u64,
    /// Program hash that produced this proof
    pub program_hash: FixedBytes<32>,
    /// Hash of previous Bankai block payload
    #[cfg_attr(feature = "serde", serde(alias = "prev_hash"))]
    pub prev_block_hash: FixedBytes<32>,
    /// Bankai MMR root using Keccak hash
    #[cfg_attr(feature = "serde", serde(alias = "mmr_root_keccak"))]
    pub bankai_mmr_root_keccak: FixedBytes<32>,
    /// Bankai MMR root using Poseidon hash
    #[cfg_attr(feature = "serde", serde(alias = "mmr_root_poseidon"))]
    pub bankai_mmr_root_poseidon: FixedBytes<32>,
    /// Bankai block number (sequential)
    pub block_number: u64,
    /// Beacon chain state at this Bankai block
    pub beacon: BeaconClient,
    /// Execution layer state at this Bankai block
    pub execution: ExecutionClient,
    /// OP chains full outputs committed by this block
    pub op_chains: Vec<OpChainClient>,
}

fn compute_op_chains_merkle_root(leaves: &[FixedBytes<32>]) -> FixedBytes<32> {
    if leaves.is_empty() {
        return FixedBytes::from([0u8; 32]);
    }

    let mut level = leaves.to_vec();
    level.resize(level.len().next_power_of_two(), FixedBytes::from([0u8; 32]));

    while level.len() > 1 {
        let mut next = Vec::with_capacity(level.len() / 2);
        for pair in level.chunks_exact(2) {
            let mut preimage = [0u8; 64];
            preimage[..32].copy_from_slice(pair[0].as_slice());
            preimage[32..].copy_from_slice(pair[1].as_slice());
            next.push(FixedBytes::from_slice(keccak256(preimage).as_slice()));
        }
        level = next;
    }

    level[0]
}

impl BankaiBlock {
    /// Compute the canonical Bankai block hash.
    ///
    /// Hash input is 22 ordered 32-byte big-endian words that match Cairo hints.
    pub fn compute_block_hash_keccak(&self) -> FixedBytes<32> {
        fn u64_to_word(value: u64) -> [u8; 32] {
            let mut out = [0u8; 32];
            out[24..32].copy_from_slice(&value.to_be_bytes());
            out
        }

        fn bytes32_to_word(value: &FixedBytes<32>) -> [u8; 32] {
            let mut out = [0u8; 32];
            out.copy_from_slice(value.as_slice());
            out
        }

        let words = [
            u64_to_word(self.version),
            bytes32_to_word(&self.program_hash),
            bytes32_to_word(&self.prev_block_hash),
            bytes32_to_word(&self.bankai_mmr_root_poseidon),
            bytes32_to_word(&self.bankai_mmr_root_keccak),
            u64_to_word(self.block_number),
            u64_to_word(self.beacon.slot_number),
            bytes32_to_word(&self.beacon.header_root),
            bytes32_to_word(&self.beacon.state_root),
            u64_to_word(self.beacon.justified_height),
            u64_to_word(self.beacon.finalized_height),
            u64_to_word(self.beacon.num_signers),
            bytes32_to_word(&self.beacon.mmr_root_keccak),
            bytes32_to_word(&self.beacon.mmr_root_poseidon),
            bytes32_to_word(&self.beacon.current_validator_root),
            bytes32_to_word(&self.beacon.next_validator_root),
            u64_to_word(self.execution.block_number),
            bytes32_to_word(&self.execution.header_hash),
            u64_to_word(self.execution.justified_height),
            u64_to_word(self.execution.finalized_height),
            bytes32_to_word(&self.execution.mmr_root_keccak),
            bytes32_to_word(&self.execution.mmr_root_poseidon),
            bytes32_to_word(&self.op_chains.root),
            u64_to_word(self.op_chains.n_clients),
        ];

        let mut preimage = Vec::with_capacity(words.len() * 32);

        for word in words {
            preimage.extend_from_slice(&word);
        }

        FixedBytes::from_slice(keccak256(preimage).as_slice())
    }
}

impl OpChainClient {
    pub fn hash(&self) -> FixedBytes<32> {
        let mut hasher = Keccak256::new();
        hasher.update(self.chain_id.to_be_bytes());
        hasher.update(self.block_number.to_be_bytes());
        hasher.update(self.header_hash.as_slice());
        hasher.update(self.l1_submission_block.to_be_bytes());
        hasher.update(self.mmr_root_keccak.as_slice());
        hasher.update(self.mmr_root_poseidon.as_slice());

        hasher.finalize()
    }

    pub fn commitment_leaf_hash(&self) -> FixedBytes<32> {
        if self.chain_id == 0
            && self.block_number == 0
            && self.header_hash == FixedBytes::from([0u8; 32])
            && self.l1_submission_block == 0
            && self.mmr_root_keccak == FixedBytes::from([0u8; 32])
            && self.mmr_root_poseidon == FixedBytes::from([0u8; 32])
        {
            FixedBytes::from([0u8; 32])
        } else {
            self.hash()
        }
    }
}

impl BankaiBlockFull {
    pub fn to_block(&self) -> BankaiBlock {
        let leaves = self
            .op_chains
            .iter()
            .map(OpChainClient::commitment_leaf_hash)
            .collect::<Vec<_>>();
        let root = compute_op_chains_merkle_root(&leaves);

        BankaiBlock {
            version: self.version,
            program_hash: self.program_hash,
            prev_block_hash: self.prev_block_hash,
            bankai_mmr_root_keccak: self.bankai_mmr_root_keccak,
            bankai_mmr_root_poseidon: self.bankai_mmr_root_poseidon,
            block_number: self.block_number,
            beacon: self.beacon.clone(),
            execution: self.execution.clone(),
            op_chains: OpChainsCommitment {
                root,
                n_clients: self.op_chains.len() as u64,
            },
        }
    }
}

impl BankaiBlockHashOutput {
    #[cfg(feature = "inputs")]
    pub fn from_verification_output(output: &cairo_air::utils::VerificationOutput) -> Option<Self> {
        let output = &output.output;
        if output.len() < 2 {
            return None;
        }

        fn bytes32_from_limbs(low: &[u8], high: &[u8]) -> FixedBytes<32> {
            let mut bytes = [0u8; 32];
            bytes[0..16].copy_from_slice(high);
            bytes[16..32].copy_from_slice(low);
            FixedBytes::from(bytes)
        }

        let get_felt = |idx: usize| output.get(idx).map(|felt| felt.to_bytes_be());

        Some(Self {
            block_hash: bytes32_from_limbs(&get_felt(0)?[16..], &get_felt(1)?[16..]),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{
        BankaiBlockFull, BeaconClient, ExecutionClient, OpChainClient, OpChainsCommitment,
    };
    use alloy_primitives::FixedBytes;

    #[test]
    fn single_op_chain_round_trips_into_block_commitment() {
        let op_chain = OpChainClient {
            chain_id: 10,
            block_number: 42,
            header_hash: FixedBytes::from([7u8; 32]),
            l1_submission_block: 99,
            mmr_root_keccak: FixedBytes::from([8u8; 32]),
            mmr_root_poseidon: FixedBytes::from([9u8; 32]),
        };
        let full = BankaiBlockFull {
            version: 1,
            program_hash: FixedBytes::from([1u8; 32]),
            prev_block_hash: FixedBytes::from([2u8; 32]),
            bankai_mmr_root_keccak: FixedBytes::from([3u8; 32]),
            bankai_mmr_root_poseidon: FixedBytes::from([4u8; 32]),
            block_number: 5,
            beacon: BeaconClient::default(),
            execution: ExecutionClient::default(),
            op_chains: vec![op_chain.clone()],
        };

        let block = full.to_block();

        assert_eq!(
            block.op_chains,
            OpChainsCommitment {
                root: op_chain.commitment_leaf_hash(),
                n_clients: 1,
            }
        );
        assert_eq!(
            block.compute_block_hash_keccak(),
            full.to_block().compute_block_hash_keccak()
        );
    }
}
