## Bankai SDK (Rust)

Ergonomic access to Bankai fetchers and verifiers.

- **Batch builder**: compose multiple proof requests (execution headers, beacon headers, accounts) and fetch in one go.
- **One-call verification**: verify everything in a wrapper with `verify::batch::verify_wrapper`.
- **Consistent errors**: all fallible APIs return `SdkResult<T>` with `SdkError`.

### Install

Add these to your `Cargo.toml` (path deps shown for working inside this repo):

```toml
[dependencies]
bankai-sdk = { path = "crates/sdk" }
bankai-types = { path = "crates/types" }
```

### Quickstart: build a batch and verify

```rust
use bankai_sdk::{Bankai, errors::SdkError};
use bankai_sdk::verify::batch::verify_wrapper;
use bankai_types::api::proofs::HashingFunctionDto;
use alloy_primitives::Address;

#[tokio::main]
async fn main() -> Result<(), SdkError> {
    // Configure RPCs via env (optional): EXECUTION_RPC, BEACON_RPC
    let exec_rpc = std::env::var("EXECUTION_RPC").ok();
    let beacon_rpc = std::env::var("BEACON_RPC").ok();
    let bankai = Bankai::new(exec_rpc, beacon_rpc);

    // Example inputs
    let bankai_block_number = 11260u64;     // binds which MMR roots to use
    let exec_block_number = 9_231_247u64;   // execution L2/L1 block number (network 1 in this repo)
    let beacon_slot = 8_551_383u64;         // beacon slot (network 0 in this repo)

    // Build a batch: beacon header, execution header, and account proof
    let wrapper = bankai
        .init_batch(bankai_block_number, HashingFunctionDto::Keccak)
        .evm_beacon_header(0, beacon_slot)            // beacon network id 0
        .evm_execution_header(1, exec_block_number)   // execution network id 1
        .evm_account(1, exec_block_number, Address::ZERO)
        .execute()
        .await?;

    // Verify all proofs against the Bankai block commitments
    let results = verify_wrapper(&wrapper).await?;
    println!("verified execution headers: {}", results.evm.execution_header.len());
    println!("verified beacon headers: {}", results.evm.beacon_header.len());
    println!("verified accounts: {}", results.evm.account.len());

    Ok(())
}
```

### Single-proof batch: verify one execution header

```rust
use bankai_sdk::{Bankai, errors::SdkError};
use bankai_sdk::verify::batch::verify_wrapper;
use bankai_types::api::proofs::HashingFunctionDto;

#[tokio::main]
async fn main() -> Result<(), SdkError> {
    let bankai = Bankai::new(std::env::var("EXECUTION_RPC").ok(), std::env::var("BEACON_RPC").ok());

    let wrapper = bankai
        .init_batch(11260, HashingFunctionDto::Keccak)
        .evm_execution_header(1, 9_231_247)
        .execute()
        .await?;

    let results = verify_wrapper(&wrapper).await?;
    let header = &results.evm.execution_header[0];
    println!("verified one execution header");
    Ok(())
}
```

### Fetch via RPC only (no proofs)

```rust
use bankai_sdk::{Bankai, errors::SdkError};

#[tokio::main]
async fn main() -> Result<(), SdkError> {
    let bankai = Bankai::new(std::env::var("EXECUTION_RPC").ok(), std::env::var("BEACON_RPC").ok());

    let exec = bankai.evm.execution()?;
    let header = exec.header_only(9_231_247).await?; // alloy_rpc_types::Header
    println!("fetched execution header {}", header.number);

    let beacon = bankai.evm.beacon()?;
    let bheader = beacon.header_only(8_551_383).await?; // bankai_types::verify::evm::beacon::BeaconHeader
    println!("fetched beacon header");
    Ok(())
}
```

### API overview

- **Construct**: `let bankai = Bankai::new(exec_rpc: Option<String>, beacon_rpc: Option<String>);`
- **Batch builder**: `bankai.init_batch(bankai_block_number, hashing)` → `.evm_execution_header(..)`, `.evm_beacon_header(..)`, `.evm_account(..)` → `.execute()` → `ProofWrapper`
- **Verify**: `verify::batch::verify_wrapper(&ProofWrapper)` → `BatchResults`
- **Direct fetch**: `bankai.evm.execution()?.header_only(..)`, `bankai.evm.beacon()?.header_only(..)`

Notes:
- Hashing can be `HashingFunctionDto::Keccak` or `HashingFunctionDto::Poseidon`.
- In this repo, execution network id is `1` and beacon network id is `0` (see how `Bankai::new` wires fetchers). Adjust as needed in your integration.

### Errors

All fallible APIs return `SdkResult<T> = Result<T, SdkError>`. Common cases:

- **ApiErrorResponse { code, message, error_id }**: well-formed API error JSON
- **Api { status, body }**: non-2xx without a parseable error body
- **NotConfigured / InvalidInput / NotFound**: client-side usage errors
- **Verification**: proof verification failed

### Build

```bash
cargo build
```

Optionally configure environment:

```bash
export EXECUTION_RPC=...   # e.g. https://sepolia.example.org
export BEACON_RPC=...      # e.g. https://beacon.example.org
```
