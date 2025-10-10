use crate::errors::SdkResult;
use crate::fetch::{
    clients::bankai_api::ApiClient,
    evm::{beacon::BeaconChainFetcher, execution::ExecutionChainFetcher},
};
use crate::verify::evm::{beacon::BeaconVerifier, execution::ExecutionVerifier};
use alloy_rpc_types::Header as ExecutionHeader;
use bankai_types::fetch::evm::beacon::BeaconHeader;
use bankai_types::fetch::evm::{beacon::BeaconHeaderProof, execution::ExecutionHeaderProof};

pub mod errors;
pub mod fetch;
pub mod verify;

pub struct EvmNamespace {
    pub execution: Option<ExecutionChainFetcher>,
    pub beacon: Option<BeaconChainFetcher>,
}

pub struct VerifyNamespace;

pub struct Bankai {
    pub api: ApiClient,
    pub evm: EvmNamespace,
    pub verify: VerifyNamespace,
}

pub struct BankaiBuilder {
    api_base: String,
    evm_execution: Option<String>,
    evm_beacon: Option<String>,
}

impl Default for BankaiBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl BankaiBuilder {
    pub fn new() -> Self {
        Self {
            api_base: "https://sepolia.api.bankai.xyz".to_string(),
            evm_execution: None,
            evm_beacon: None,
        }
    }

    pub fn with_api_base(mut self, api_base: String) -> Self {
        self.api_base = api_base;
        self
    }

    pub fn with_evm_execution(mut self, rpc: String) -> Self {
        self.evm_execution = Some(rpc);
        self
    }

    pub fn with_evm_beacon(mut self, rpc: String) -> Self {
        self.evm_beacon = Some(rpc);
        self
    }

    pub fn build(self) -> Bankai {
        let api = ApiClient::new(self.api_base);
        let execution = self
            .evm_execution
            .map(|rpc| ExecutionChainFetcher::new(api.clone(), rpc, 1));
        let beacon = self
            .evm_beacon
            .map(|rpc| BeaconChainFetcher::new(api.clone(), rpc, 0));

        Bankai {
            api: api.clone(),
            evm: EvmNamespace { execution, beacon },
            verify: VerifyNamespace,
        }
    }
}

impl Bankai {
    pub fn builder() -> BankaiBuilder {
        BankaiBuilder::new()
    }
}

impl VerifyNamespace {
    pub async fn evm_execution_header(
        &self,
        proof: &ExecutionHeaderProof,
    ) -> SdkResult<ExecutionHeader> {
        ExecutionVerifier::verify_header_proof(proof).await
    }

    pub async fn evm_beacon_header(&self, proof: &BeaconHeaderProof) -> SdkResult<BeaconHeader> {
        BeaconVerifier::verify_header_proof(proof).await
    }
}
