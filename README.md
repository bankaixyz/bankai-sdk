# Bankai SDK

**Trustless blockchain data access through zero-knowledge proofs**

Bankai SDK enables trustless access to blockchain data without running full nodes or maintaining state. Built on Bankai block proofs and stateless light client architecture, it uses zero-knowledge proofs and Merkle Mountain Ranges (MMRs) to provide cryptographic guarantees for any blockchain data.

## How It Works

The verification process follows a three-step process:

1. **Verify the Bankai block proof**: Validate the zero-knowledge proof to establish trust in the MMR roots
2. **Retrieve MMR proofs**: Use MMR proofs to decommit and verify specific headers from the MMR
3. **Generate storage proofs**: Create Merkle proofs against the header's state root to access specific data (accounts, transactions, storage)

This **stateless light client architecture** is fully trustless - no chains to sync, no state to maintain, no trusted intermediaries. Each proof bundle is self-contained and independently verifiable.

## Current Support

| Feature | Sepolia | Mainnet | Status |
|---------|---------|---------|--------|
| **Beacon Headers** | ✅ | ❌ | Available |
| **Execution Headers** | ✅ | ❌ | Available |
| **Execution Accounts** | ✅ | ❌ | Available |
| **Execution Storage Slots** | ✅ | ❌ | Available |
| **Execution Transactions** | ✅ | ❌ | Available |

**Note**: Mainnet support is coming soon. Currently only Sepolia testnet is supported.

### 📊 Bankai Dashboard

Monitor the status of Bankai networks and available blocks at the [Sepolia Dashboard](https://sepolia.dashboard.bankai.xyz/). The dashboard provides real-time information about:

- Available Bankai blocks and their numbers
- Network status and health metrics
- Latest MMR roots and proof availability
- System performance and uptime

---

## ⚠️ Important: Setup Requirements

**Before installing, you must patch the `ethereum_hashing` crate in your `Cargo.toml`:**

```toml
[dependencies]
bankai-sdk = "0.1"
bankai-verify = "0.1"
bankai-types = "0.1"

# Required dependency
ethereum_hashing = { git = "https://github.com/bankaixyz/ethereum_hashing", rev = "c457c3e927cc146d7bc91e944cf6d9c55b05d45e", default-features = false, features = ["portable"] }

[patch.crates-io]
ethereum_hashing = { git = "https://github.com/bankaixyz/ethereum_hashing", rev = "c457c3e927cc146d7bc91e944cf6d9c55b05d45e" }
```

**This patch is required for the SDK to work correctly.** We're working to remove this requirement in a future release.

---

## Installation

For local development within this repo:

```toml
[dependencies]
bankai-sdk = { path = "crates/sdk" }
bankai-verify = { path = "crates/verify" }
bankai-types = { path = "crates/types" }

# Required dependency (same as above)
ethereum_hashing = { git = "https://github.com/bankaixyz/ethereum_hashing", rev = "c457c3e927cc146d7bc91e944cf6d9c55b05d45e", default-features = false, features = ["portable"] }

[patch.crates-io]
ethereum_hashing = { git = "https://github.com/bankaixyz/ethereum_hashing", rev = "c457c3e927cc146d7bc91e944cf6d9c55b05d45e" }
```

---

## Getting Started

Here's a complete example showing how to fetch and verify blockchain data:

```rust
use bankai_sdk::{Bankai, Network, HashingFunctionDto};
use bankai_verify::verify_batch_proof;
use alloy_primitives::{Address, FixedBytes, U256};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Step 1: Initialize the SDK
    let bankai = Bankai::new(
        Network::Sepolia,
        Some("https://sepolia.infura.io/v3/YOUR_KEY".to_string()),  // Execution RPC
        Some("https://sepolia.beacon-api.example.com".to_string())  // Beacon RPC
    );

    // Step 2: Build a batch with multiple proof requests
    
    let proof_batch = bankai
        .init_batch(
            Network::Sepolia,
            None,  // Use latest Bankai block (or specify a block number)
            HashingFunctionDto::Keccak
        )
        .await?
        .ethereum_beacon_header(8_551_383)                               // Request beacon header
        .ethereum_execution_header(9_231_247)                            // Request execution header
        .ethereum_account(9_231_247, Address::ZERO)                      // Request account state
        .ethereum_storage_slot(9_231_247, Address::ZERO, vec![U256::from(0)]) // Request storage slot
        .ethereum_tx(FixedBytes::from([0u8; 32]))                        // Request transaction
        .execute()
        .await?;

    // Step 3: Verify the entire batch
    // This validates the block proof, MMR proofs, and all Merkle proofs
    let results = verify_batch_proof(proof_batch)?;
    
    // Step 4: Use the verified data - it's cryptographically guaranteed valid!
    for header in &results.evm.execution_header {
        println!("✓ Verified execution block {}: {:?}", header.number, header.hash());
    }
    
    for header in &results.evm.beacon_header {
        println!("✓ Verified beacon slot {}", header.slot);
    }
    
    for account in &results.evm.account {
        println!("✓ Verified account balance: {} wei", account.balance);
    }

    for slot in &results.evm.storage_slot {
        println!("✓ Verified storage slot: {:?}", slot);
    }
    
    for tx in &results.evm.tx {
        println!("✓ Verified transaction: {:?}", tx);
    }

    Ok(())
}
```

That's it! The data is now trustlessly verified and ready to use.

---

## The Three Crates

Bankai SDK is composed of three crates that work together:

### 📦 `bankai-sdk` - Data Fetching
Fetches blockchain data with cryptographic proofs from the Bankai API. Provides ergonomic batch builders and individual fetchers for headers, accounts, and transactions.

### ✅ `bankai-verify` - Trustless Verification
Cryptographically verifies all fetched data. Once verified, data is guaranteed to be valid - no further checks needed. Handles block proof verification, MMR proof verification, and Merkle proof verification.

### 🔧 `bankai-types` - Common Types
Shared types used across the SDK and verification crates. Works in both `std` and `no_std` environments.

---

## `bankai-sdk` - Data Fetching

The SDK provides three ways to fetch blockchain data:å

### 1. Batch Operations (Recommended)

Batch multiple proof requests into a single optimized operation:

```rust
use bankai_sdk::{Bankai, Network, HashingFunctionDto};
use alloy_primitives::{Address, FixedBytes};

let batch = sdk.init_batch(
    Network::Sepolia,
    None,  // Use latest block
    HashingFunctionDto::Keccak
).await?;

let tx_hash = FixedBytes::from([0u8; 32]);

let result = batch
    .ethereum_beacon_header(8551383)                  // Beacon header
    .ethereum_execution_header(9231247)               // Execution header
    .ethereum_tx(tx_hash)                             // Transaction by hash
    .ethereum_account(9231247, Address::ZERO)         // Account proof
    .execute()
    .await?;

// Verify with bankai-verify
use bankai_verify::verify_batch_proof;
let verification_result = verify_batch_proof(result)?;
```

### 2. API Client

Direct access to the Bankai API for low-level operations:

```rust
use bankai_sdk::{Bankai, Network, HashingFunctionDto};
use bankai_types::api::ethereum::{
    BankaiBlockFilterDto, EthereumLightClientProofRequestDto, EthereumMmrProofRequestDto,
};

// Get latest Bankai block number
let latest_block = sdk.api.blocks().latest_number().await?;

// Fetch Bankai block proof
let block_proof = sdk.api.blocks().proof(latest_block).await?;

// Fetch MMR proof for a specific header
let filter = BankaiBlockFilterDto::with_bankai_block_number(latest_block);
let mmr_request = EthereumMmrProofRequestDto {
    filter: filter.clone(),
    hashing_function: HashingFunctionDto::Keccak,
    header_hash: "0x...".to_string(),
};
let mmr_proof = sdk.api.ethereum().execution().mmr_proof(&mmr_request).await?;

// Fetch batch light client proof (block proof + multiple MMR proofs)
let lc_request = EthereumLightClientProofRequestDto {
    filter,
    hashing_function: HashingFunctionDto::Keccak,
    header_hashes: vec!["0x...".to_string()],
};
let light_client_proof = sdk.api.ethereum().execution().light_client_proof(&lc_request).await?;
```

### Configuration

```rust
let sdk = Bankai::new(
    Network::Sepolia,                         // Network to connect to
    Some("https://eth-sepolia.rpc".to_string()),  // Execution RPC (optional)
    Some("https://sepolia.beacon.api".to_string()) // Beacon RPC (optional)
);
```

**Network IDs:**
- Beacon chain: Always `0`
- Execution layer: Always `11155111`

**Supported Networks:**
- `Network::Sepolia` - Ethereum Sepolia testnet

---

## `bankai-verify` - Trustless Verification

The verification library provides cryptographic guarantees for all fetched data.

### Batch Verification (Recommended)

Verify complete proof bundles in one call:

```rust
use bankai_verify::verify_batch_proof;

// Verify an entire batch of proofs at once
let results = verify_batch_proof(proof_bundle)?;

// Access verified data - no further checks needed
for header in &results.evm.execution_header {
    println!("Verified execution header at block {}", header.number);
}

for account in &results.evm.account {
    println!("Verified account with balance: {}", account.balance);
}
```

### Verify Block Proof Only

Verify just the block proof to get trusted MMR roots:

```rust
use bankai_verify::bankai::stwo::verify_stwo_proof;
use cairo_air::CairoProof;
use stwo::core::vcs::blake2_merkle::Blake2sMerkleHasher;

// Verify the block proof and extract the Bankai block
let bankai_block = verify_stwo_proof(&block_proof)?;

// Access the verified MMR roots
println!("Execution MMR root (Keccak): {:?}", bankai_block.execution.mmr_root_keccak);
println!("Beacon MMR root (Keccak): {:?}", bankai_block.beacon.mmr_root_keccak);
```

### Verify MMR Proofs

Verify that a header is committed in the MMR:

```rust
use bankai_verify::bankai::mmr::MmrVerifier;
use bankai_types::fetch::evm::MmrProof;

// Verify that a header is committed in the MMR
let is_valid = MmrVerifier::verify_mmr_proof(&mmr_proof)?;
```

### Verify Header Proofs

Verify individual headers against a trusted MMR root:

```rust
use bankai_verify::evm::{ExecutionVerifier, BeaconVerifier};
use bankai_types::fetch::evm::execution::{ExecutionHeaderProof, AccountProof, TxProof};
use bankai_types::verify::evm::execution::ExecutionHeader;
use alloy_primitives::FixedBytes;

// Verify an execution header
let verified_header = ExecutionVerifier::verify_header_proof(&proof, mmr_root)?;

// Verify accounts and transactions against the verified header
let account = ExecutionVerifier::verify_account_proof(&account_proof, &[verified_header.clone()])?;
let transaction = ExecutionVerifier::verify_tx_proof(&tx_proof, &[verified_header])?;
```

### How Verification Works

The verification follows a hierarchical trust chain:

1. **Block Proof Verification**: Validates the zero-knowledge proof to establish trust in MMR roots
2. **MMR Proof Verification**: Verifies headers are committed in the MMR using the trusted roots
3. **Storage Proof Verification**: Verifies accounts/transactions against the header's state/transaction roots

**Once verified, data is cryptographically guaranteed to be valid.** No further checks are needed.

---

## `bankai-types` - Common Types

Shared type definitions used across the SDK and verification library.

### Core Modules

- **`proofs`** - MMR proofs, hashing functions (works in `no_std`)
- **`api`** - API request/response types (requires `std` and `api` feature)
- **`fetch`** - Proof fetching types (requires `verifier-types` feature)
- **`verify`** - Verification result types (requires `verifier-types` feature)
- **`block`** - Bankai block representations
- **`utils`** - MMR utility functions

### Feature Flags

- `std` (default) - Standard library support
- `api` - Enable API types
- `verifier-types` - Enable verifier-specific types

---

## Error Handling

### SDK Errors

```rust
pub enum SdkError {
    ApiErrorResponse { code: String, message: String, error_id: String },
    Api { status: StatusCode, body: String },
    NotConfigured(String),
    InvalidInput(String),
    NotFound(String),
    Reqwest(reqwest::Error),
    SerdeJson(serde_json::Error),
}
```

### Verification Errors

```rust
pub enum VerifyError {
    InvalidStwoProof,        // ZK proof verification failed
    InvalidMmrProof,         // MMR inclusion proof failed
    InvalidHeaderHash,       // Header hash mismatch
    InvalidAccountProof,     // Account MPT proof failed
    InvalidTxProof,          // Transaction MPT proof failed
    // ... and more
}
```
