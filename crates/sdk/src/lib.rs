//! Bankai fetches proof bundles from the Bankai API and local RPC providers.
//!
//! The intended flow is:
//!
//! 1. configure [`Bankai`]
//! 2. build a batch with [`Bankai::init_batch`]
//! 3. call `.execute()` to get a [`ProofBundle`]
//! 4. verify the bundle with `bankai-verify`
//!
//! ```no_run
//! use alloy_primitives::Address;
//! use bankai_sdk::{Bankai, HashingFunction, Network};
//! use bankai_verify::verify_batch_proof;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let bankai = Bankai::new(
//!         Network::Sepolia,
//!         Some("https://sepolia.infura.io/v3/YOUR_KEY".to_string()),
//!         Some("https://sepolia.beacon-api.example.com".to_string()),
//!         None,
//!     );
//!
//!     let proof_bundle = bankai
//!         .init_batch(None, HashingFunction::Keccak)
//!         .await?
//!         .ethereum_execution_header(9_231_247)
//!         .ethereum_account(9_231_247, Address::ZERO)
//!         .execute()
//!         .await?;
//!
//!     let results = verify_batch_proof(proof_bundle)?;
//!     println!("Verified block {}", results.evm.execution_header[0].number);
//!     println!("Verified balance {}", results.evm.account[0].balance);
//!     Ok(())
//! }
//! ```
//!
//! Guides:
//!
//! - [SDK quickstart](https://github.com/bankaixyz/bankai-docs/blob/main/content/docs/sdk/quickstart.mdx)
//! - [Proof bundles](https://github.com/bankaixyz/bankai-docs/blob/main/content/docs/sdk/proof-bundles.mdx)
//! - [API client](https://github.com/bankaixyz/bankai-docs/blob/main/content/docs/sdk/api-client.mdx)

/// SDK error types and result aliases.
pub mod errors;

mod debug;

use std::collections::BTreeMap;

// Re-export common types from bankai_types
pub use crate::fetch::evm::op_stack::OpStackChainFetcher;
pub use bankai_types::common::HashingFunction;
pub use bankai_types::inputs::ProofBundle;

pub use crate::fetch::api::blocks::parse_block_proof_payload;

// ============================================================================
// Network Configuration
// ============================================================================

/// Supported blockchain networks
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Network {
    /// Ethereum Sepolia testnet
    Sepolia,
    /// Local API
    Local,
}

impl Network {
    /// Returns the API base URL for this network
    pub fn api_url(&self) -> &'static str {
        match self {
            Network::Sepolia => "https://sepolia.api.bankai.xyz",
            Network::Local => "http://localhost:8080",
        }
    }

    /// Returns the beacon chain network ID (always 0)
    pub const fn beacon_network_id(&self) -> u64 {
        0
    }

    /// Returns the default execution layer chain ID for this network.
    pub const fn execution_network_id(&self) -> u64 {
        match self {
            Network::Sepolia => 11155111,
            Network::Local => 11155111,
        }
    }
}

// ============================================================================
// Public API Components
// ============================================================================

/// API client for Bankai's low-level HTTP endpoints.
pub use crate::fetch::api::ApiClient;

/// Batch proof generation for the fetch-then-verify flow.
///
/// All requests in a batch share the same Bankai block and block proof.
pub mod batch {

    pub use crate::fetch::batch::ProofBatchBuilder;
}

// Keep fetch module private (internal implementation details)
mod fetch;

use crate::errors::{SdkError, SdkResult};
use crate::fetch::evm::{beacon::BeaconChainFetcher, execution::ExecutionChainFetcher};

// ============================================================================
// Main SDK Struct
// ============================================================================

/// Namespace for Ethereum-related operations
struct EthereumNamespace {
    execution: Option<ExecutionChainFetcher>,
    beacon: Option<BeaconChainFetcher>,
}

struct OpStackNamespace {
    chains: BTreeMap<String, OpStackChainFetcher>,
}

/// Main entry point for configuring RPCs, fetching proof bundles, and accessing the API client.
pub struct Bankai {
    /// Direct access to the Bankai API client
    pub api: ApiClient,
    /// Ethereum execution and beacon chain fetchers (internal)
    ethereum: EthereumNamespace,
    op_stack: OpStackNamespace,
    network: Network,
}

impl Bankai {
    /// Creates a new SDK instance using the default API URL for `network`.
    ///
    /// Provide only the RPCs you need for the proofs you plan to request.
    /// OP Stack RPCs are configured as `chain name -> rpc url`.
    pub fn new(
        network: Network,
        ethereum_execution_rpc: Option<String>,
        ethereum_beacon_rpc: Option<String>,
        op_stack_execution_rpcs: Option<BTreeMap<String, String>>,
    ) -> Self {
        Self::new_with_base_url(
            network,
            network.api_url().to_string(),
            ethereum_execution_rpc,
            ethereum_beacon_rpc,
            op_stack_execution_rpcs,
        )
    }

    /// Creates a new SDK instance with an explicit Bankai API base URL.
    pub fn new_with_base_url(
        network: Network,
        api_base_url: String,
        ethereum_execution_rpc: Option<String>,
        ethereum_beacon_rpc: Option<String>,
        op_stack_execution_rpcs: Option<BTreeMap<String, String>>,
    ) -> Self {
        let api = ApiClient::new_with_base_url(api_base_url);
        let execution = ethereum_execution_rpc.map(|rpc| {
            ExecutionChainFetcher::new(api.clone(), rpc, network.execution_network_id())
        });
        let beacon = ethereum_beacon_rpc
            .map(|rpc| BeaconChainFetcher::new(api.clone(), rpc, network.beacon_network_id()));
        let op_stack = OpStackNamespace {
            chains: op_stack_execution_rpcs
                .unwrap_or_default()
                .into_iter()
                .map(|(chain_name, rpc)| {
                    let fetcher = OpStackChainFetcher::new(api.clone(), chain_name.clone(), rpc);
                    (chain_name, fetcher)
                })
                .collect(),
        };

        Bankai {
            api: api.clone(),
            ethereum: EthereumNamespace { execution, beacon },
            op_stack,
            network,
        }
    }

    /// Returns the network this SDK instance is configured for
    pub fn network(&self) -> Network {
        self.network
    }

    /// Returns the configured OP Stack fetcher for `chain_name`.
    ///
    /// The name must match the key passed in `op_stack_execution_rpcs`.
    pub fn op_stack(&self, chain_name: &str) -> SdkResult<&OpStackChainFetcher> {
        self.op_stack
            .chains
            .get(chain_name)
            .ok_or_else(|| SdkError::NotConfigured(format!("OP Stack fetcher for {chain_name}")))
    }

    /// Starts a proof batch anchored to a Bankai block.
    ///
    /// The batch inherits the network configured on this [`Bankai`] instance.
    ///
    /// Pass `None` to use the latest completed Bankai block.
    pub async fn init_batch(
        &self,
        bankai_block_number: Option<u64>,
        hashing: HashingFunction,
    ) -> SdkResult<batch::ProofBatchBuilder<'_>> {
        let block_number = match bankai_block_number {
            Some(bn) => bn,
            None => self.api.blocks().latest_number().await?,
        };
        Ok(batch::ProofBatchBuilder::new(self, block_number, hashing))
    }

    pub(crate) fn ethereum(&self) -> &EthereumNamespace {
        &self.ethereum
    }
}

impl EthereumNamespace {
    /// Get the execution layer fetcher
    pub(crate) fn execution(&self) -> SdkResult<&ExecutionChainFetcher> {
        self.execution
            .as_ref()
            .ok_or_else(|| SdkError::NotConfigured("Ethereum execution fetcher".to_string()))
    }

    /// Get the beacon chain fetcher
    pub(crate) fn beacon(&self) -> SdkResult<&BeaconChainFetcher> {
        self.beacon
            .as_ref()
            .ok_or_else(|| SdkError::NotConfigured("Ethereum beacon fetcher".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::{Bankai, Network};
    use crate::errors::SdkError;

    #[test]
    fn op_stack_fetcher_requires_configuration() {
        let sdk = Bankai::new(Network::Local, None, None, None);
        assert!(matches!(
            sdk.op_stack("base"),
            Err(SdkError::NotConfigured(_))
        ));
    }
}
