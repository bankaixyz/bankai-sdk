//! Bankai block representation.

use alloy_primitives::{FixedBytes, keccak256};
use cairo_air::utils::VerificationOutput;

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
#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
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

        let mut preimage = Vec::with_capacity(22 * 32);
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
        ];

        for word in words {
            preimage.extend_from_slice(&word);
        }

        FixedBytes::from_slice(keccak256(preimage).as_slice())
    }

    /// Deprecated for current Cairo output shape.
    ///
    /// Cairo output now returns only `block_hash`, so full block reconstruction
    /// from verification output is intentionally unsupported.
    pub fn from_verication_output(output: &VerificationOutput) -> Option<Self> {
        let _ = output;
        None
    }
}

impl BankaiBlockHashOutput {
    pub fn from_verication_output(output: &VerificationOutput) -> Option<Self> {
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
