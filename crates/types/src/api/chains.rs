use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ChainEcosystemDto {
    Ethereum,
    Bitcoin,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ChainTypeDto {
    ConsensusLayer,
    ExecutionLayer,
    OpStack,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ChainInfoDto {
    pub integration_id: u64,
    pub chain_id: u64,
    pub name: String,
    pub ecosystem: ChainEcosystemDto,
    pub chain_type: ChainTypeDto,
    pub active: bool,
    pub parent_chain_id: Option<u64>,
    pub activation_block_height: Option<u64>,
}
