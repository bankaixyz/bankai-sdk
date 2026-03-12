# Verify Crate Guide

`bankai-sdk` fetches proof data. `bankai-verify` verifies that proof data and returns the verified headers and EVM objects.

## Main Entry Point

```rust
use bankai_types::inputs::ProofBundle;
use bankai_verify::verify_batch_proof;

fn verify(bundle: ProofBundle) -> Result<(), Box<dyn std::error::Error>> {
    let results = verify_batch_proof(bundle)?;
    println!("Verified {} execution headers", results.evm.execution_header.len());
    Ok(())
}
```

## Verification Order

The verifier follows a staged flow.

### 1. Verify the Bankai block proof

This gives you a verified Bankai block.

### 2. Select the relevant MMR root

From the trusted Bankai block, you can:

- read Ethereum execution or beacon MMR roots directly
- or decommit OP chain client output from the committed OP chains root

### 3. Verify the MMR proof

This gives you a verified header.

### 4. Verify the MPT proof

With the trusted header in hand, you can verify:

- accounts
- storage slots
- transactions
- receipts

## What Success Means

If verification succeeds, each returned object has been checked against the verified Bankai block and the roots derived from it.

You can then use those results:

- inside an application
- in an off-chain proving pipeline
- inside a zkVM
- anywhere else you want a self-contained trust boundary

## Why This Matters For zkVMs

Because the proof bundle carries the full verification path, the same logic can run inside a proving system.

That is why Bankai fits zk workflows:

- fetch a bundle once
- verify it deterministically
- carry the verified result into a new proof system

The verifier is not just an application-side check. It is the step that turns Bankai proof data into an object another proof system can rely on.

## Result Shape

`verify_batch_proof(...)` returns grouped results:

- `results.evm.*` for Ethereum data
- `results.op_stack.*` for OP Stack data

You only need to read the groups that match the requests you made in the batch.

## Next Reads

- [Proof Bundles](proof-bundles.md)
- [Bankai Blocks](concepts-bankai-blocks.md)
- [World ID Replicator Placeholder](../example/worldid-replicator/README.md)
