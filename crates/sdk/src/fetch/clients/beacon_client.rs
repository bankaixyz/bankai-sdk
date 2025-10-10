use alloy_rpc_types_beacon::header::HeaderResponse;
use crate::errors::{SdkError, SdkResult};

pub struct BeaconFetcher {
    pub beacon_rpc: String,
}

impl BeaconFetcher {
    pub fn new(beacon_rpc: String) -> Self {
        Self { beacon_rpc }
    }

    pub async fn fetch_header(&self, slot: u64) -> SdkResult<HeaderResponse> {
        let url = format!("{}/eth/v1/beacon/headers/{}", self.beacon_rpc, slot);
        let client = reqwest::Client::new();
        let response = client
            .get(&url)
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(SdkError::from)?;
        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(SdkError::NotFound(format!("beacon header slot {slot} not found")));
        }

        let header_response = response
            .json::<HeaderResponse>()
            .await
            .map_err(SdkError::from)?;

        Ok(header_response)
    }
}
