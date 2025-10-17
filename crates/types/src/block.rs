//! Bankai block representation
//!
//! A Bankai block represents a verified state of both the beacon chain and execution layer,
//! containing their respective MMR roots that can be used to verify individual headers.

use alloy_primitives::FixedBytes;
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
    /// Bankai block number (sequential)
    pub block_number: u64,
    /// Beacon chain state at this Bankai block
    pub beacon: BeaconClient,
    /// Execution layer state at this Bankai block
    pub execution: ExecutionClient,
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
    /// Hash of current validator committee
    pub current_committee_hash: FixedBytes<32>,
    /// Hash of next validator committee
    pub next_committee_hash: FixedBytes<32>,
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
    pub fn from_verication_output(output: &VerificationOutput) -> Self {
        let output = &output.output;

        fn bytes32_from_limbs(low: &[u8], high: &[u8]) -> FixedBytes<32> {
            let mut bytes = [0u8; 32];
            bytes[0..16].copy_from_slice(high);
            bytes[16..32].copy_from_slice(low);
            FixedBytes::from(bytes)
        }

        Self {
            block_number: output[0].try_into().unwrap(),
            beacon: BeaconClient {
                slot_number: output[1].try_into().unwrap(),
                header_root: bytes32_from_limbs(
                    &output[2].to_bytes_be()[16..],
                    &output[3].to_bytes_be()[16..],
                ),
                justified_height: output[4].try_into().unwrap(),
                finalized_height: output[5].try_into().unwrap(),
                num_signers: output[6].try_into().unwrap(),
                mmr_root_keccak: bytes32_from_limbs(
                    &output[7].to_bytes_be()[16..],
                    &output[8].to_bytes_be()[16..],
                ),
                mmr_root_poseidon: FixedBytes::from_slice(output[9].to_bytes_be().as_slice()),
                current_committee_hash: bytes32_from_limbs(
                    &output[10].to_bytes_be()[16..],
                    &output[11].to_bytes_be()[16..],
                ),
                next_committee_hash: bytes32_from_limbs(
                    &output[12].to_bytes_be()[16..],
                    &output[13].to_bytes_be()[16..],
                ),
            },
            execution: ExecutionClient {
                block_number: output[14].try_into().unwrap(),
                header_hash: bytes32_from_limbs(
                    &output[15].to_bytes_be()[16..],
                    &output[16].to_bytes_be()[16..],
                ),
                justified_height: output[17].try_into().unwrap(),
                finalized_height: output[18].try_into().unwrap(),
                mmr_root_keccak: bytes32_from_limbs(
                    &output[19].to_bytes_be()[16..],
                    &output[20].to_bytes_be()[16..],
                ),
                mmr_root_poseidon: FixedBytes::from_slice(output[21].to_bytes_be().as_slice()),
            },
        }
    }
}
