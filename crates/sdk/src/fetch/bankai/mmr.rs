use bankai_types::api::proofs::{MmrProofDto, MmrProofRequestDto};

use crate::errors::SdkResult;
use crate::fetch::clients::bankai_api::ApiClient;

pub async fn fetch_mmr_proof(
    client: &ApiClient,
    request: &MmrProofRequestDto,
) -> SdkResult<MmrProofDto> {
    let proof = client.get_mmr_proof(request).await?;
    Ok(proof)
}
