use std::sync::Arc;

use bankai_types::api::ethereum::{BankaiBlockFilterDto, EthereumEpochDto, SyncCommitteeKeysDto};

use crate::errors::SdkResult;
use crate::fetch::api::{handle_response, ApiCore};

pub mod beacon;
pub mod execution;

pub struct EthereumApi {
    core: Arc<ApiCore>,
}

impl EthereumApi {
    pub(crate) fn new(core: Arc<ApiCore>) -> Self {
        Self { core }
    }

    pub fn beacon(&self) -> beacon::BeaconApi {
        beacon::BeaconApi::new(Arc::clone(&self.core))
    }

    pub fn execution(&self) -> execution::ExecutionApi {
        execution::ExecutionApi::new(Arc::clone(&self.core))
    }

    /// Fetch the Ethereum epoch for a selector/filter.
    pub async fn epoch(&self, filter: &BankaiBlockFilterDto) -> SdkResult<EthereumEpochDto> {
        let url = format!("{}/v1/ethereum/epoch", self.core.base_url);
        let response = self.core.client.get(&url).query(filter).send().await?;
        handle_response(response).await
    }

    /// Fetch a specific Ethereum epoch by number.
    pub async fn epoch_by_number(&self, number: u64) -> SdkResult<EthereumEpochDto> {
        let url = format!("{}/v1/ethereum/epoch/{}", self.core.base_url, number);
        let response = self.core.client.get(&url).send().await?;
        handle_response(response).await
    }

    /// Fetch sync committee keys by term id.
    pub async fn sync_committee(&self, term_id: u64) -> SdkResult<SyncCommitteeKeysDto> {
        let url = format!("{}/v1/ethereum/sync_committee", self.core.base_url);
        let response = self.core.client.get(&url).query(&[("term_id", term_id)]).send().await?;
        handle_response(response).await
    }
}
