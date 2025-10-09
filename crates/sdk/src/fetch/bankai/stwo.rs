use anyhow::Error;
use cairo_air::CairoProof;
use stwo::core::vcs::blake2_merkle::Blake2sMerkleHasher;

use crate::fetch::clients::bankai_api::ApiClient;

pub async fn fetch_block_proof(
    client: &ApiClient,
    block_number: u64,
) -> Result<CairoProof<Blake2sMerkleHasher>, Error> {
    let proof = client.get_zk_proof(block_number).await?;
    let proof: CairoProof<Blake2sMerkleHasher> = serde_json::from_value(proof.proof)?;
    Ok(proof)
}
