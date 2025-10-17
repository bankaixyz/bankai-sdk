use alloy_primitives::hex::ToHexExt;
use bankai_types::api::proofs::MmrProofRequestDto;
use bankai_types::verify::evm::beacon::BeaconHeader;
use bankai_types::{api::proofs::HashingFunctionDto, fetch::evm::beacon::BeaconHeaderProof};
use tree_hash::TreeHash;

use crate::errors::SdkResult;
use crate::fetch::{
    bankai,
    clients::{bankai_api::ApiClient, beacon_client::BeaconFetcher},
};

pub struct BeaconChainFetcher {
    pub api_client: ApiClient,
    pub beacon_client: BeaconFetcher,
    pub network_id: u64,
}

impl BeaconChainFetcher {
    pub fn new(api_client: ApiClient, beacon_rpc: String, network_id: u64) -> Self {
        Self {
            api_client,
            beacon_client: BeaconFetcher::new(beacon_rpc),
            network_id,
        }
    }

    pub async fn header(
        &self,
        slot: u64,
        hashing_function: HashingFunctionDto,
        bankai_block_number: u64,
    ) -> SdkResult<BeaconHeaderProof> {
        let header_response = self.beacon_client.fetch_header(slot).await?;
        let header: BeaconHeader = header_response.into();
        let header_root = header.tree_hash_root();
        let header_root_string = format!("0x{}", header_root.encode_hex());
        let mmr_proof = bankai::mmr::fetch_mmr_proof(
            &self.api_client,
            &MmrProofRequestDto {
                network_id: self.network_id,
                block_number: bankai_block_number,
                hashing_function,
                header_hash: header_root_string,
            },
        )
        .await?;
        Ok(BeaconHeaderProof { header, mmr_proof: mmr_proof.into() })
    }

    pub async fn header_only(&self, slot: u64) -> SdkResult<BeaconHeader> {
        let header_response = self.beacon_client.fetch_header(slot).await?;
        let header: BeaconHeader = header_response.into();
        Ok(header)
    }

    pub fn network_id(&self) -> u64 {
        self.network_id
    }
}
