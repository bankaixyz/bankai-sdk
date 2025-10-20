//! Bankai SDK - Zero Knowledge Proof SDK
//!
//! # Overview
//!
//! Bankai enables trustless access to blockchain data through STWO zero-knowledge proofs.
//! The process involves three steps:
//!
//! 1. **Verify the block proof**: Validate the STWO proof to establish trust in the MMR roots
//! 2. **Retrieve MMR proofs**: Use MMR proofs to decommit and verify specific headers
//! 3. **Generate storage proofs**: Create Merkle proofs against the header's state root to access specific data
//!
//! # Setup
//!
//! ```no_run
//! use bankai_sdk::{Bankai, Network};
//!
//! let sdk = Bankai::new(
//!     Network::Sepolia,                              // Network to connect to
//!     Some("https://eth-sepolia.rpc".to_string()),   // Execution layer RPC (optional)
//!     Some("https://sepolia.beacon.api".to_string()) // Beacon chain RPC (optional)
//! );
//! ```
//!
//! **Required parameters:**
//! - `network`: Target blockchain network (e.g., `Network::Sepolia`)
//! - `evm_execution_rpc`: Optional - required for execution layer operations (headers, accounts, transactions)
//! - `evm_beacon_rpc`: Optional - required for beacon chain operations (consensus headers)
//!
//! # Batch Operations (Recommended)
//!
//! Batch multiple proof requests into a single optimized operation. All proofs share the same
//! STWO block proof and are anchored to the same Bankai block number.
//!
//! ```no_run
//! use bankai_sdk::{Bankai, Network, HashingFunctionDto};
//! use alloy_primitives::{Address, FixedBytes};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! # let sdk = Bankai::new(Network::Sepolia, 
//! #     Some("https://eth-sepolia.rpc".to_string()),
//! #     Some("https://sepolia.beacon.api".to_string()));
//!
//! let batch = sdk.init_batch(
//!     Network::Sepolia,
//!     None,  // Use latest block (or specify a Bankai block number)
//!     HashingFunctionDto::Keccak
//! ).await?;
//!
//! let tx_hash = FixedBytes::from([0u8; 32]);
//!
//! let result = batch
//!     .evm_beacon_header(8551383)                                    // Beacon header
//!     .evm_execution_header(9231247)                                 // Execution header
//!     .evm_tx(tx_hash)                                               // Transaction by hash
//!     .evm_account(9231247, Address::ZERO)                           // Account proof
//!     .execute()
//!     .await?;
//!
//! // Verify the batch proof using the verify crate
//! use bankai_verify::verify_batch_proof;
//! let verification_result = verify_batch_proof(&result.proof.proof)?;
//!
//! // Access individual proofs from the result
//! let beacon_proof = &result.beacon_headers[0];
//! let exec_proof = &result.execution_headers[0];
//! let tx_proof = &result.transactions[0];
//! let account_proof = &result.accounts[0];
//! # Ok(())
//! # }
//! ```
//!
//! # API Client
//!
//! Direct access to the Bankai API for low-level operations:
//!
//! ```no_run
//! use bankai_sdk::{Bankai, Network, HashingFunctionDto};
//! use bankai_types::api::proofs::{MmrProofRequestDto, LightClientProofRequestDto, HeaderRequestDto};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! # let sdk = Bankai::new(Network::Sepolia, None, None);
//!
//! // Get latest Bankai block number
//! let latest_block = sdk.api.get_latest_block_number().await?;
//!
//! // Fetch STWO block proof
//! let block_proof = sdk.api.get_block_proof(latest_block).await?;
//!
//! // Fetch MMR proof for a specific header
//! let mmr_request = MmrProofRequestDto {
//!     network_id: 1,  // 0 = beacon, 1 = execution
//!     block_number: 12345, // Bankai block number
//!     hashing_function: HashingFunctionDto::Keccak,
//!     header_hash: "0x...".to_string(),
//! };
//! let mmr_proof = sdk.api.get_mmr_proof(&mmr_request).await?;
//!
//! // Fetch batch light client proof (STWO proof + multiple MMR proofs)
//! let lc_request = LightClientProofRequestDto {
//!     bankai_block_number: Some(latest_block),
//!     hashing_function: HashingFunctionDto::Keccak,
//!     requested_headers: vec![
//!         HeaderRequestDto {
//!             network_id: 1,  // Execution layer
//!             header_hash: "0x...".to_string(),
//!         },
//!         HeaderRequestDto {
//!             network_id: 0,  // Beacon chain
//!             header_hash: "0x...".to_string(),
//!         },
//!     ],
//! };
//! let light_client_proof = sdk.api.get_light_client_proof(&lc_request).await?;
//! # Ok(())
//! # }
//! ```
//!
//! # EVM Fetchers
//!
//! Use individual fetchers to generate storage proofs and retrieve MMR proofs for specific data:
//!
//! ```no_run
//! use bankai_sdk::{Bankai, Network, HashingFunctionDto};
//! use alloy_primitives::{Address, FixedBytes};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! # let sdk = Bankai::new(Network::Sepolia,
//! #     Some("https://eth-sepolia.rpc".to_string()),
//! #     Some("https://sepolia.beacon.api".to_string()));
//!
//! // Execution layer fetcher
//! let execution = sdk.evm.execution()?;
//!
//! // Fetch execution header with MMR proof
//! let header = execution.header(9231247, HashingFunctionDto::Keccak, 12345).await?;
//!
//! // Fetch account with storage proofs
//! let account = execution.account(
//!     9231247,
//!     Address::ZERO,
//!     HashingFunctionDto::Keccak,
//!     12345
//! ).await?;
//!
//! // Fetch transaction with proof (by hash)
//! let tx_hash = FixedBytes::from([0u8; 32]);
//! let tx = execution.transaction(9231247, tx_hash, HashingFunctionDto::Keccak, 12345).await?;
//!
//! // Beacon chain fetcher
//! let beacon = sdk.evm.beacon()?;
//!
//! // Fetch beacon header with MMR proof
//! let beacon_header = beacon.header(8551383, HashingFunctionDto::Keccak, 12345).await?;
//! # Ok(())
//! # }
//! ```

pub mod errors;

// Re-export common types from bankai_types
pub use bankai_types::api::proofs::HashingFunctionDto;
pub use bankai_types::fetch::ProofWrapper;
pub use bankai_types::verify::evm::beacon::BeaconHeader;

// ============================================================================
// Network Configuration
// ============================================================================

/// Supported blockchain networks
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Network {
    /// Ethereum Sepolia testnet
    Sepolia,
}

impl Network {
    /// Returns the API base URL for this network
    pub fn api_url(&self) -> &'static str {
        match self {
            Network::Sepolia => "https://sepolia.api.bankai.xyz",
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

/// API client for interacting with Bankai's proof generation service
pub use crate::fetch::clients::bankai_api::ApiClient;

/// EVM execution and beacon chain fetchers
///
/// Access execution layer (headers, accounts, transactions) and beacon chain (consensus headers)
/// data with MMR proofs for trustless verification.
pub mod evm {

    pub use crate::fetch::evm::beacon::BeaconChainFetcher;
    pub use crate::fetch::evm::execution::ExecutionChainFetcher;

    // Re-export common EVM types
    pub use bankai_types::fetch::evm::{
        beacon::BeaconHeaderProof,
        execution::{AccountProof, ExecutionHeaderProof, TxProof},
    };
}

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
use crate::fetch::evm::{beacon::BeaconChainFetcher, execution::ExecutionChainFetcher};

// ============================================================================
// Main SDK Struct
// ============================================================================

/// Namespace for EVM-related operations
pub struct EvmNamespace {
    execution: Option<ExecutionChainFetcher>,
    beacon: Option<BeaconChainFetcher>,
}

/// Namespace for verification operations (placeholder for future functionality)
pub struct VerifyNamespace;

/// Main entry point for the Bankai SDK
pub struct Bankai {
    /// Direct access to the Bankai API client
    pub api: ApiClient,
    /// EVM execution and beacon chain fetchers
    pub evm: EvmNamespace,
    /// Verification utilities
    pub verify: VerifyNamespace,
    network: Network,
}

impl Bankai {
    /// Creates a new Bankai SDK instance
    ///
    /// # Arguments
    ///
    /// * `network` - The blockchain network (e.g., `Network::Sepolia`)
    /// * `evm_execution_rpc` - Optional execution layer RPC endpoint
    /// * `evm_beacon_rpc` - Optional beacon chain API endpoint
    pub fn new(
        network: Network,
        evm_execution_rpc: Option<String>,
        evm_beacon_rpc: Option<String>,
    ) -> Self {
        let api = ApiClient::new(network);
        let execution = evm_execution_rpc.map(|rpc| {
            ExecutionChainFetcher::new(api.clone(), rpc, network.execution_network_id())
        });
        let beacon = evm_beacon_rpc
            .map(|rpc| BeaconChainFetcher::new(api.clone(), rpc, network.beacon_network_id()));

        Bankai {
            api: api.clone(),
            evm: EvmNamespace { execution, beacon },
            verify: VerifyNamespace,
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
            None => self.api.get_latest_block_number().await?,
        };
        Ok(batch::ProofBatchBuilder::new(
            self,
            network,
            block_number,
            hashing,
        ))
    }
}

impl EvmNamespace {
    /// Get the execution layer fetcher
    pub fn execution(&self) -> SdkResult<&ExecutionChainFetcher> {
        self.execution
            .as_ref()
            .ok_or_else(|| SdkError::NotConfigured("EVM execution fetcher".to_string()))
    }

    /// Get the beacon chain fetcher
    pub fn beacon(&self) -> SdkResult<&BeaconChainFetcher> {
        self.beacon
            .as_ref()
            .ok_or_else(|| SdkError::NotConfigured("EVM beacon fetcher".to_string()))
    }
}
