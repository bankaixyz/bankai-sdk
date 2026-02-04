use std::sync::Arc;

use bankai_types::api::blocks::MmrRootsDto;
use bankai_types::api::ethereum::{
    BankaiBlockFilterDto, EthereumLightClientProofRequestDto, EthereumMmrProofRequestDto,
    ExecutionSnapshotDto, HeightDto,
};
use bankai_types::api::proofs::{LightClientProofDto, MmrProofDto};

use crate::errors::SdkResult;
use crate::fetch::api::{handle_response, ApiCore};

pub struct ExecutionApi {
    core: Arc<ApiCore>,
}

impl ExecutionApi {
    pub(crate) fn new(core: Arc<ApiCore>) -> Self {
        Self { core }
    }

    /// Fetch the resolved execution height for a selector/filter.
    pub async fn height(&self, filter: &BankaiBlockFilterDto) -> SdkResult<HeightDto> {
        let url = format!("{}/v1/ethereum/execution/height", self.core.base_url);
        let response = self.core.client.get(&url).query(filter).send().await?;
        handle_response(response).await
    }

    /// Fetch the full execution snapshot for a selector/filter.
    pub async fn snapshot(
        &self,
        filter: &BankaiBlockFilterDto,
    ) -> SdkResult<ExecutionSnapshotDto> {
        let url = format!("{}/v1/ethereum/execution/snapshot", self.core.base_url);
        let response = self.core.client.get(&url).query(filter).send().await?;
        handle_response(response).await
    }

    /// Fetch the execution MMR roots for a selector/filter.
    pub async fn mmr_root(&self, filter: &BankaiBlockFilterDto) -> SdkResult<MmrRootsDto> {
        let url = format!("{}/v1/ethereum/execution/mmr_root", self.core.base_url);
        let response = self.core.client.get(&url).query(filter).send().await?;
        handle_response(response).await
    }

    /// Fetch an execution MMR proof for a specific header hash.
    pub async fn mmr_proof(
        &self,
        request: &EthereumMmrProofRequestDto,
    ) -> SdkResult<MmrProofDto> {
        let url = format!("{}/v1/ethereum/execution/mmr_proof", self.core.base_url);
        let response = self.core.client.post(&url).json(request).send().await?;
        handle_response(response).await
    }

    /// Fetch an execution light client proof bundle for requested header hashes.
    pub async fn light_client_proof(
        &self,
        request: &EthereumLightClientProofRequestDto,
    ) -> SdkResult<LightClientProofDto> {
        let url = format!("{}/v1/ethereum/execution/light_client_proof", self.core.base_url);
        let response = self.core.client.post(&url).json(request).send().await?;
        handle_response(response).await
    }
}
