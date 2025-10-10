use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ErrorResponse {
    pub error_id: String,
    pub code: String,
    pub message: String,
    #[serde(default)]
    pub details: serde_json::Value,
}
