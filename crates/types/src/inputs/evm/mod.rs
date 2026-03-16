extern crate alloc;

use alloc::vec::Vec;

#[cfg(feature = "api")]
use alloy_primitives::hex::FromHex;
use alloy_primitives::FixedBytes;
use serde::{Deserialize, Serialize};

#[cfg(feature = "api")]
use crate::api::proofs::MmrProofDto;
use crate::common::HashingFunction;
use crate::inputs::evm::{
    beacon::BeaconHeaderProof,
    execution::{AccountProof, ExecutionHeaderProof, ReceiptProof, StorageSlotProof, TxProof},
};

pub mod beacon;
pub mod execution;
pub(crate) mod header_serde;
pub mod op_stack;

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Default, Serialize, Deserialize)]
pub struct EvmProofs {
    #[serde(default)]
    pub execution_header_proof: Vec<ExecutionHeaderProof>,
    #[serde(default)]
    pub beacon_header_proof: Vec<BeaconHeaderProof>,
    #[serde(default)]
    pub account_proof: Vec<AccountProof>,
    #[serde(default)]
    pub storage_slot_proof: Vec<StorageSlotProof>,
    #[serde(default)]
    pub tx_proof: Vec<TxProof>,
    #[serde(default)]
    pub receipt_proof: Vec<ReceiptProof>,
}

impl EvmProofs {
    pub fn is_empty(&self) -> bool {
        self.execution_header_proof.is_empty()
            && self.beacon_header_proof.is_empty()
            && self.account_proof.is_empty()
            && self.storage_slot_proof.is_empty()
            && self.tx_proof.is_empty()
            && self.receipt_proof.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::{EvmProofs, MmrProof};
    use crate::common::HashingFunction;
    use crate::inputs::evm::execution::{ExecutionHeaderProof, StorageSlotEntry, StorageSlotProof};
    use alloy_primitives::{Address, Bytes, FixedBytes, U256};
    use alloy_rpc_types_eth::{Account, Header as ExecutionHeader};

    fn sample_mmr_proof() -> MmrProof {
        MmrProof {
            network_id: 1,
            block_number: 42,
            hashing_function: HashingFunction::Keccak,
            header_hash: FixedBytes::from([1u8; 32]),
            root: FixedBytes::from([2u8; 32]),
            elements_index: 3,
            elements_count: 4,
            path: vec![FixedBytes::from([5u8; 32])],
            peaks: vec![FixedBytes::from([6u8; 32])],
        }
    }

    fn sample_execution_header_proof() -> ExecutionHeaderProof {
        ExecutionHeaderProof {
            header: ExecutionHeader::default(),
            mmr_proof: sample_mmr_proof(),
        }
    }

    fn sample_storage_slot_proof() -> StorageSlotProof {
        StorageSlotProof {
            account: Account::default(),
            address: Address::repeat_byte(0x11),
            network_id: 1,
            block_number: 42,
            state_root: FixedBytes::from([7u8; 32]),
            account_mpt_proof: vec![Bytes::from(vec![1u8, 2, 3])],
            slots: vec![StorageSlotEntry {
                slot_key: U256::from(1u64),
                slot_value: U256::from(2u64),
                storage_mpt_proof: vec![Bytes::from(vec![4u8, 5, 6])],
            }],
        }
    }

    #[test]
    fn evm_proofs_bincode_roundtrip_with_empty_middle_fields() {
        let proofs = EvmProofs {
            execution_header_proof: vec![sample_execution_header_proof()],
            storage_slot_proof: vec![sample_storage_slot_proof()],
            ..Default::default()
        };

        let bytes = bincode::serialize(&proofs).expect("failed to serialize EvmProofs");
        let decoded: EvmProofs =
            bincode::deserialize(&bytes).expect("failed to deserialize EvmProofs");

        assert_eq!(decoded.execution_header_proof.len(), 1);
        assert!(decoded.beacon_header_proof.is_empty());
        assert!(decoded.account_proof.is_empty());
        assert_eq!(decoded.storage_slot_proof.len(), 1);
        assert!(decoded.tx_proof.is_empty());
        assert!(decoded.receipt_proof.is_empty());
    }
}

#[cfg(feature = "api")]
impl TryFrom<MmrProofDto> for MmrProof {
    type Error = alloy_primitives::hex::FromHexError;

    fn try_from(mmr_proof: MmrProofDto) -> Result<Self, Self::Error> {
        Ok(MmrProof {
            network_id: mmr_proof.network_id,
            block_number: mmr_proof.block_number,
            hashing_function: mmr_proof.hashing_function,
            header_hash: FixedBytes::from_hex(mmr_proof.header_hash)?,
            root: FixedBytes::from_hex(mmr_proof.root)?,
            elements_index: mmr_proof.elements_index,
            elements_count: mmr_proof.elements_count,
            path: mmr_proof
                .path
                .iter()
                .map(FixedBytes::from_hex)
                .collect::<Result<Vec<_>, _>>()?,
            peaks: mmr_proof
                .peaks
                .iter()
                .map(FixedBytes::from_hex)
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Clone, Serialize, Deserialize)]
pub struct MmrProof {
    pub network_id: u64,
    pub block_number: u64,
    pub hashing_function: HashingFunction,
    pub header_hash: FixedBytes<32>,
    pub root: FixedBytes<32>,
    pub elements_index: u64,
    pub elements_count: u64,
    pub path: Vec<FixedBytes<32>>,
    pub peaks: Vec<FixedBytes<32>>,
}
