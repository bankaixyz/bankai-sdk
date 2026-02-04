use std::sync::Arc;

use crate::errors::SdkResult;
use crate::fetch::api::{handle_response, ApiCore};

pub struct HealthApi {
    core: Arc<ApiCore>,
}

#[derive(Debug, serde::Deserialize)]
pub struct HealthResponse {
    pub status: String,
}

impl HealthApi {
    pub(crate) fn new(core: Arc<ApiCore>) -> Self {
        Self { core }
    }

    /// Fetch the service health status.
    pub async fn get(&self) -> SdkResult<HealthResponse> {
        let url = format!("{}/v1/health", self.core.base_url);
        let response = self.core.client.get(&url).send().await?;
        handle_response(response).await
    }
}
