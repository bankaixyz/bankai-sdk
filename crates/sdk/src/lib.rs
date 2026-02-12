//! Bankai SDK
//!
//! **Trustless blockchain data access through zero-knowledge proofs**
//!
//! ## How It Works
//!
//! 1. **Verify the Bankai block proof**: validate the STWO proof to establish trust in the MMR roots
//! 2. **Verify MMR proofs**: decommit and verify headers against those trusted MMR roots
//! 3. **Verify chain data**: verify accounts/transactions/storage against the verified headers
//!
//! ## Getting Started (Fetch + Verify)
//!
//! This example follows the full flow: fetch a proof batch via the SDK, then verify it with
//! `bankai-verify`, and finally use the verified results.
//!
//! ```no_run
//! use alloy_primitives::{Address, FixedBytes, U256};
//! use bankai_sdk::{Bankai, HashingFunctionDto, Network};
//! use bankai_verify::verify_batch_proof;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Step 1: Initialize the SDK
//!     let bankai = Bankai::new(
//!         Network::Sepolia,
//!         Some("https://sepolia.infura.io/v3/YOUR_KEY".to_string()),  // Execution RPC
//!         Some("https://sepolia.beacon-api.example.com".to_string()), // Beacon RPC
//!     );
//!
//!     // Step 2: Build and fetch a batch with multiple proof requests
//!     let proof_batch = bankai
//!         .init_batch(Network::Sepolia, None, HashingFunctionDto::Keccak)
//!         .await?
//!         .ethereum_beacon_header(8_551_383)
//!         .ethereum_execution_header(9_231_247)
//!         .ethereum_account(9_231_247, Address::ZERO)
//!         .ethereum_storage_slot(9_231_247, Address::ZERO, vec![U256::from(0)])
//!         .ethereum_tx(FixedBytes::from([0u8; 32]))
//!         .execute()
//!         .await?;
//!
//!     // Step 3: Verify everything (block proof + MMR proofs + Merkle proofs)
//!     let results = verify_batch_proof(proof_batch)?;
//!
//!     // Step 4: Use the verified data (cryptographically guaranteed valid)
//!     for header in &results.evm.execution_header {
//!         println!("✓ Verified execution header at block {}", header.number);
//!     }
//!     for header in &results.evm.beacon_header {
//!         println!("✓ Verified beacon slot {}", header.slot);
//!     }
//!     for account in &results.evm.account {
//!         println!("✓ Verified account balance: {} wei", account.balance);
//!     }
//!     for slot in &results.evm.storage_slot {
//!         println!("✓ Verified storage slot: {:?}", slot);
//!     }
//!     for tx in &results.evm.tx {
//!         println!("✓ Verified transaction: {:?}", tx);
//!     }
//!
//!     Ok(())
//! }
//! ```
//!
//! # API Client
//!
//! Direct access to the Bankai API for low-level operations:
//!
//! ```no_run
//! use bankai_sdk::{Bankai, Network, HashingFunctionDto};
//! use bankai_types::api::ethereum::{
//!     BankaiBlockFilterDto, EthereumLightClientProofRequestDto, EthereumMmrProofRequestDto,
//! };
//! use bankai_types::api::proofs::ProofFormatDto;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! # let sdk = Bankai::new(Network::Sepolia, None, None);
//!
//! // Get latest Bankai block number
//! let latest_block = sdk.api.blocks().latest_number().await?;
//!
//! // Fetch STWO block proof
//! let block_proof = sdk.api.blocks().proof(latest_block).await?;
//!
//! // Fetch MMR proof for a specific header
//! let filter = BankaiBlockFilterDto::with_bankai_block_number(latest_block);
//! let mmr_request = EthereumMmrProofRequestDto {
//!     filter: filter.clone(),
//!     hashing_function: HashingFunctionDto::Keccak,
//!     header_hash: "0x...".to_string(),
//! };
//! let mmr_proof = sdk.api.ethereum().execution().mmr_proof(&mmr_request).await?;
//!
//! // Fetch batch light client proof (STWO proof + multiple MMR proofs)
//! let lc_request = EthereumLightClientProofRequestDto {
//!     filter,
//!     hashing_function: HashingFunctionDto::Keccak,
//!     header_hashes: vec!["0x...".to_string()],
//!     proof_format: ProofFormatDto::Bin, // default if omitted by backend
//! };
//! let light_client_proof = sdk.api.ethereum().execution().light_client_proof(&lc_request).await?;
//! # Ok(())
//! # }
//! ```

pub mod errors;

// Re-export common types from bankai_types
pub use bankai_types::api::proofs::HashingFunctionDto;
pub use bankai_types::fetch::ProofBundle;
pub use bankai_types::verify::evm::beacon::BeaconHeader;

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

    /// Returns the execution layer network ID (always 1)
    pub const fn execution_network_id(&self) -> u64 {
        11155111
    }
}

// ============================================================================
// Public API Components
// ============================================================================

/// API client for interacting with Bankai's API
pub use crate::fetch::api::ApiClient;

/// Batch proof generation for efficient multi-proof operations
///
/// Combine multiple proof requests into a single optimized operation.
/// All proofs share the same STWO block proof and Bankai block number.
pub mod batch {

    pub use crate::fetch::batch::ProofBatchBuilder;
}

// Keep fetch module private (internal implementation details)
mod fetch;

use crate::errors::{SdkError, SdkResult};
use crate::fetch::ethereum::{beacon::BeaconChainFetcher, execution::ExecutionChainFetcher};

// ============================================================================
// Main SDK Struct
// ============================================================================

/// Namespace for Ethereum-related operations
struct EthereumNamespace {
    execution: Option<ExecutionChainFetcher>,
    beacon: Option<BeaconChainFetcher>,
}

/// Main entry point for the Bankai SDK
pub struct Bankai {
    /// Direct access to the Bankai API client
    pub api: ApiClient,
    /// Ethereum execution and beacon chain fetchers (internal)
    ethereum: EthereumNamespace,
    network: Network,
}

impl Bankai {
    /// Creates a new Bankai SDK instance
    ///
    /// # Arguments
    ///
    /// * `network` - The blockchain network (e.g., `Network::Sepolia`)
    /// * `ethereum_execution_rpc` - Optional execution layer RPC endpoint
    /// * `ethereum_beacon_rpc` - Optional beacon chain API endpoint
    pub fn new(
        network: Network,
        ethereum_execution_rpc: Option<String>,
        ethereum_beacon_rpc: Option<String>,
    ) -> Self {
        let api = ApiClient::new(network);
        let execution = ethereum_execution_rpc.map(|rpc| {
            ExecutionChainFetcher::new(api.clone(), rpc, network.execution_network_id())
        });
        let beacon = ethereum_beacon_rpc
            .map(|rpc| BeaconChainFetcher::new(api.clone(), rpc, network.beacon_network_id()));

        Bankai {
            api: api.clone(),
            ethereum: EthereumNamespace { execution, beacon },
            network,
        }
    }

    /// Returns the network this SDK instance is configured for
    pub fn network(&self) -> Network {
        self.network
    }

    /// Initialize a new batch proof builder
    ///
    /// # Arguments
    ///
    /// * `network` - The blockchain network (e.g., `Network::Sepolia`)
    /// * `bankai_block_number` - Optional Bankai block number (uses latest if `None`)
    /// * `hashing` - The hashing function for MMR proofs (Keccak, Poseidon, or Blake3)
    pub async fn init_batch(
        &self,
        network: Network,
        bankai_block_number: Option<u64>,
        hashing: HashingFunctionDto,
    ) -> SdkResult<batch::ProofBatchBuilder> {
        let block_number = match bankai_block_number {
            Some(bn) => bn,
            None => self.api.blocks().latest_number().await?,
        };
        Ok(batch::ProofBatchBuilder::new(
            self,
            network,
            block_number,
            hashing,
        ))
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
