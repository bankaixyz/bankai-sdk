use crate::errors::SdkResult;
use crate::fetch::clients::bankai_api::ApiClient;
use cairo_air::CairoProof;
use serde_json::Value;
use starknet_ff::FieldElement;
use stwo::core::vcs::blake2_merkle::Blake2sMerkleHasher;
use stwo_cairo_serialize::deserialize::CairoDeserialize;

pub fn parse_block_proof_value(
    value: serde_json::Value,
) -> SdkResult<CairoProof<Blake2sMerkleHasher>> {
    // Try JSON struct first
    if let Ok(parsed) = serde_json::from_value::<CairoProof<Blake2sMerkleHasher>>(value.clone()) {
        Ok(parsed)
    } else {
        let data: Vec<FieldElement> = value
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap().parse().unwrap())
            .collect();
        let res = CairoProof::<Blake2sMerkleHasher>::deserialize(&mut data.iter());
        Ok(res)
    }
}

pub async fn fetch_block_proof(
    client: &ApiClient,
    block_number: u64,
) -> SdkResult<CairoProof<Blake2sMerkleHasher>> {
    let proof = client.get_block_proof(block_number).await?;
    let value: Value = proof.proof;
    parse_block_proof_value(value)
}
