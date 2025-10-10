use alloy_rpc_types_beacon::header::HeaderResponse;
use anyhow::Error;


pub struct BeaconFetcher {
    pub beacon_rpc: String,
}

impl BeaconFetcher {
    pub fn new(beacon_rpc: String) -> Self {
        Self { beacon_rpc }
    }

    pub async fn fetch_header(&self, slot: u64) -> Result<HeaderResponse, Error> {
        let url = format!("{}/eth/v1/beacon/headers/{}", self.beacon_rpc, slot);
        let client = reqwest::Client::new();
        let response = client
            .get(&url)
            .header("Accept", "application/json")
            .send()
            .await?;
        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(anyhow::anyhow!("Block not found"));
        }

        let header_response = response.json::<HeaderResponse>().await?;

        Ok(header_response)
    }
}