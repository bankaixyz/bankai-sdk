# Bankai SDK (Rust)

Ergonomic access to Bankai fetchers and verifiers with a simple, TypeScript-friendly error model.

- Namespaced access: `bankai.evm.execution.header(...)`, `bankai.evm.beacon.header(...)`
- Simple verification: `bankai.verify.evm_execution_header(&proof)` and `bankai.verify.evm_beacon_header(&proof)`
- Robust errors via `SdkError` (no unwraps/asserts), aligned to the public API (`openapi.json`)

## Install

Add the crate to your workspace (or use the workspace member if you’re inside this repo):

```toml
[dependencies]
bankai-sdk = { path = "crates/sdk" }
bankai-types = { path = "crates/types" }
```

## Quickstart

Fetch an Ethereum execution header proof and verify it.

```rust
use bankai_sdk::{Bankai, errors::SdkError};
use bankai_types::api::proofs::HashingFunctionDto;

#[tokio::main]
async fn main() -> Result<(), SdkError> {
    // Build the SDK with configured endpoints
    let bankai = Bankai::builder()
        .with_api_base("https://sepolia.api.bankai.xyz".to_string())
        .with_evm_execution("https://sepolia.drpc.org".to_string())
        .build();

    // Fetch an execution header proof (Bankai block number ties MMR roots)
    let exec_fetcher = bankai
        .evm
        .execution
        .as_ref()
        .ok_or_else(|| SdkError::InvalidInput("execution not configured".into()))?;

    let proof = exec_fetcher
        .header(
            5_000_000,
            HashingFunctionDto::Keccak,
            42, // bankai block number to bind the MMR root
        )
        .await?;

    // Verify it
    let header = bankai.verify.evm_execution_header(&proof).await?;
    println!("verified execution header hash: 0x{}", header.inner.hash);

    Ok(())
}
```

Fetch a Beacon header proof and verify it.

```rust
use bankai_sdk::{Bankai, errors::SdkError};
use bankai_types::api::proofs::HashingFunctionDto;

#[tokio::main]
async fn main() -> Result<(), SdkError> {
    let bankai = Bankai::builder()
        .with_api_base("https://sepolia.api.bankai.xyz".to_string())
        .with_evm_beacon("https://lodestar-sepolia.beacon-api.nimbus.team".to_string())
        .build();

    let beacon_fetcher = bankai
        .evm
        .beacon
        .as_ref()
        .ok_or_else(|| SdkError::InvalidInput("beacon not configured".into()))?;

    let proof = beacon_fetcher
        .header(
            1_234_567,                // slot
            HashingFunctionDto::Poseidon,
            42,                        // bankai block number
        )
        .await?;

    let header = bankai.verify.evm_beacon_header(&proof).await?;
    println!("verified beacon header root bound via MMR");
    Ok(())
}
```

## API Overview

- Builder:
  - `Bankai::builder().with_api_base(..).with_evm_execution(rpc).with_evm_beacon(rpc).build()`
- Namespaces:
  - `bankai.evm.execution.header(block_number, hashing_function, bankai_block_number)`
  - `bankai.evm.beacon.header(slot, hashing_function, bankai_block_number)`
- Verification helpers:
  - `bankai.verify.evm_execution_header(&ExecutionHeaderProof)` → `ExecutionHeader`
  - `bankai.verify.evm_beacon_header(&BeaconHeaderProof)` → `BeaconHeader`

## Error Handling

All public functions return `SdkResult<T> = Result<T, SdkError>`.

```rust
use bankai_sdk::{Bankai, errors::SdkError};

async fn example() {
    let bankai = Bankai::builder().build();
    let result = async {
        let exec = bankai.evm.execution.as_ref().ok_or_else(|| SdkError::InvalidInput("missing exec".into()))?;
        exec.header(100, bankai_types::api::proofs::HashingFunctionDto::Keccak, 10).await
    }
    .await;

    match result {
        Ok(proof) => {
            println!("ok {} elements in path", proof.mmr_proof.path.len());
        }
        Err(SdkError::ApiErrorResponse { code, message, error_id }) => {
            eprintln!("api error: {code} ({error_id}) - {message}");
        }
        Err(SdkError::Api { status, body }) => {
            eprintln!("http {status}: {body}");
        }
        Err(e) => {
            eprintln!("error: {e}");
        }
    }
}
```

Notes:
- Errors from the Bankai API are parsed into `SdkError::ApiErrorResponse { code, message, error_id }` when the body matches the schema.
- Non-2xx responses that aren’t parseable map to `SdkError::Api { status, body }`.
- Provider/transport issues map to `SdkError::Provider`/`SdkError::Transport`.
- Verification failures return `SdkError::Verification`.

## Types you’ll use

- Hashing: `bankai_types::api::proofs::HashingFunctionDto::{Keccak, Poseidon}`
- Proofs:
  - `bankai_types::fetch::evm::execution::ExecutionHeaderProof`
  - `bankai_types::fetch::evm::beacon::BeaconHeaderProof`

## Future compatibility

- The `evm` namespace leaves room to add other chains later without breaking the surface.
- Errors are stable and stringly for smooth TS/WASM wrappers.
