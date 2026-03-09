use std::sync::Arc;

use bankai_types::api::error::ErrorResponse;

use crate::errors::{SdkError, SdkResult};
use crate::Network;

pub mod blocks;
pub mod chains;
pub mod ethereum;
pub mod health;
pub mod op_stack;
pub mod stats;

/// Low-level client for Bankai HTTP APIs.
///
/// Use this when you need raw endpoint access instead of the batch builder.
#[derive(Clone)]
pub struct ApiClient {
    core: Arc<ApiCore>,
}

pub(crate) struct ApiCore {
    pub(crate) client: reqwest::Client,
    pub(crate) base_url: String,
}

impl ApiClient {
    /// Creates an API client using the default base URL for `network`.
    pub fn new(network: Network) -> Self {
        Self::new_with_base_url(network.api_url())
    }

    /// Creates an API client for an explicit base URL.
    pub fn new_with_base_url(base_url: impl Into<String>) -> Self {
        let base_url = base_url.into().trim_end_matches('/').to_string();
        Self {
            core: Arc::new(ApiCore {
                client: reqwest::Client::new(),
                base_url,
            }),
        }
    }

    /// Access block discovery and block-proof endpoints.
    pub fn blocks(&self) -> blocks::BlocksApi {
        blocks::BlocksApi::new(Arc::clone(&self.core))
    }

    /// Access chain metadata endpoints.
    pub fn chains(&self) -> chains::ChainsApi {
        chains::ChainsApi::new(Arc::clone(&self.core))
    }

    /// Access API health endpoints.
    pub fn health(&self) -> health::HealthApi {
        health::HealthApi::new(Arc::clone(&self.core))
    }

    /// Access network and block statistics endpoints.
    pub fn stats(&self) -> stats::StatsApi {
        stats::StatsApi::new(Arc::clone(&self.core))
    }

    /// Access Ethereum proof endpoints.
    pub fn ethereum(&self) -> ethereum::EthereumApi {
        ethereum::EthereumApi::new(Arc::clone(&self.core))
    }

    /// Access OP Stack proof endpoints.
    pub fn op_stack(&self) -> op_stack::OpStackApi {
        op_stack::OpStackApi::new(Arc::clone(&self.core))
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
