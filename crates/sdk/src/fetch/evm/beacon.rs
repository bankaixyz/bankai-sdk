use alloy_primitives::hex::ToHexExt;
use bankai_types::api::ethereum::{BankaiBlockFilterDto, EthereumMmrProofRequestDto};
use bankai_types::common::HashingFunction;
use bankai_types::inputs::evm::beacon::BeaconHeaderProof;
use bankai_types::results::evm::beacon::BeaconHeader;
use tree_hash::TreeHash;

use crate::errors::SdkResult;
use crate::fetch::{api::ApiClient, clients::beacon_client::BeaconFetcher};

/// Fetcher for Ethereum beacon chain data with MMR proofs
///
/// This fetcher retrieves beacon chain (consensus layer) headers along with MMR proofs
/// needed to decommit headers from STWO proofs.
///
/// The typical flow is:
/// 1. Fetch a beacon header with its MMR proof
/// 2. Use the MMR proof to decommit and verify the header from the STWO block proof
/// 3. The verified beacon header can be used to verify consensus layer data
pub struct BeaconChainFetcher {
    #[allow(dead_code)]
    pub api_client: ApiClient,
    pub beacon_client: BeaconFetcher,
    pub network_id: u64,
}

impl BeaconChainFetcher {
    /// Creates a new beacon chain fetcher
    ///
    /// # Arguments
    ///
    /// * `api_client` - The Bankai API client for fetching MMR proofs
    /// * `beacon_rpc` - The beacon chain API endpoint URL
    /// * `network_id` - The network ID for this chain
    pub fn new(api_client: ApiClient, beacon_rpc: String, network_id: u64) -> Self {
        Self {
            api_client,
            beacon_client: BeaconFetcher::new(beacon_rpc),
            network_id,
        }
    }

    /// Fetches a beacon chain header with its MMR proof
    ///
    /// This retrieves the beacon chain header from the API and generates an MMR proof
    /// that can be used to decommit this header from the STWO block proof's beacon MMR.
    ///
    /// # Arguments
    ///
    /// * `slot` - The beacon chain slot number to fetch
    /// * `hashing_function` - The hash function to use for the MMR proof
    /// * `filter` - Bankai block selector/filter for resolving the snapshot
    ///
    /// # Returns
    ///
    /// A `BeaconHeaderProof` containing the header and MMR proof for decommitment
    #[allow(dead_code)]
    pub async fn header(
        &self,
        slot: u64,
        hashing_function: HashingFunction,
        filter: BankaiBlockFilterDto,
    ) -> SdkResult<BeaconHeaderProof> {
        let header_response = self.beacon_client.fetch_header(slot).await?;
        let header: BeaconHeader = header_response.clone().into();
        let header_root = header.tree_hash_root();
        let header_root_string = format!("0x{}", header_root.encode_hex());
        let request = EthereumMmrProofRequestDto {
            filter,
            hashing_function,
            header_hash: header_root_string,
        };
        let mmr_proof = self
            .api_client
            .ethereum()
            .beacon()
            .mmr_proof(&request)
            .await?;
        Ok(BeaconHeaderProof {
            header: header_response,
            mmr_proof: mmr_proof.try_into().map_err(|e| {
                crate::errors::SdkError::InvalidInput(format!(
                    "invalid beacon MMR proof hex from API: {e}"
                ))
            })?,
        })
    }

    /// Fetches a beacon header without an MMR proof
    ///
    /// Used internally by the batch builder. For verification purposes, use `header()` instead
    /// to get the header with its MMR proof.
    pub async fn header_only(
        &self,
        slot: u64,
    ) -> SdkResult<alloy_rpc_types_beacon::header::HeaderResponse> {
        self.beacon_client.fetch_header(slot).await
    }

    /// Returns the network ID for this fetcher
    pub fn network_id(&self) -> u64 {
        self.network_id
    }
}
