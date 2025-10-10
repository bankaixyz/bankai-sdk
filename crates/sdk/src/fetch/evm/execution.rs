use alloy_primitives::Address;
use alloy_rpc_types::EIP1186AccountProofResponse;
pub use alloy_rpc_types::Header as ExecutionHeader;
use bankai_types::api::proofs::{HashingFunctionDto, MmrProofRequestDto};

use crate::errors::SdkResult;
use crate::fetch::{
    bankai,
    clients::{bankai_api::ApiClient, execution_client::ExecutionFetcher},
};
use bankai_types::fetch::evm::execution::ExecutionHeaderProof;

pub struct ExecutionChainFetcher {
    api_client: ApiClient,
    rpc_url: String,
    network_id: u64,
}

impl ExecutionChainFetcher {
    pub fn new(api_client: ApiClient, rpc_url: String, network_id: u64) -> Self {
        Self {
            api_client,
            rpc_url,
            network_id,
        }
    }

    pub async fn header(
        &self,
        block_number: u64,
        hashing_function: HashingFunctionDto,
        bankai_block_number: u64,
    ) -> SdkResult<ExecutionHeaderProof> {
        let header = ExecutionFetcher::new(self.rpc_url.clone())
            .fetch_header(block_number)
            .await?;
        let stwo_proof =
            bankai::stwo::fetch_block_proof(&self.api_client, bankai_block_number).await?;
        let mmr_proof = bankai::mmr::fetch_mmr_proof(
            &self.api_client,
            &MmrProofRequestDto {
                network_id: self.network_id,
                block_number: bankai_block_number,
                hashing_function,
                header_hash: header.hash.to_string(),
            },
        )
        .await?;
        Ok(ExecutionHeaderProof {
            header,
            block_proof: stwo_proof,
            mmr_proof,
        })
    }

    pub async fn account(
        &self,
        block_number: u64,
        address: Address,
        _hashing_function: HashingFunctionDto,
        _bankai_block_number: u64,
    ) -> SdkResult<EIP1186AccountProofResponse> {
        let proof = ExecutionFetcher::new(self.rpc_url.clone())
            .fetch_account_proof(address, block_number)
            .await?;
        Ok(proof)
    }
}
