use std::sync::Arc;

use bankai_types::api::error::ErrorResponse;

use crate::errors::{SdkError, SdkResult};
use crate::Network;

pub mod blocks;
pub mod chains;
pub mod ethereum;
pub mod health;
pub mod stats;

#[derive(Clone)]
pub struct ApiClient {
    core: Arc<ApiCore>,
}

pub(crate) struct ApiCore {
    pub(crate) client: reqwest::Client,
    pub(crate) base_url: String,
}

impl ApiClient {
    pub fn new(network: Network) -> Self {
        Self {
            core: Arc::new(ApiCore {
                client: reqwest::Client::new(),
                base_url: network.api_url().to_string(),
            }),
        }
    }

    pub fn blocks(&self) -> blocks::BlocksApi {
        blocks::BlocksApi::new(Arc::clone(&self.core))
    }

    pub fn chains(&self) -> chains::ChainsApi {
        chains::ChainsApi::new(Arc::clone(&self.core))
    }

    pub fn health(&self) -> health::HealthApi {
        health::HealthApi::new(Arc::clone(&self.core))
    }

    pub fn stats(&self) -> stats::StatsApi {
        stats::StatsApi::new(Arc::clone(&self.core))
    }

    pub fn ethereum(&self) -> ethereum::EthereumApi {
        ethereum::EthereumApi::new(Arc::clone(&self.core))
    }
}

pub(crate) async fn handle_response<T: serde::de::DeserializeOwned>(
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
