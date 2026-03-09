extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

use alloy_primitives::{FixedBytes, hex::FromHex};
use alloy_rpc_types_eth::Header as ExecutionHeader;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::api::op_stack::OpMerkleProofDto;
use crate::block::OpChainClient;
use crate::inputs::evm::{
    MmrProof,
    execution::{AccountProof, ReceiptProof, StorageSlotProof, TxProof},
};

fn serialize_execution_header<S>(header: &ExecutionHeader, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let json_str = serde_json::to_string(header).map_err(serde::ser::Error::custom)?;
    json_str.serialize(serializer)
}

fn deserialize_execution_header<'de, D>(deserializer: D) -> Result<ExecutionHeader, D::Error>
where
    D: Deserializer<'de>,
{
    let json_str = String::deserialize(deserializer)?;
    serde_json::from_str(&json_str).map_err(serde::de::Error::custom)
}

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Clone, Serialize, Deserialize)]
pub struct OpStackProofs {
    pub header_proof: Option<Vec<OpStackHeaderProof>>,
    pub account_proof: Option<Vec<AccountProof>>,
    pub storage_slot_proof: Option<Vec<StorageSlotProof>>,
    pub tx_proof: Option<Vec<TxProof>>,
    pub receipt_proof: Option<Vec<ReceiptProof>>,
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

impl From<OpMerkleProofDto> for OpStackMerkleProof {
    fn from(value: OpMerkleProofDto) -> Self {
        OpStackMerkleProof {
            chain_id: value.chain_id,
            merkle_leaf_index: value.merkle_leaf_index,
            leaf_hash: FixedBytes::from_hex(value.leaf_hash).unwrap(),
            root: FixedBytes::from_hex(value.root).unwrap(),
            path: value
                .path
                .into_iter()
                .map(|node| FixedBytes::from_hex(node).unwrap())
                .collect(),
        }
    }
}

#[cfg(test)]
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

        let proof = OpStackMerkleProof::from(dto);

        assert_eq!(proof.chain_id, 10);
        assert_eq!(proof.merkle_leaf_index, 2);
        assert_eq!(proof.path.len(), 1);
    }
}
