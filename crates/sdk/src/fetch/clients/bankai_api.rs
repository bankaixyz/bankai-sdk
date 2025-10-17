use bankai_types::api::blocks::{BlockStatusDto, BlockSummaryDto, LatestBlockQueryDto};
use bankai_types::api::error::ErrorResponse;
use bankai_types::api::proofs::{
    BankaiBlockProofDto, LightClientProofDto, LightClientProofRequestDto, MmrProofDto,
    MmrProofRequestDto,
};

use crate::errors::{SdkError, SdkResult};
use crate::Network;

/// Client for interacting with the Bankai API
///
/// This client provides access to the Bankai proof generation service, which generates
/// STWO zero-knowledge proofs containing MMRs of blockchain headers. These proofs enable
/// trustless verification of blockchain data.
///
/// # Available Operations
///
/// - **Light Client Proofs**: Fetch complete proof bundles with STWO proof + multiple MMR proofs
/// - **Block Proofs**: Fetch just the STWO proof for a Bankai block
/// - **MMR Proofs**: Fetch individual MMR proofs for specific headers
/// - **Block Queries**: Query latest block numbers and block metadata
///
/// # Example
///
/// ```no_run
/// use bankai_sdk::{ApiClient, Network};
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let api = ApiClient::new(Network::Sepolia);
///     
///     // Get latest block
///     let latest = api.get_latest_block_number().await?;
///     println!("Latest block: {}", latest);
///     
///     // Get block proof
///     let block_proof = api.get_block_proof(latest).await?;
///     
///     Ok(())
/// }
/// ```
#[derive(Clone)]
pub struct ApiClient {
    client: reqwest::Client,
    base_url: String,
}

impl Default for ApiClient {
    fn default() -> Self {
        Self::new(Network::Sepolia)
    }
}

impl ApiClient {
    /// Creates a new API client for the specified network
    ///
    /// The API endpoint is automatically selected based on the network.
    ///
    /// # Arguments
    ///
    /// * `network` - The blockchain network to connect to
    pub fn new(network: Network) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: network.api_url().to_string(),
        }
    }

    async fn handle_response<T: serde::de::DeserializeOwned>(
        &self,
        response: reqwest::Response,
    ) -> SdkResult<T> {
        if response.status().is_success() {
            let value = response.json::<T>().await?;
            return Ok(value);
        }

        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        if let Ok(api_err) = serde_json::from_str::<ErrorResponse>(&body) {
            return Err(SdkError::from(api_err));
        }
        Err(SdkError::Api { status, body })
    }

    /// Fetches a light client proof containing MMR proofs for multiple headers
    ///
    /// This is an optimized endpoint for fetching MMR proofs for multiple blockchain headers
    /// at once, along with the STWO block proof. This is more efficient than requesting
    /// individual MMR proofs when you need to verify multiple headers.
    ///
    /// # Arguments
    ///
    /// * `request` - The light client proof request specifying:
    ///   - `bankai_block_number`: The Bankai block to anchor proofs to
    ///   - `hashing_function`: The hash function to use (Keccak, Poseidon, Blake3)
    ///   - `requested_headers`: List of headers to generate MMR proofs for
    ///
    /// # Returns
    ///
    /// A proof bundle containing:
    /// - STWO block proof with MMRs
    /// - MMR proofs for each requested header to decommit from the MMRs
    pub async fn get_light_client_proof(
        &self,
        request: &LightClientProofRequestDto,
    ) -> SdkResult<LightClientProofDto> {
        let url = format!("{}/v1/proofs/light-client", self.base_url);
        let response = self.client.post(&url).json(request).send().await?;
        self.handle_response(response).await
    }

    /// Fetches the STWO block proof for a specific Bankai block number
    ///
    /// The block proof is an STWO zero-knowledge proof that contains MMRs of valid
    /// blockchain headers. This proof is the foundation for verifying any blockchain
    /// data - headers can be decommitted from the MMRs using MMR proofs.
    ///
    /// # Arguments
    ///
    /// * `block_number` - The Bankai block number to fetch the proof for
    ///
    /// # Returns
    ///
    /// The STWO block proof containing MMRs with commitments to blockchain headers
    pub async fn get_block_proof(&self, block_number: u64) -> SdkResult<BankaiBlockProofDto> {
        let url = format!("{}/v1/proofs/block/{}", self.base_url, block_number);
        let response = self.client.get(&url).send().await?;
        self.handle_response(response).await
    }

    /// Fetches an MMR proof for a specific blockchain header
    ///
    /// The MMR proof enables decommitment of a specific header from the STWO block proof's MMR.
    /// Once decommitted, the header is verified and can be used to verify chain data
    /// (accounts, transactions, storage) via Merkle proofs against the header's roots.
    ///
    /// # Arguments
    ///
    /// * `request` - The MMR proof request specifying:
    ///   - `network_id`: The blockchain network ID (0 = beacon, 1 = execution)
    ///   - `block_number`: The block number of the header
    ///   - `hashing_function`: The hash function to use
    ///   - `header_hash`: The header hash to generate a proof for
    ///
    /// # Returns
    ///
    /// An MMR proof that can decommit the specified header from the MMR
    pub async fn get_mmr_proof(&self, request: &MmrProofRequestDto) -> SdkResult<MmrProofDto> {
        let url = format!("{}/v1/proofs/mmr", self.base_url);
        let response = self.client.post(&url).json(request).send().await?;
        self.handle_response(response).await
    }

    /// Fetches the latest Bankai block number
    ///
    /// This is useful for getting the most recent block number when you want to
    /// anchor proofs to the latest available data. Only returns completed blocks.
    ///
    /// # Returns
    ///
    /// The latest completed Bankai block number
    pub async fn get_latest_block_number(&self) -> SdkResult<u64> {
        let url = format!("{}/v1/blocks/latest", self.base_url);
        let query = LatestBlockQueryDto {
            status: Some(BlockStatusDto::Completed),
        };
        let response = self.client.get(&url).query(&query).send().await?;

        let block_summary: BlockSummaryDto = self.handle_response(response).await?;
        Ok(block_summary.height)
    }
}
