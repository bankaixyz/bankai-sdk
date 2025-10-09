use alloy_primitives::FixedBytes;
use cairo_air::utils::VerificationOutput;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BankaiBlock {
    pub block_number: u64,
    pub beacon: BeaconClient,
    pub execution: ExecutionClient,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeaconClient {
    pub slot_number: u64,
    pub header_root: FixedBytes<32>, // 2 limbs
    pub justified_height: u64,
    pub finalized_height: u64,
    pub num_signers: u64,
    pub mmr_root_keccak: FixedBytes<32>, // 2 limbs
    pub mmr_root_poseidon: FixedBytes<32>,
    pub current_committee_hash: FixedBytes<32>, // 2 limbs
    pub next_committee_hash: FixedBytes<32>,    // 2 limbs
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionClient {
    pub block_number: u64,
    pub header_hash: FixedBytes<32>, // 2 limbs
    pub justified_height: u64,
    pub finalized_height: u64,
    pub mmr_root_keccak: FixedBytes<32>, // 2 limbs
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
