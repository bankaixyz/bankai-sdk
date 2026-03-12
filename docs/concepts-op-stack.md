# OP Stack Concepts

For OP chains, Bankai stores a committed OP client inside the verified Bankai block.

That committed client contains the roots needed to verify OP headers, and from those headers you can verify OP accounts, storage, transactions, and receipts.

## Current Support Model

Current OP Stack support is narrow by design.

- Bankai currently uses proposer FDG submissions
- claimed roots are only accepted from the registered proposer for the OP chain
- this path still trusts the sequencer for the submitted OP state
- Bankai links that OP state to the corresponding L1 block and mirrors L1 finality

Today, this is an L1-linked finality model, not native OP finality.

## How OP Data Enters A Bankai Block

The Bankai block contains an `op_chains` commitment.

That commitment points to one or more committed OP chain clients, each of which includes:

- `chain_id`
- `block_number`
- `header_hash`
- `l1_submission_block`
- `mmr_root_keccak`
- `mmr_root_poseidon`

The verification path is:

1. verify the Bankai block
2. decommit the OP client from the OP chains root
3. read the OP client's MMR root
4. verify the OP header you want
5. verify account, storage, transaction, or receipt proofs under that header

## Why `l1_submission_block` Matters

`l1_submission_block` ties the committed OP client state back to the L1 block context used for finality today.

That is why the current OP story is "L1-linked finality," not "fully independent OP finality."

## What This Means For Users

When you use the SDK's OP methods, you still follow the same Bankai pattern:

- start from a Bankai block
- use committed roots to recover a trusted header
- use the trusted header to recover concrete chain data

The only extra step is decommitting the committed OP client in the middle.

## Base And World Chain In Practice

The SDK's OP flow is chain-name based, which is why examples use names such as:

- `"base"`
- `"worldchain"`

The chain name must match both:

- the key in your configured OP RPC map
- the chain name the Bankai API expects for that OP chain

## Future Direction

Native OP finality is planned work, not current behavior.

The trigger for that path is the end of the fraud period, which is about 3.5 days. That gives stronger native OP finality, but it is much slower than the current L1-linked model.

- today: L1-linked finality for supported OP flows
- later: native finality after the fraud period
