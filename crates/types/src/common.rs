//! Shared Bankai enums used across API, SDK, and verification layers.

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "utoipa")]
use utoipa::ToSchema;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "lowercase"))]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
pub enum ProofFormat {
    Bin,
    Json,
}

impl Default for ProofFormat {
    fn default() -> Self {
        Self::Bin
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
pub enum HashingFunction {
    Keccak,
    Poseidon,
}
