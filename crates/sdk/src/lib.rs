//! Bankai SDK - Zero Knowledge Proof SDK
//!
//! # Overview
//!
//! This SDK enables trustless verification of blockchain data through STWO zero-knowledge proofs.
//!
//! ## How It Works
//!
//! The Bankai system generates **STWO proofs** (block proofs) that contain **Merkle Mountain Ranges (MMRs)**
//! storing cryptographic commitments to valid blockchain headers. These proofs are the foundation of the system.
//!
//! To verify specific blockchain data:
//! 1. **Decommit a header**: Use MMR proofs to decommit and verify a specific header from the MMR
//! 2. **Verify chain data**: Once you have a verified header, you can verify any data from that block
//!    (accounts, transactions, storage, etc.) using standard Merkle proofs against the header's state root
//!
//! This two-step process (MMR decommitment → chain data verification) enables efficient, trustless
//! verification of any blockchain state without needing to sync the entire chain.
//!
//! # Examples
//!
//! ```no_run
//! use bankai_sdk::{Bankai, Network, HashingFunctionDto};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let sdk = Bankai::new(
//!         Network::Sepolia,
//!         Some("https://eth-sepolia.rpc".to_string()),
//!         Some("https://sepolia.beacon.api".to_string())
//!     );
//!     
//!     // Fetch execution header with MMR proof for decommitment
//!     let header_proof = sdk.evm.execution()?
//!         .header(12345, HashingFunctionDto::Keccak, 100).await?;
//!     
//!     // Or use batch operations for efficiency (network IDs are automatic)
//!     let batch = sdk.init_batch(None, HashingFunctionDto::Keccak)
//!         .await?
//!         .evm_execution_header(9231247)  // No network_id needed!
//!         .evm_beacon_header(8551383)     // No network_id needed!
//!         .execute()
//!         .await?;
//!     
//!     Ok(())
//! }
//! ```

pub mod errors;

// Re-export common types from bankai_types
pub use bankai_types::api::proofs::HashingFunctionDto;
pub use bankai_types::verify::evm::beacon::BeaconHeader;
pub use bankai_types::fetch::ProofWrapper;

// ============================================================================
// Network Configuration
// ============================================================================

/// Supported blockchain networks
///
/// Each network has associated configuration including:
/// - API endpoint URL for proof generation
/// - Beacon chain network ID (always 0)
/// - Execution layer network ID (always 1)
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
        1
    }
}

// ============================================================================
// Public API Components
// ============================================================================

/// API client for interacting with Bankai's proof generation service
///
/// Provides methods to:
/// - Fetch light client proofs
/// - Get block proofs
/// - Request MMR proofs
/// - Query latest block numbers
pub use crate::fetch::clients::bankai_api::ApiClient;

/// EVM-related functionality for fetching blockchain data with MMR proofs
pub mod evm {
    //! EVM chain data fetching with MMR proofs for header decommitment
    //!
    //! This module provides fetchers that retrieve blockchain headers along with MMR proofs.
    //! These MMR proofs enable decommitment of headers from the STWO block proofs, establishing
    //! trust in the header data without syncing the full chain.
    //!
    //! ## Available Fetchers
    //!
    //! - **Execution Layer** (`ExecutionChainFetcher`): Fetch execution headers, accounts, and transactions
    //!   with MMR proofs for decommitment from the STWO proof's execution MMR
    //! - **Beacon Chain** (`BeaconChainFetcher`): Fetch consensus layer headers with MMR proofs for
    //!   decommitment from the STWO proof's beacon MMR
    
    pub use crate::fetch::evm::beacon::BeaconChainFetcher;
    pub use crate::fetch::evm::execution::ExecutionChainFetcher;
    
    // Re-export common EVM types
    pub use bankai_types::fetch::evm::{
        beacon::BeaconHeaderProof,
        execution::{AccountProof, ExecutionHeaderProof, TxProof},
    };
}

/// Batch proof generation for efficient multi-proof operations
pub mod batch {
    //! Efficiently batch multiple proof requests into a single STWO proof
    //!
    //! The batch builder combines multiple proof requests (headers, accounts, transactions)
    //! into a single optimized request. All proofs share the same STWO block proof and
    //! are anchored to the same Bankai block number, making verification more efficient.
    //!
    //! Each proof in the batch uses MMR proofs to decommit headers from the STWO proof's MMRs,
    //! enabling verification of all requested data through a single block proof.
    //!
    //! Network IDs are automatically determined from the SDK's configured network:
    //! - Beacon chain: network_id = 0
    //! - Execution layer: network_id = 1
    //!
    //! # Example
    //!
    //! ```no_run
    //! # use bankai_sdk::{Bankai, Network, HashingFunctionDto};
    //! # async fn example(sdk: Bankai) -> Result<(), Box<dyn std::error::Error>> {
    //! // Use latest block automatically, network IDs are automatic
    //! let batch = sdk.init_batch(None, HashingFunctionDto::Keccak)
    //!     .await?
    //!     .evm_execution_header(9231247)
    //!     .evm_beacon_header(8551383)
    //!     .execute()
    //!     .await?;
    //! # Ok(())
    //! # }
    //! ```
    
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
///
/// Access execution and beacon chain fetchers through this namespace.
pub struct EvmNamespace {
    execution: Option<ExecutionChainFetcher>,
    beacon: Option<BeaconChainFetcher>,
}

/// Namespace for verification operations (placeholder for future functionality)
pub struct VerifyNamespace;

/// Main entry point for the Bankai SDK
///
/// The `Bankai` struct provides access to all SDK functionality:
/// - `api`: Direct access to the Bankai API client
/// - `evm`: EVM execution and beacon chain fetchers
/// - `verify`: Verification utilities (future)
///
/// # Example
///
/// ```no_run
/// use bankai_sdk::{Bankai, Network};
///
/// let sdk = Bankai::new(
///     Network::Sepolia,
///     Some("https://eth-sepolia.rpc".to_string()),
///     Some("https://sepolia.beacon.api".to_string())
/// );
/// ```
pub struct Bankai {
    /// Direct access to the Bankai API client for proof generation
    pub api: ApiClient,
    /// EVM execution and beacon chain data fetchers
    pub evm: EvmNamespace,
    /// Verification utilities
    pub verify: VerifyNamespace,
    /// The network this SDK instance is configured for
    network: Network,
}

impl Bankai {
    /// Creates a new Bankai SDK instance
    ///
    /// # Arguments
    ///
    /// * `network` - The blockchain network to connect to (e.g., `Network::Sepolia`)
    /// * `evm_execution_rpc` - Optional URL for EVM execution layer RPC endpoint
    /// * `evm_beacon_rpc` - Optional URL for EVM beacon chain API endpoint
    ///
    /// # Note
    ///
    /// If RPC endpoints are not provided, the corresponding functionality
    /// will not be available and will return `SdkError::NotConfigured` when accessed.
    ///
    /// The network determines:
    /// - API endpoint for proof generation
    /// - Network IDs (beacon=0, execution=1)
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
    /// Batching multiple proofs together is more efficient than requesting them individually.
    /// All proofs in the batch will be anchored to the same STWO block proof, sharing the same
    /// MMRs for header decommitment.
    ///
    /// # Arguments
    ///
    /// * `bankai_block_number` - Optional Bankai block number to anchor proofs to.
    ///   If `None`, automatically fetches and uses the latest block number from the API.
    /// * `hashing` - The hashing function to use for MMR proofs (Keccak, Poseidon, or Blake3)
    ///
    /// # Returns
    ///
    /// A `ProofBatchBuilder` that can be configured with multiple proof requests
    /// and executed to generate an optimized batch proof.
    ///
    /// # Errors
    ///
    /// Returns an error if `bankai_block_number` is `None` and fetching the latest block fails.
    pub async fn init_batch(
        &self,
        bankai_block_number: Option<u64>,
        hashing: HashingFunctionDto,
    ) -> SdkResult<batch::ProofBatchBuilder> {
        let block_number = match bankai_block_number {
            Some(bn) => bn,
            None => self.api.get_latest_block_number().await?,
        };
        Ok(batch::ProofBatchBuilder::new(self, block_number, hashing))
    }
}

impl EvmNamespace {
    /// Get the execution layer fetcher
    ///
    /// # Errors
    ///
    /// Returns `SdkError::NotConfigured` if no execution RPC was provided during initialization
    pub fn execution(&self) -> SdkResult<&ExecutionChainFetcher> {
        self.execution
            .as_ref()
            .ok_or_else(|| SdkError::NotConfigured("EVM execution fetcher".to_string()))
    }

    /// Get the beacon chain fetcher
    ///
    /// # Errors
    ///
    /// Returns `SdkError::NotConfigured` if no beacon RPC was provided during initialization
    pub fn beacon(&self) -> SdkResult<&BeaconChainFetcher> {
        self.beacon
            .as_ref()
            .ok_or_else(|| SdkError::NotConfigured("EVM beacon fetcher".to_string()))
    }
}