use alloc::string::String;

use alloy_rpc_types_eth::Header as ExecutionHeader;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

pub fn serialize_execution_header<S>(
    header: &ExecutionHeader,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let json_str = serde_json::to_string(header).map_err(serde::ser::Error::custom)?;
    json_str.serialize(serializer)
}

pub fn deserialize_execution_header<'de, D>(
    deserializer: D,
) -> Result<ExecutionHeader, D::Error>
where
    D: Deserializer<'de>,
{
    let json_str = String::deserialize(deserializer)?;
    serde_json::from_str(&json_str).map_err(serde::de::Error::custom)
}
