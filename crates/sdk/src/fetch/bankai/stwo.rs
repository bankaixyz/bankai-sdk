use cairo_air::CairoProof;
use stwo::core::vcs::blake2_merkle::Blake2sMerkleHasher;

use crate::fetch::clients::bankai_api::ApiClient;
use crate::errors::{SdkError, SdkResult};

pub async fn fetch_block_proof(
    client: &ApiClient,
    block_number: u64,
) -> SdkResult<CairoProof<Blake2sMerkleHasher>> {
    let proof = client.get_block_proof(block_number).await?;
    let proof: CairoProof<Blake2sMerkleHasher> = serde_json::from_value(proof.proof)
        .map_err(SdkError::from)?;
    Ok(proof)
}
