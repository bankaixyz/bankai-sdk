use thiserror::Error;

use reqwest::StatusCode;

use bankai_types::api::error::ErrorResponse;

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum SdkError {
    #[error("network error: {0}")]
    Transport(#[from] reqwest::Error),

    #[error("serialization error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("invalid input: {0}")]
    InvalidInput(String),

    #[error("provider error: {0}")]
    Provider(String),

    #[error("beacon error: {0}")]
    Beacon(String),

    #[error("api error ({status}): {body}")]
    Api { status: StatusCode, body: String },

    #[error("api error response: {code} - {message}")]
    ApiErrorResponse {
        code: String,
        message: String,
        error_id: String,
    },

    #[error("verification error: {0}")]
    Verification(String),

    #[error("not found: {0}")]
    NotFound(String),

    #[error("other error: {0}")]
    Other(String),
}

pub type SdkResult<T> = Result<T, SdkError>;

impl From<ErrorResponse> for SdkError {
    fn from(e: ErrorResponse) -> Self {
        SdkError::ApiErrorResponse {
            code: e.code,
            message: e.message,
            error_id: e.error_id,
        }
    }
}
