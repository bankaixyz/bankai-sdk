use bankai_types::api::{MmrProofDto, MmrProofRequestDto, ZkProofDto};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("request failed: {0}")]
    RequestFailed(#[from] reqwest::Error),
}

pub struct ApiClient {
    client: reqwest::Client,
    base_url: String,
}

impl ApiClient {
    pub fn new(base_url: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url,
        }
    }

    pub async fn get_zk_proof(&self, block_number: u64) -> Result<ZkProofDto, Error> {
        let url = format!("{}/v1/proofs/zk/{}", self.base_url, block_number);
        let response = self.client.get(&url).send().await?;
        let proof = response.json().await?;
        Ok(proof)
    }

    pub async fn get_mmr_proof(&self, request: &MmrProofRequestDto) -> Result<MmrProofDto, Error> {
        let url = format!("{}/v1/proofs/mmr", self.base_url);
        let response = self.client.post(&url).json(request).send().await?;
        let proof = response.json().await?;
        Ok(proof)
    }
}
