use bankai_types::api::error::ErrorResponse;
use bankai_types::api::proofs::{
    BankaiBlockProofDto, LightClientProofDto, LightClientProofRequestDto, MmrProofDto,
    MmrProofRequestDto,
};

use crate::errors::{SdkError, SdkResult};

#[derive(Clone)]
pub struct ApiClient {
    client: reqwest::Client,
    base_url: String,
}

impl Default for ApiClient {
    fn default() -> Self {
        Self::new()
    }
}

impl ApiClient {
    const DEFAULT_BASE_URL: &'static str = "https://sepolia.api.bankai.xyz";

    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: Self::DEFAULT_BASE_URL.to_string(),
        }
    }

    async fn handle_response<T: serde::de::DeserializeOwned>(
        &self,
        response: reqwest::Response,
    ) -> SdkResult<T> {
        if response.status().is_success() {
            let value = response.json::<T>().await?;
            return Ok(value);
        }

        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        if let Ok(api_err) = serde_json::from_str::<ErrorResponse>(&body) {
            return Err(SdkError::from(api_err));
        }
        Err(SdkError::Api { status, body })
    }

    pub async fn get_light_client_proof(
        &self,
        request: &LightClientProofRequestDto,
    ) -> SdkResult<LightClientProofDto> {
        let url = format!("{}/v1/proofs/light-client", self.base_url);
        let response = self.client.post(&url).json(request).send().await?;
        self.handle_response(response).await
    }

    pub async fn get_block_proof(&self, block_number: u64) -> SdkResult<BankaiBlockProofDto> {
        let url = format!("{}/v1/proofs/block/{}", self.base_url, block_number);
        let response = self.client.get(&url).send().await?;
        self.handle_response(response).await
    }

    pub async fn get_mmr_proof(&self, request: &MmrProofRequestDto) -> SdkResult<MmrProofDto> {
        let url = format!("{}/v1/proofs/mmr", self.base_url);
        let response = self.client.post(&url).json(request).send().await?;
        self.handle_response(response).await
    }
}
