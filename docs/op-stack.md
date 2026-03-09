# OP Stack Integration

OP Stack requests use the same core flow as Ethereum:

1. configure chain RPCs keyed by name
2. add OP Stack requests to the batch
3. execute the batch
4. verify the returned bundle

## Configure Chain RPCs

The OP Stack configuration is a `BTreeMap<String, String>` where the key is the chain name used by the Bankai API.

```rust
use std::collections::BTreeMap;

use bankai_sdk::{Bankai, Network};

let mut op_rpcs = BTreeMap::new();
op_rpcs.insert("base".to_string(), "https://mainnet.base.org".to_string());

let bankai = Bankai::new(Network::Sepolia, None, None, Some(op_rpcs));
```

The chain name passed to batch methods must match this key exactly.

## End-to-End Example

```rust
use std::collections::BTreeMap;

use alloy_primitives::Address;
use bankai_sdk::{Bankai, HashingFunction, Network};
use bankai_verify::verify_batch_proof;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut op_rpcs = BTreeMap::new();
    op_rpcs.insert("base".to_string(), "https://mainnet.base.org".to_string());

    let bankai = Bankai::new(Network::Sepolia, None, None, Some(op_rpcs));

    let proof_bundle = bankai
        .init_batch(Network::Sepolia, None, HashingFunction::Keccak)
        .await?
        .op_stack_account(
            "base",
            38_381_200,
            "0xcF93D9de9965B960769aa9B28164D571cBbCE39C".parse::<Address>()?,
        )
        .execute()
        .await?;

    let results = verify_batch_proof(proof_bundle)?;

    let header = &results.op_stack.header[0];
    let account = &results.op_stack.account[0];

    println!("Verified OP Stack block {}", header.number);
    println!("Verified OP Stack account balance {}", account.balance);

    Ok(())
}
```

## What the Batch Builder Supports

The OP Stack methods mirror the Ethereum-side builder surface:

- `op_stack_header`
- `op_stack_latest_header`
- `op_stack_header_by_hash`
- `op_stack_account`
- `op_stack_storage_slot`
- `op_stack_tx`
- `op_stack_receipt`

The builder automatically fetches the OP snapshot and any required header proofs needed for verification.

## Hosted API vs Local API

- Use `Network::Sepolia` when you want the default hosted Bankai API.
- Use `Network::Local` when you are working against a fully local Bankai deployment.
- Use `Bankai::new_with_base_url(Network::Sepolia, ...)` when the Bankai API is local but the Ethereum-side data should still behave like Sepolia.

The example under [`example/worldid-root`](../example/worldid-root/README.md) shows the local API variant.

## Low-Level API

If you need the raw OP endpoints directly, use `sdk.api.op_stack()`.

See [API client overview](api-client.md).
