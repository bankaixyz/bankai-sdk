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
//!         .evm_beacon_header(8_551_383)
//!         .evm_execution_header(9_231_247)
//!         .evm_account(9_231_247, Address::ZERO)
//!         .evm_storage_slot(9_231_247, Address::ZERO, U256::from(0))
//!         .evm_tx(FixedBytes::from([0u8; 32]))
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
//!         println!("✓ Verified storage slot: {}", slot);
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
        execution::{AccountProof, ExecutionHeaderProof, StorageSlotProof, TxProof},
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
