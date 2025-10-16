use alloy_primitives::{Address, FixedBytes};
use alloy_rpc_types_eth::EIP1186AccountProofResponse;
pub use alloy_rpc_types_eth::Header as ExecutionHeader;
use bankai_types::api::proofs::{HashingFunctionDto, MmrProofRequestDto};

use crate::errors::SdkResult;
use crate::fetch::{
    bankai,
    clients::{bankai_api::ApiClient, execution_client::ExecutionFetcher},
};
use bankai_types::fetch::evm::execution::{ExecutionHeaderProof, TxProof};

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
        let header = ExecutionFetcher::new(self.rpc_url.clone(), self.network_id)
            .fetch_header(block_number)
            .await?;
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
        Ok(ExecutionHeaderProof { header, mmr_proof })
    }

    pub async fn header_only(&self, block_number: u64) -> SdkResult<ExecutionHeader> {
        let header = ExecutionFetcher::new(self.rpc_url.clone(), self.network_id)
            .fetch_header(block_number)
            .await?;
        Ok(header)
    }

    pub fn network_id(&self) -> u64 {
        self.network_id
    }

    pub async fn account(
        &self,
        block_number: u64,
        address: Address,
        _hashing_function: HashingFunctionDto,
        _bankai_block_number: u64,
    ) -> SdkResult<EIP1186AccountProofResponse> {
        let proof = ExecutionFetcher::new(self.rpc_url.clone(), self.network_id)
            .fetch_account_proof(address, block_number)
            .await?;
        Ok(proof)
    }

    pub async fn tx_proof(&self, tx_hash: FixedBytes<32>) -> SdkResult<TxProof> {
        let proof = ExecutionFetcher::new(self.rpc_url.clone(), self.network_id)
            .fetch_tx_proof(tx_hash)
            .await?;
        Ok(proof)
    }
}
