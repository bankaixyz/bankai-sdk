extern crate alloc;
use alloc::vec::Vec;
use alloc::string::String;
use serde::{Deserialize, Serialize, Serializer, Deserializer};

#[cfg(feature = "verifier-types")]
use crate::fetch::evm::MmrProof;
use alloy_primitives::{Address, Bytes, FixedBytes};

#[cfg(feature = "verifier-types")]
use alloy_rpc_types_eth::{Account, Header as ExecutionHeader};

// Custom serialization for ExecutionHeader to work with bincode
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
    // Deserialize as JSON string first
    let json_str = String::deserialize(deserializer)?;
    // Then parse the JSON string to ExecutionHeader
    serde_json::from_str(&json_str).map_err(serde::de::Error::custom)
}

#[cfg(feature = "verifier-types")]
#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Clone, Serialize, Deserialize)]
pub struct ExecutionHeaderProof {
    #[serde(serialize_with = "serialize_execution_header", deserialize_with = "deserialize_execution_header")]
    pub header: ExecutionHeader,
    pub mmr_proof: MmrProof,
}

#[cfg(feature = "verifier-types")]
#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Clone, Serialize, Deserialize)]
pub struct AccountProof {
    pub account: Account,
    pub address: Address,
    pub network_id: u64,
    pub block_number: u64,
    pub state_root: FixedBytes<32>,
    pub mpt_proof: Vec<Bytes>,
}

#[cfg(feature = "verifier-types")]
#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Clone, Serialize, Deserialize)]
pub struct TxProof {
    pub network_id: u64,
    pub block_number: u64,
    pub tx_hash: FixedBytes<32>,
    pub tx_index: u64,
    pub proof: Vec<Bytes>,
    pub encoded_tx: Vec<u8>,
}
