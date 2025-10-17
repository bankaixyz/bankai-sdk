# Bankai SDK

**Trustless blockchain data access through zero-knowledge proofs**

Bankai SDK provides a stateless light client architecture that enables trustless verification of blockchain data without syncing full nodes or maintaining any state. Using STWO zero-knowledge proofs and Merkle Mountain Ranges (MMRs), you can fetch and verify any blockchain data with cryptographic guarantees.

## Overview

The Bankai SDK consists of three main crates:

### 📦 `bankai-sdk` - Data Fetching
Ergonomic APIs for fetching blockchain data with cryptographic proofs:
- **Batch Builder**: Compose multiple proof requests (headers, accounts, transactions) in a single efficient batch
- **EVM Support**: Fetch execution layer headers, beacon chain headers, account states, and transactions
- **MMR Proofs**: Automatically generates Merkle Mountain Range proofs for header verification
- **Network Configuration**: Built-in network support (Sepolia, with Mainnet coming soon)

### ✅ `bankai-verify` - Trustless Verification  
Cryptographically verify all fetched data using STWO zero-knowledge proofs:
- **Guaranteed Validity**: Once verified, data is cryptographically guaranteed to be correct—no further checks needed
- **Stateless Architecture**: No state to maintain, no chains to sync, no trusted intermediaries
- **Batch Verification**: Verify entire proof bundles in one call
- **Hierarchical Trust**: STWO proof → MMR proofs → Merkle proofs for complete verification chain

### 🔧 `bankai-types` - Common Types
Shared types used across the SDK and verification crates

## Key Features

- ⚡ **Stateless**: No need to sync chains or maintain state
- 🔒 **Trustless**: Cryptographic proofs guarantee data validity
- 🎯 **Efficient**: Batch multiple proofs into a single request
- 🌐 **Cross-Chain Ready**: Verify data from any supported blockchain
- 🛠️ **Developer Friendly**: Ergonomic Rust APIs with comprehensive error handling

## Installation

Add the SDK to your `Cargo.toml`:

```toml
[dependencies]
bankai-sdk = "0.1"
bankai-verify = "0.1"
bankai-types = "0.1"
```

For local development within this repo:
```toml
[dependencies]
bankai-sdk = { path = "crates/sdk" }
bankai-verify = { path = "crates/verify" }
bankai-types = { path = "crates/types" }
```

## Quick Start

### Complete Example: Fetch and Verify Blockchain Data

```rust
use bankai_sdk::{Bankai, Network, HashingFunctionDto};
use bankai_verify::verify_batch_proof;
use alloy_primitives::Address;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize SDK with network and RPC endpoints
    let bankai = Bankai::new(
        Network::Sepolia,
        Some("https://sepolia.infura.io/v3/YOUR_KEY".to_string()),
        Some("https://sepolia.beacon-api.example.com".to_string())
    );

    // Build a proof batch with multiple requests
    // Network IDs are automatic: beacon=0, execution=1
    let proof_batch = bankai
        .init_batch(Network::Sepolia, None, HashingFunctionDto::Keccak)  // None = use latest block
        .await?
        .evm_beacon_header(8_551_383)           // Beacon slot
        .evm_execution_header(9_231_247)        // Execution block  
        .evm_account(9_231_247, Address::ZERO)  // Account at block
        .execute()
        .await?;

    // Verify the entire batch - all returned data is cryptographically guaranteed valid!
    let results = verify_batch_proof(&proof_batch)?;
    
    // Use the verified data - no further validation needed
    for header in &results.evm.execution_header {
        println!("✓ Verified execution block {}: {:?}", header.number, header.hash());
    }
    
    for header in &results.evm.beacon_header {
        println!("✓ Verified beacon slot {}", header.slot);
    }
    
    for account in &results.evm.account {
        println!("✓ Verified account balance: {} wei", account.balance);
    }

    Ok(())
}
```

## Usage Examples

### Verify a Single Execution Header

```rust
use bankai_sdk::{Bankai, Network, HashingFunctionDto};
use bankai_verify::verify_batch_proof;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let bankai = Bankai::new(
        Network::Sepolia,
        Some(std::env::var("EXECUTION_RPC")?),
        None  // Beacon RPC not needed for execution-only
    );

    let proof_batch = bankai
        .init_batch(Network::Sepolia, None, HashingFunctionDto::Keccak)
        .await?
        .evm_execution_header(9_231_247)
        .execute()
        .await?;

    let results = verify_batch_proof(&proof_batch)?;
    let header = &results.evm.execution_header[0];
    
    // Header is cryptographically guaranteed valid!
    println!("✓ Block {}: state_root = {:?}", header.number, header.state_root);
    Ok(())
}
```

### Verify Multiple Transactions

```rust
use bankai_sdk::{Bankai, Network, HashingFunctionDto};
use bankai_verify::verify_batch_proof;
use alloy_primitives::{hex::FromHex, FixedBytes};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let bankai = Bankai::new(
        Network::Sepolia,
        Some(std::env::var("EXECUTION_RPC")?),
        None
    );

    let proof_batch = bankai
        .init_batch(Network::Sepolia, None, HashingFunctionDto::Keccak)
        .await?
        .evm_tx(FixedBytes::from_hex(
            "0x501b7c72c1e5f14f02e1a58a7264e18f5e26a793d42e4e802544e6629764f58c"
        )?)
        .evm_tx(FixedBytes::from_hex(
            "0xd7e25cbf8ff63e3d9e4fa1e9783afae248a50df836f2cd853f89440f4c76891d"
        )?)
        .execute()
        .await?;

    let results = verify_batch_proof(&proof_batch)?;
    
    // All transactions are cryptographically verified!
    for (i, tx) in results.evm.tx.iter().enumerate() {
        println!("✓ Transaction {}: verified and included in block", i + 1);
    }
    
    Ok(())
}
```

### Fetch Data Without Proofs (Quick RPC Access)

For cases where you need quick data access without verification:

```rust
use bankai_sdk::{Bankai, Network};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let bankai = Bankai::new(
        Network::Sepolia,
        Some(std::env::var("EXECUTION_RPC")?),
        Some(std::env::var("BEACON_RPC")?)
    );

    // Fetch execution header directly (no proof)
    let exec = bankai.evm.execution()?;
    let header = exec.header_only(9_231_247).await?;
    println!("Fetched execution header {}", header.number);

    // Fetch beacon header directly (no proof)
    let beacon = bankai.evm.beacon()?;
    let beacon_header = beacon.header_only(8_551_383).await?;
    println!("Fetched beacon header at slot {}", beacon_header.slot);
    
    Ok(())
}
```

## API Client

The Bankai API client provides direct access to the proof generation service. You can use it to fetch different types of proofs at various levels of granularity.

### Accessing the API Client

```rust
use bankai_sdk::{Bankai, Network, ApiClient};

// Through the SDK
let sdk = Bankai::new(Network::Sepolia, None, None);
let latest_block = sdk.api.get_latest_block_number().await?;

// Or directly
let api = ApiClient::new(Network::Sepolia);
let block_proof = api.get_block_proof(12345).await?;
```

### Available Operations

#### 1. Get Latest Block Number

```rust
// Fetch the latest completed Bankai block number
let latest = api.get_latest_block_number().await?;
println!("Latest block: {}", latest);
```

#### 2. Get Block Proof (STWO Proof)

```rust
// Fetch just the STWO zero-knowledge proof
let block_proof = api.get_block_proof(12345).await?;

// This proof contains MMR roots for both beacon and execution chains
// Verify it using bankai-verify:
// let bankai_block = verify_stwo_proof(&block_proof.proof)?;
```

#### 3. Get MMR Proof

```rust
use bankai_types::proofs::{MmrProofRequestDto, HashingFunctionDto};

// Fetch an MMR proof for a specific header
let request = MmrProofRequestDto {
    network_id: 1,  // 1 = execution, 0 = beacon
    block_number: 9231247,
    hashing_function: HashingFunctionDto::Keccak,
    header_hash: "0x...".to_string(),
};

let mmr_proof = api.get_mmr_proof(&request).await?;

// Verify the MMR proof against a trusted MMR root:
// let verified_hash = verify_mmr_proof(&mmr_proof, mmr_root)?;
```

#### 4. Get Light Client Proof (Batch)

```rust
use bankai_types::proofs::{LightClientProofRequestDto, HeaderRequestDto, HashingFunctionDto};

// Fetch multiple MMR proofs + STWO proof in one request
let request = LightClientProofRequestDto {
    bankai_block_number: Some(12345),
    hashing_function: HashingFunctionDto::Keccak,
    requested_headers: vec![
        HeaderRequestDto {
            network_id: 1,
            header_hash: "0x...".to_string(),
        },
        HeaderRequestDto {
            network_id: 0,
            header_hash: "0x...".to_string(),
        },
    ],
};

let light_client_proof = api.get_light_client_proof(&request).await?;
// Contains: block_proof + mmr_proofs for all requested headers
```

### When to Use Each API

| Use Case | API Method | Best For |
|----------|------------|----------|
| Get MMR roots only | `get_block_proof()` | When you already have headers and just need to verify MMRs |
| Verify single header | `get_mmr_proof()` | When you need one specific header verified |
| Verify multiple headers | `get_light_client_proof()` | When you need multiple headers (more efficient than individual calls) |
| Complex workflows | Batch Builder (`init_batch()`) | When you need headers + accounts + transactions in one proof |
| Check latest data | `get_latest_block_number()` | When you want to anchor to the most recent block |

## API Reference

### Core Types

```rust
// Initialize SDK
let bankai = Bankai::new(
    network: Network,           // Network::Sepolia (Mainnet coming soon)
    evm_execution_rpc: Option<String>,
    evm_beacon_rpc: Option<String>
);

// Build proof batch
let batch = bankai
    .init_batch(
        network: Network,
        bankai_block_number: Option<u64>,  // None = use latest
        hashing: HashingFunctionDto         // Keccak, Poseidon, or Blake3
    )
    .await?
    .evm_execution_header(block_number: u64)
    .evm_beacon_header(slot: u64)
    .evm_account(block_number: u64, address: Address)
    .evm_tx(tx_hash: FixedBytes<32>)
    .execute()
    .await?;

// Verify batch
let results = verify_batch_proof(&batch)?;
```

### Network Configuration

Network IDs are automatically handled:
- **Beacon chain**: Always network ID `0`
- **Execution layer**: Always network ID `1`

Supported networks:
- `Network::Sepolia` - Ethereum Sepolia testnet
- `Network::Mainnet` - Coming soon

### Error Handling

All SDK operations return `Result<T, SdkError>` with comprehensive error types:

```rust
pub enum SdkError {
    // API Errors
    ApiErrorResponse { code: String, message: String, error_id: String },
    Api { status: StatusCode, body: String },
    
    // Configuration Errors
    NotConfigured(String),   // Required component not initialized
    InvalidInput(String),    // Invalid parameter provided
    NotFound(String),        // Requested resource not found
    
    // Network & Parsing Errors
    Reqwest(reqwest::Error),
    SerdeJson(serde_json::Error),
}

// Verification errors from bankai-verify
pub enum VerifyError {
    InvalidStwoProof,        // ZK proof verification failed
    InvalidMmrProof,         // MMR inclusion proof failed
    InvalidHeaderHash,       // Header hash mismatch
    InvalidAccountProof,     // Account MPT proof failed
    InvalidTxProof,          // Transaction MPT proof failed
    // ... and more
}
```

## Use Cases

### Cross-Chain Bridges
Verify state from other chains without running full nodes:
```rust
// Verify account balance on source chain before bridging
let proof = sdk.init_batch(Network::Sepolia, None, HashingFunctionDto::Keccak)
    .await?
    .evm_account(block_number, user_address)
    .execute()
    .await?;
let results = verify_batch_proof(&proof)?;
let balance = results.evm.account[0].balance;
// Balance is cryptographically guaranteed - safe to bridge!
```

### On-Chain Data Access
Smart contracts can verify external data trustlessly:
```rust
// Generate proof off-chain
let proof = sdk.init_batch(Network::Sepolia, None, HashingFunctionDto::Keccak)
    .await?
    .evm_execution_header(target_block)
    .execute()
    .await?;

// Submit proof to smart contract for verification
// Contract can trustlessly access the verified data
```ankai

### ZK Circuit Inputs
Provide verified inputs to zero-knowledge circuits:
```rust
// Fetch and verify historical transaction data
let proof = sdk.init_batch(Network::Sepolia, None, HashingFunctionDto::Keccak)
    .await?
    .evm_tx(tx_hash)
    .execute()
    .await?;

let results = verify_batch_proof(&proof)?;
// Use verified transaction as trusted input to ZK circuit
```

### Historical Data Queries
Access any historical blockchain data without syncing:
```rust
// Query account state at specific block without full node
let proof = sdk.init_batch(Network::Sepolia, None, HashingFunctionDto::Keccak)
    .await?
    .evm_account(historical_block, account_address)
    .execute()
    .await?;

let results = verify_batch_proof(&proof)?;
// Access historical balance, nonce, code_hash - all verified
```

## Development

### Build

```bash
cargo build
```

### Run Examples

```bash
# Set up environment
export EXECUTION_RPC="https://sepolia.infura.io/v3/YOUR_KEY"
export BEACON_RPC="https://sepolia.beacon-api.example.com"

# Run the example
cargo run --package bankai-sdk
```

### Run Tests

```bash
cargo test
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

[Add your license information here]
