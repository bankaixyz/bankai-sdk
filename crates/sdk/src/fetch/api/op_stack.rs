use std::sync::Arc;
use std::time::Instant;

use bankai_types::api::ethereum::{BankaiBlockFilterDto, HeightDto};
use bankai_types::api::op_stack::{
    OpChainSnapshotSummaryDto, OpMerkleProofDto, OpStackLightClientProofDto,
    OpStackLightClientProofRequestDto, OpStackMerkleProofRequestDto, OpStackMmrProofDto,
    OpStackMmrProofRequestDto,
};

use crate::debug;
use crate::errors::SdkResult;
use crate::fetch::api::{handle_response, ApiCore};

pub struct OpStackApi {
    core: Arc<ApiCore>,
}

impl OpStackApi {
    pub(crate) fn new(core: Arc<ApiCore>) -> Self {
        Self { core }
    }

    /// Fetch the resolved OP stack submission height for a chain and selector/filter.
    pub async fn height(&self, name: &str, filter: &BankaiBlockFilterDto) -> SdkResult<HeightDto> {
        let url = format!("{}/v1/op/{}/height", self.core.base_url, name);
        let response = self.core.client.get(&url).query(filter).send().await?;
        handle_response(response).await
    }

    /// Fetch the OP stack snapshot for a chain and selector/filter.
    pub async fn snapshot(
        &self,
        name: &str,
        filter: &BankaiBlockFilterDto,
    ) -> SdkResult<OpChainSnapshotSummaryDto> {
        let url = format!("{}/v1/op/{}/snapshot", self.core.base_url, name);
        let response = self.core.client.get(&url).query(filter).send().await?;
        handle_response(response).await
    }

    /// Generate a merkle proof for a chain's OP output decommitment.
    pub async fn merkle_proof(
        &self,
        name: &str,
        request: &OpStackMerkleProofRequestDto,
    ) -> SdkResult<OpMerkleProofDto> {
        let url = format!("{}/v1/op/{}/merkle_proof", self.core.base_url, name);
        let response = self.core.client.post(&url).json(request).send().await?;
        handle_response(response).await
    }

    /// Generate an OP stack MMR proof bundle.
    pub async fn mmr_proof(
        &self,
        name: &str,
        request: &OpStackMmrProofRequestDto,
    ) -> SdkResult<OpStackMmrProofDto> {
        let url = format!("{}/v1/op/{}/mmr_proof", self.core.base_url, name);
        let response = self.core.client.post(&url).json(request).send().await?;
        handle_response(response).await
    }

    /// Generate a full OP stack light client proof bundle.
    pub async fn light_client_proof(
        &self,
        name: &str,
        request: &OpStackLightClientProofRequestDto,
    ) -> SdkResult<OpStackLightClientProofDto> {
        let url = format!("{}/v1/op/{}/light_client_proof", self.core.base_url, name);
        let start = Instant::now();
        let result = async {
            let response = self.core.client.post(&url).json(request).send().await?;
            handle_response(response).await
        }
        .await;
        debug::log_result(
            format!(
                "api POST /v1/op/{name}/light_client_proof headers={}",
                request.header_hashes.len()
            ),
            start,
            &result,
        );
        result
    }
}
