use std::sync::Arc;

use bankai_types::api::blocks::MmrRootsDto;
use bankai_types::api::ethereum::{
    BankaiBlockFilterDto, BeaconSnapshotDto, EthereumLightClientProofRequestDto,
    EthereumMmrProofRequestDto, HeightDto,
};
use bankai_types::api::proofs::{LightClientProofDto, MmrProofDto};

use crate::errors::SdkResult;
use crate::fetch::api::{handle_response, ApiCore};

pub struct BeaconApi {
    core: Arc<ApiCore>,
}

impl BeaconApi {
    pub(crate) fn new(core: Arc<ApiCore>) -> Self {
        Self { core }
    }

    /// Fetch the resolved beacon height for a selector/filter.
    pub async fn height(&self, filter: &BankaiBlockFilterDto) -> SdkResult<HeightDto> {
        let url = format!("{}/v1/ethereum/beacon/height", self.core.base_url);
        let response = self.core.client.get(&url).query(filter).send().await?;
        handle_response(response).await
    }

    /// Fetch the full beacon snapshot for a selector/filter.
    pub async fn snapshot(&self, filter: &BankaiBlockFilterDto) -> SdkResult<BeaconSnapshotDto> {
        let url = format!("{}/v1/ethereum/beacon/snapshot", self.core.base_url);
        let response = self.core.client.get(&url).query(filter).send().await?;
        handle_response(response).await
    }

    /// Fetch the beacon MMR roots for a selector/filter.
    pub async fn mmr_root(&self, filter: &BankaiBlockFilterDto) -> SdkResult<MmrRootsDto> {
        let url = format!("{}/v1/ethereum/beacon/mmr_root", self.core.base_url);
        let response = self.core.client.get(&url).query(filter).send().await?;
        handle_response(response).await
    }

    /// Fetch a beacon MMR proof for a specific header hash.
    pub async fn mmr_proof(&self, request: &EthereumMmrProofRequestDto) -> SdkResult<MmrProofDto> {
        let url = format!("{}/v1/ethereum/beacon/mmr_proof", self.core.base_url);
        let response = self.core.client.post(&url).json(request).send().await?;
        handle_response(response).await
    }

    /// Fetch a beacon light client proof bundle for requested header hashes.
    pub async fn light_client_proof(
        &self,
        request: &EthereumLightClientProofRequestDto,
    ) -> SdkResult<LightClientProofDto> {
        let url = format!(
            "{}/v1/ethereum/beacon/light_client_proof",
            self.core.base_url
        );
        let response = self.core.client.post(&url).json(request).send().await?;
        handle_response(response).await
    }
}
