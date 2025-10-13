use crate::errors::{SdkError, SdkResult};
use crate::fetch::batch::ProofBatchBuilder;
use crate::fetch::{
    clients::bankai_api::ApiClient,
    evm::{beacon::BeaconChainFetcher, execution::ExecutionChainFetcher},
};
use crate::verify::evm::{beacon::BeaconVerifier, execution::ExecutionVerifier};
use alloy_rpc_types::Header as ExecutionHeader;
use bankai_types::fetch::evm::{beacon::BeaconHeaderProof, execution::ExecutionHeaderProof};
use bankai_types::api::proofs::HashingFunctionDto;

pub use bankai_types::verify::evm::beacon::BeaconHeader;

pub mod errors;
pub mod fetch;
pub mod verify;

pub struct EvmNamespace {
    execution: Option<ExecutionChainFetcher>,
    beacon: Option<BeaconChainFetcher>,
}

pub struct VerifyNamespace;

pub struct Bankai {
    pub api: ApiClient,
    pub evm: EvmNamespace,
    pub verify: VerifyNamespace,
}

impl Bankai {
    pub fn new(evm_execution_rpc: Option<String>, evm_beacon_rpc: Option<String>) -> Self {
        let api = ApiClient::new();
        let execution = evm_execution_rpc
            .map(|rpc| ExecutionChainFetcher::new(api.clone(), rpc, 1));
        let beacon = evm_beacon_rpc
            .map(|rpc| BeaconChainFetcher::new(api.clone(), rpc, 0));

        Bankai {
            api: api.clone(),
            evm: EvmNamespace { execution, beacon },
            verify: VerifyNamespace,
        }
    }

    pub fn init_batch(
        &self,
        bankai_block_number: u64,
        hashing: HashingFunctionDto,
    ) -> ProofBatchBuilder {
        ProofBatchBuilder::new(self, bankai_block_number, hashing)
    }
}

impl EvmNamespace {
    pub fn execution(&self) -> SdkResult<&ExecutionChainFetcher> {
        self.execution
            .as_ref()
            .ok_or_else(|| SdkError::NotConfigured("EVM execution fetcher".to_string()))
    }

    pub fn beacon(&self) -> SdkResult<&BeaconChainFetcher> {
        self.beacon
            .as_ref()
            .ok_or_else(|| SdkError::NotConfigured("EVM beacon fetcher".to_string()))
    }
}

// impl VerifyNamespace {
//     pub async fn evm_execution_header(
//         &self,
//         proof: &ExecutionHeaderProof,
//     ) -> SdkResult<ExecutionHeader> {
//         ExecutionVerifier::verify_header_proof(proof).await
//     }

//     pub async fn evm_beacon_header(&self, proof: &BeaconHeaderProof) -> SdkResult<BeaconHeader> {
//         BeaconVerifier::verify_header_proof(proof).await
//     }
// }
