//! Bankai block representation.

use alloc::vec::Vec;

use alloy_primitives::{keccak256, FixedBytes};
use bankai_core::merkle::op_stack;

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

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct OpChainsCommitment {
    pub root: FixedBytes<32>,
    pub n_clients: u64,
}

pub const OP_STACK_TREE_DEPTH: usize = 5;
pub const OP_STACK_MAX_CLIENTS: usize = 1 << OP_STACK_TREE_DEPTH;

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
pub struct IndexedOpChainClient {
    pub merkle_index: u64,
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub client: OpChainClient,
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
    pub op_chains: Vec<IndexedOpChainClient>,
}

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

pub fn empty_op_chains_root() -> FixedBytes<32> {
    op_stack::empty_root()
}

impl Default for OpChainsCommitment {
    fn default() -> Self {
        Self {
            root: empty_op_chains_root(),
            n_clients: 0,
        }
    }
}

impl BankaiBlock {
    /// Compute the canonical Bankai block hash.
    ///
    /// Hash input is 22 ordered 32-byte big-endian words that match Cairo hints.
    pub fn compute_block_hash_keccak(&self) -> FixedBytes<32> {
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
    pub fn empty_leaf_hash() -> FixedBytes<32> {
        op_stack::empty_leaf_hash()
    }

    pub fn hash(&self) -> FixedBytes<32> {
        op_stack::leaf_hash(
            self.chain_id,
            self.block_number,
            self.header_hash,
            self.l1_submission_block,
            self.mmr_root_keccak,
            self.mmr_root_poseidon,
        )
    }

    pub fn commitment_leaf_hash(&self) -> FixedBytes<32> {
        op_stack::leaf_hash(
            self.chain_id,
            self.block_number,
            self.header_hash,
            self.l1_submission_block,
            self.mmr_root_keccak,
            self.mmr_root_poseidon,
        )
    }
}

impl BankaiBlockFull {
    pub fn to_block(&self) -> BankaiBlock {
        let leaf_hashes = self
            .op_chains
            .iter()
            .enumerate()
            .map(|(index, entry)| {
                assert_eq!(
                    entry.merkle_index, index as u64,
                    "invalid OP stack client payload in BankaiBlockFull"
                );
                entry.client.commitment_leaf_hash()
            })
            .collect::<Vec<_>>();
        let root = op_stack::compute_root(&leaf_hashes)
            .expect("invalid OP stack client payload in BankaiBlockFull");
        let n_clients = leaf_hashes.len() as u64;

        BankaiBlock {
            version: self.version,
            program_hash: self.program_hash,
            prev_block_hash: self.prev_block_hash,
            bankai_mmr_root_keccak: self.bankai_mmr_root_keccak,
            bankai_mmr_root_poseidon: self.bankai_mmr_root_poseidon,
            block_number: self.block_number,
            beacon: self.beacon.clone(),
            execution: self.execution.clone(),
            op_chains: OpChainsCommitment { root, n_clients },
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
        empty_op_chains_root, BankaiBlockFull, BeaconClient, ExecutionClient, IndexedOpChainClient,
        OpChainClient, OpChainsCommitment,
    };
    use alloy_primitives::{hex::FromHex, keccak256, FixedBytes};
    use bankai_core::merkle::op_stack;

    fn u64_word(value: u64) -> [u8; 32] {
        let mut out = [0u8; 32];
        out[24..32].copy_from_slice(&value.to_be_bytes());
        out
    }

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
            op_chains: vec![IndexedOpChainClient {
                merkle_index: 0,
                client: op_chain.clone(),
            }],
        };

        let block = full.to_block();
        let expected_leaves = vec![op_chain.commitment_leaf_hash()];

        assert_eq!(
            block.op_chains,
            OpChainsCommitment {
                root: op_stack::compute_root(&expected_leaves).unwrap(),
                n_clients: 1,
            }
        );
        assert_eq!(
            block.compute_block_hash_keccak(),
            full.to_block().compute_block_hash_keccak()
        );
    }

    #[test]
    fn op_chain_hash_uses_be_uint256_words() {
        let op_chain = OpChainClient {
            chain_id: 0x0102_0304_0506_0708,
            block_number: 0x1112_1314_1516_1718,
            header_hash: FixedBytes::from([0x22; 32]),
            l1_submission_block: 0x2122_2324_2526_2728,
            mmr_root_keccak: FixedBytes::from([0x33; 32]),
            mmr_root_poseidon: FixedBytes::from([0x44; 32]),
        };

        let mut preimage = Vec::with_capacity(32 * 6);
        preimage.extend_from_slice(&u64_word(op_chain.chain_id));
        preimage.extend_from_slice(&u64_word(op_chain.block_number));
        preimage.extend_from_slice(op_chain.header_hash.as_slice());
        preimage.extend_from_slice(&u64_word(op_chain.l1_submission_block));
        preimage.extend_from_slice(op_chain.mmr_root_keccak.as_slice());
        preimage.extend_from_slice(op_chain.mmr_root_poseidon.as_slice());

        assert_eq!(
            op_chain.hash(),
            FixedBytes::from_slice(keccak256(preimage).as_slice())
        );
    }

    #[test]
    fn zero_op_chain_uses_hashed_empty_leaf() {
        let empty = OpChainClient {
            chain_id: 0,
            block_number: 0,
            header_hash: FixedBytes::ZERO,
            l1_submission_block: 0,
            mmr_root_keccak: FixedBytes::ZERO,
            mmr_root_poseidon: FixedBytes::ZERO,
        };

        assert_eq!(
            empty.commitment_leaf_hash(),
            OpChainClient::empty_leaf_hash()
        );
        assert_ne!(OpChainClient::empty_leaf_hash(), FixedBytes::ZERO);
    }

    #[test]
    fn op_chains_merkle_root_pads_with_hashed_empty_leaf() {
        let leaves = vec![
            FixedBytes::from([0x55; 32]),
            FixedBytes::from([0x66; 32]),
            FixedBytes::from([0x77; 32]),
        ];
        let empty_leaf = OpChainClient::empty_leaf_hash();
        let mut compact_level = vec![leaves[0], leaves[1], leaves[2], empty_leaf];
        while compact_level.len() > 1 {
            let mut next = Vec::with_capacity(compact_level.len() / 2);
            for pair in compact_level.chunks_exact(2) {
                let mut preimage = [0u8; 64];
                preimage[..32].copy_from_slice(pair[0].as_slice());
                preimage[32..].copy_from_slice(pair[1].as_slice());
                next.push(FixedBytes::from_slice(keccak256(preimage).as_slice()));
            }
            compact_level = next;
        }

        assert_ne!(op_stack::compute_root(&leaves).unwrap(), compact_level[0]);
    }

    #[test]
    #[should_panic(expected = "invalid OP stack client payload in BankaiBlockFull")]
    fn bankai_block_full_rejects_non_sequential_indices() {
        let first = OpChainClient {
            chain_id: 10,
            block_number: 42,
            header_hash: FixedBytes::from([7u8; 32]),
            l1_submission_block: 99,
            mmr_root_keccak: FixedBytes::from([8u8; 32]),
            mmr_root_poseidon: FixedBytes::from([9u8; 32]),
        };
        let second = OpChainClient {
            chain_id: 8453,
            block_number: 100,
            header_hash: FixedBytes::from([1u8; 32]),
            l1_submission_block: 123,
            mmr_root_keccak: FixedBytes::from([2u8; 32]),
            mmr_root_poseidon: FixedBytes::from([3u8; 32]),
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
            op_chains: vec![
                IndexedOpChainClient {
                    merkle_index: 1,
                    client: second.clone(),
                },
                IndexedOpChainClient {
                    merkle_index: 0,
                    client: first.clone(),
                },
            ],
        };

        full.to_block();
    }

    #[test]
    fn bankai_block_full_reconstructs_contiguous_root() {
        let first = OpChainClient {
            chain_id: 10,
            block_number: 42,
            header_hash: FixedBytes::from([7u8; 32]),
            l1_submission_block: 99,
            mmr_root_keccak: FixedBytes::from([8u8; 32]),
            mmr_root_poseidon: FixedBytes::from([9u8; 32]),
        };
        let second = OpChainClient {
            chain_id: 8453,
            block_number: 100,
            header_hash: FixedBytes::from([1u8; 32]),
            l1_submission_block: 123,
            mmr_root_keccak: FixedBytes::from([2u8; 32]),
            mmr_root_poseidon: FixedBytes::from([3u8; 32]),
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
            op_chains: vec![
                IndexedOpChainClient {
                    merkle_index: 0,
                    client: first.clone(),
                },
                IndexedOpChainClient {
                    merkle_index: 1,
                    client: second.clone(),
                },
            ],
        };

        let leaves = vec![first.commitment_leaf_hash(), second.commitment_leaf_hash()];

        let block = full.to_block();
        assert_eq!(
            block.op_chains.root,
            op_stack::compute_root(&leaves).unwrap()
        );
        assert_eq!(block.op_chains.n_clients, 2);
    }

    #[test]
    fn op_chains_commitment_default_uses_fixed_empty_root() {
        assert_eq!(OpChainsCommitment::default().root, empty_op_chains_root());
        assert_eq!(
            OpChainsCommitment::default().root,
            op_stack::compute_root(&[]).unwrap()
        );
        assert_ne!(OpChainsCommitment::default().root, FixedBytes::ZERO);
        assert_eq!(OpChainsCommitment::default().n_clients, 0);
    }

    #[test]
    fn empty_op_chains_root_matches_cairo_constant() {
        let expected = FixedBytes::<32>::from_hex(
            "0xd686d974150e54f427421b5805b6464c7736dcf70944067195505a19e433d326",
        )
        .unwrap();

        assert_eq!(empty_op_chains_root(), expected);
    }
}
