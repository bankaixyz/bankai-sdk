use std::sync::Arc;

use bankai_types::api::chains::ChainInfoDto;

use crate::errors::SdkResult;
use crate::fetch::api::{handle_response, ApiCore};

pub struct ChainsApi {
    core: Arc<ApiCore>,
}

impl ChainsApi {
    pub(crate) fn new(core: Arc<ApiCore>) -> Self {
        Self { core }
    }

    /// Fetch chain metadata for all supported chains.
    pub async fn list(&self) -> SdkResult<Vec<ChainInfoDto>> {
        let url = format!("{}/v1/chains", self.core.base_url);
        let response = self.core.client.get(&url).send().await?;
        handle_response(response).await
    }

    /// Fetch metadata for a single chain by id.
    pub async fn by_id(&self, chain_id: u64) -> SdkResult<ChainInfoDto> {
        let url = format!("{}/v1/chains/{}", self.core.base_url, chain_id);
        let response = self.core.client.get(&url).send().await?;
        handle_response(response).await
    }
}
