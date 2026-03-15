use std::sync::Arc;

use bankai_types::api::explorer::ExplorerOverviewDto;

use crate::errors::SdkResult;
use crate::fetch::api::{ApiCore, handle_response};

pub struct ExplorerApi {
    core: Arc<ApiCore>,
}

impl ExplorerApi {
    pub(crate) fn new(core: Arc<ApiCore>) -> Self {
        Self { core }
    }

    /// Fetch the explorer overview payload.
    pub async fn overview(&self) -> SdkResult<ExplorerOverviewDto> {
        let url = format!("{}/v1/explorer/overview", self.core.base_url);
        let response = self.core.client.get(&url).send().await?;
        handle_response(response).await
    }
}
