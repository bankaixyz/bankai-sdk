use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ChainInfoDto {
    pub integration_id: u64,
    pub chain_id: u64,
    pub name: String,
    pub active: bool,
}
