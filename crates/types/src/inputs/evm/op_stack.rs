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
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub header_proof: Vec<OpStackHeaderProof>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub account_proof: Vec<AccountProof>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub storage_slot_proof: Vec<StorageSlotProof>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tx_proof: Vec<TxProof>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
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
