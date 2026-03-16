extern crate alloc;

use alloc::vec::Vec;

#[cfg(feature = "api")]
use alloy_primitives::hex::FromHex;
use alloy_primitives::FixedBytes;
use alloy_rpc_types_eth::Header as ExecutionHeader;
use serde::{Deserialize, Serialize};

#[cfg(feature = "api")]
use crate::api::op_stack::OpMerkleProofDto;
use crate::block::OpChainClient;
use crate::inputs::evm::header_serde::{deserialize_execution_header, serialize_execution_header};
use crate::inputs::evm::{
    execution::{AccountProof, ReceiptProof, StorageSlotProof, TxProof},
    MmrProof,
};

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Clone, Default, Serialize, Deserialize)]
pub struct OpStackProofs {
    #[serde(default)]
    pub header_proof: Vec<OpStackHeaderProof>,
    #[serde(default)]
    pub account_proof: Vec<AccountProof>,
    #[serde(default)]
    pub storage_slot_proof: Vec<StorageSlotProof>,
    #[serde(default)]
    pub tx_proof: Vec<TxProof>,
    #[serde(default)]
    pub receipt_proof: Vec<ReceiptProof>,
}

impl OpStackProofs {
    pub fn is_empty(&self) -> bool {
        self.header_proof.is_empty()
            && self.account_proof.is_empty()
            && self.storage_slot_proof.is_empty()
            && self.tx_proof.is_empty()
            && self.receipt_proof.is_empty()
    }
}

#[cfg(test)]
mod roundtrip_tests {
    use super::{OpStackHeaderProof, OpStackMerkleProof, OpStackProofs};
    use crate::block::OpChainClient;
    use crate::common::HashingFunction;
    use crate::inputs::evm::execution::{StorageSlotEntry, StorageSlotProof};
    use crate::inputs::evm::MmrProof;
    use alloy_primitives::{Address, Bytes, FixedBytes, U256};
    use alloy_rpc_types_eth::{Account, Header as ExecutionHeader};

    fn sample_mmr_proof() -> MmrProof {
        MmrProof {
            network_id: 10,
            block_number: 99,
            hashing_function: HashingFunction::Keccak,
            header_hash: FixedBytes::from([1u8; 32]),
            root: FixedBytes::from([2u8; 32]),
            elements_index: 3,
            elements_count: 4,
            path: vec![FixedBytes::from([5u8; 32])],
            peaks: vec![FixedBytes::from([6u8; 32])],
        }
    }

    fn sample_header_proof() -> OpStackHeaderProof {
        OpStackHeaderProof {
            header: ExecutionHeader::default(),
            snapshot: OpChainClient {
                chain_id: 10,
                block_number: 99,
                header_hash: FixedBytes::from([7u8; 32]),
                l1_submission_block: 123,
                mmr_root_keccak: FixedBytes::from([8u8; 32]),
                mmr_root_poseidon: FixedBytes::from([9u8; 32]),
            },
            merkle_proof: OpStackMerkleProof {
                chain_id: 10,
                merkle_leaf_index: 0,
                leaf_hash: FixedBytes::from([10u8; 32]),
                root: FixedBytes::from([11u8; 32]),
                path: vec![FixedBytes::from([12u8; 32])],
            },
            mmr_proof: sample_mmr_proof(),
        }
    }

    fn sample_storage_slot_proof() -> StorageSlotProof {
        StorageSlotProof {
            account: Account::default(),
            address: Address::repeat_byte(0x22),
            network_id: 10,
            block_number: 99,
            state_root: FixedBytes::from([13u8; 32]),
            account_mpt_proof: vec![Bytes::from(vec![1u8, 2, 3])],
            slots: vec![StorageSlotEntry {
                slot_key: U256::from(3u64),
                slot_value: U256::from(4u64),
                storage_mpt_proof: vec![Bytes::from(vec![4u8, 5, 6])],
            }],
        }
    }

    #[test]
    fn op_stack_proofs_bincode_roundtrip_with_empty_middle_fields() {
        let proofs = OpStackProofs {
            header_proof: vec![sample_header_proof()],
            storage_slot_proof: vec![sample_storage_slot_proof()],
            ..Default::default()
        };

        let bytes = bincode::serialize(&proofs).expect("failed to serialize OpStackProofs");
        let decoded: OpStackProofs =
            bincode::deserialize(&bytes).expect("failed to deserialize OpStackProofs");

        assert_eq!(decoded.header_proof.len(), 1);
        assert!(decoded.account_proof.is_empty());
        assert_eq!(decoded.storage_slot_proof.len(), 1);
        assert!(decoded.tx_proof.is_empty());
        assert!(decoded.receipt_proof.is_empty());
    }
}

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Clone, Serialize, Deserialize)]
pub struct OpStackHeaderProof {
    #[serde(
        serialize_with = "serialize_execution_header",
        deserialize_with = "deserialize_execution_header"
    )]
    pub header: ExecutionHeader,
    pub snapshot: OpChainClient,
    pub merkle_proof: OpStackMerkleProof,
    pub mmr_proof: MmrProof,
}

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Clone, Serialize, Deserialize)]
pub struct OpStackMerkleProof {
    pub chain_id: u64,
    pub merkle_leaf_index: u64,
    pub leaf_hash: FixedBytes<32>,
    pub root: FixedBytes<32>,
    pub path: Vec<FixedBytes<32>>,
}

#[cfg(feature = "api")]
impl TryFrom<OpMerkleProofDto> for OpStackMerkleProof {
    type Error = alloy_primitives::hex::FromHexError;

    fn try_from(value: OpMerkleProofDto) -> Result<Self, Self::Error> {
        Ok(OpStackMerkleProof {
            chain_id: value.chain_id,
            merkle_leaf_index: value.merkle_leaf_index,
            leaf_hash: FixedBytes::from_hex(value.leaf_hash)?,
            root: FixedBytes::from_hex(value.root)?,
            path: value
                .path
                .into_iter()
                .map(FixedBytes::from_hex)
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

#[cfg(all(test, feature = "api"))]
mod tests {
    use super::OpStackMerkleProof;
    use crate::api::op_stack::OpMerkleProofDto;

    #[test]
    fn converts_op_merkle_proof_dto() {
        let dto = OpMerkleProofDto {
            bankai_block_number: 5,
            chain_id: 10,
            merkle_leaf_index: 2,
            leaf_hash: "0x1111111111111111111111111111111111111111111111111111111111111111"
                .to_string(),
            root: "0x2222222222222222222222222222222222222222222222222222222222222222".to_string(),
            path: vec![
                "0x3333333333333333333333333333333333333333333333333333333333333333".to_string(),
            ],
        };

        let proof = OpStackMerkleProof::try_from(dto).unwrap();

        assert_eq!(proof.chain_id, 10);
        assert_eq!(proof.merkle_leaf_index, 2);
        assert_eq!(proof.path.len(), 1);
    }
}
