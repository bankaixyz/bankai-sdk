# Supported Surfaces

This page lists the public surfaces available today.

It combines the SDK batch-builder methods, the low-level API namespaces, and the endpoint families covered by the compatibility harness.

## Chain Families

### Ethereum

Bankai exposes Ethereum light-client data across:

- beacon chain headers
- execution chain headers
- account and storage proofs
- transaction and receipt proofs

### OP Stack

Bankai also exposes OP Stack proof flows through chain names configured in your RPC map and recognized by the Bankai API.

In these docs, the main examples use:

- Base
- World Chain

More generally, the SDK works with names such as `"base"` or `"worldchain"` as long as the Bankai API and your configured RPCs both support them.

## SDK Batch Builder Methods

### Ethereum Methods

| Method | What it returns |
| --- | --- |
| `ethereum_execution_header` | A verified Ethereum execution header |
| `ethereum_beacon_header` | A verified Ethereum beacon header |
| `ethereum_account` | An account proof against an execution header |
| `ethereum_storage_slot` | One or more storage slot proofs |
| `ethereum_tx` | A transaction proof |
| `ethereum_receipt` | A receipt proof |

### OP Stack Methods

| Method | What it returns |
| --- | --- |
| `op_stack_header` | A specific OP header proof |
| `op_stack_account` | An OP account proof |
| `op_stack_storage_slot` | One or more OP storage slot proofs |
| `op_stack_tx` | An OP transaction proof |
| `op_stack_receipt` | An OP receipt proof |

## Low-Level API Namespaces

`ApiClient` exposes these namespaces:

| Namespace | What it is for |
| --- | --- |
| `blocks()` | Bankai block discovery and block-proof access |
| `chains()` | Active chain metadata |
| `health()` | API health checks |
| `stats()` | Overview and block-level stats |
| `ethereum()` | Ethereum root, beacon, and execution light-client endpoints |
| `op_stack()` | OP snapshot, merkle proof, MMR proof, and light-client endpoints |

## Bankai Block Selectors

Low-level API calls and proof requests support these selectors:

| Selector | Meaning |
| --- | --- |
| `latest` | The latest Bankai view available |
| `justified` | The latest justified Bankai view |
| `finalized` | The latest finalized Bankai view |
| explicit `bankai_block_number` | A specific Bankai block anchor |

Use `finalized` by default for trust-sensitive flows.

## Ethereum Endpoint Families

The compatibility harness covers these Ethereum endpoint families:

- `/v1/ethereum/epoch`
- `/v1/ethereum/epoch/{number}`
- `/v1/ethereum/sync_committee`
- `/v1/ethereum/beacon/height`
- `/v1/ethereum/beacon/snapshot`
- `/v1/ethereum/beacon/mmr_root`
- `/v1/ethereum/beacon/mmr_proof`
- `/v1/ethereum/beacon/light_client_proof`
- `/v1/ethereum/execution/height`
- `/v1/ethereum/execution/snapshot`
- `/v1/ethereum/execution/mmr_root`
- `/v1/ethereum/execution/mmr_proof`
- `/v1/ethereum/execution/light_client_proof`

## OP Stack Endpoint Families

The compatibility harness covers these OP endpoint families:

- `/v1/op/{name}/height`
- `/v1/op/{name}/snapshot`
- `/v1/op/{name}/merkle_proof`
- `/v1/op/{name}/mmr_proof`
- `/v1/op/{name}/light_client_proof`

## Example Coverage

| Example | Focus |
| --- | --- |
| [Basic Bundle Example](../example/basic-bundle/README.md) | Multi-chain proof bundle flow across Ethereum and OP Stack chains |
| [Basic API Example](../example/basic-api/README.md) | Raw API inspection and discovery |
| [World ID Root Example](../example/worldid-root/README.md) | Current Base plus Ethereum flow tied to the World ID story |
| [World ID Replicator Placeholder](../example/worldid-replicator/README.md) | Future zkVM replication design |

## Recommended Defaults

- Start with the batch builder unless you know you need raw API control.
- Start with `HashingFunction::Keccak`.
- Start with finalized Bankai views unless you have a reason to inspect `latest` or `justified`.
- Use the API chain list to discover active OP chains instead of assuming a fixed set forever.
