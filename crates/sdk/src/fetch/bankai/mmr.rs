use anyhow::Error;
use bankai_types::api::{MmrProofDto, MmrProofRequestDto};

use crate::fetch::clients::bankai_api::ApiClient;

pub async fn fetch_mmr_proof(
    client: &ApiClient,
    request: &MmrProofRequestDto,
) -> Result<MmrProofDto, Error> {
    let proof = client.get_mmr_proof(request).await?;
    Ok(proof)
}
