use std::sync::Arc;

use bankai_types::api::stats::{BlockDetailStatsDto, OverviewStatsDto};

use crate::errors::SdkResult;
use crate::fetch::api::{handle_response, ApiCore};

pub struct StatsApi {
    core: Arc<ApiCore>,
}

impl StatsApi {
    pub(crate) fn new(core: Arc<ApiCore>) -> Self {
        Self { core }
    }

    /// Fetch aggregate overview stats.
    pub async fn overview(&self) -> SdkResult<OverviewStatsDto> {
        let url = format!("{}/v1/stats/overview", self.core.base_url);
        let response = self.core.client.get(&url).send().await?;
        handle_response(response).await
    }

    /// Fetch detailed stats for a specific block.
    pub async fn block_detail(&self, height: u64) -> SdkResult<BlockDetailStatsDto> {
        let url = format!("{}/v1/stats/block/{}", self.core.base_url, height);
        let response = self.core.client.get(&url).send().await?;
        handle_response(response).await
    }
}
