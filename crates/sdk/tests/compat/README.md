# SDK/API Compatibility Test Harness

This directory contains the live compatibility harness used by `crates/sdk/tests/compat_live.rs`.

The harness validates that:

1. SDK request/response decoding still matches backend API behavior.
2. Proof verification flows still work across decode + verify surfaces.
3. OpenAPI endpoint coverage stays complete for `/v1/*` business routes.
4. Matrix variants (filters/formats/selectors/hash modes) keep working as the API evolves.

## Where It Runs

Test entrypoint:

- `crates/sdk/tests/compat_live.rs`

Main modules:

- `case.rs`: case model (`CompatCaseDef`, endpoint metadata, required/optional, matrix scope).
- `api/*`: case registry per domain (health/chains/blocks/stats/ethereum).
- `openapi_minimal.rs`: OpenAPI parity check.
- `context.rs`: reusable matrix inputs + request builders.
- `assertions.rs`: logical invariants shared across suites.
- `runner/*`: suite execution, decode logic, verify logic, debug repro curl output.

## What Is Covered

### 1) Endpoint decode coverage

All SDK-facing endpoints are exercised in decode mode:

- Health: `/v1/health`
- Chains: `/v1/chains`, `/v1/chains/{chain_id}`
- Blocks:
  - `/v1/blocks`
  - `/v1/blocks/latest`
  - `/v1/blocks/{height}`
  - `/v1/blocks/get_proof`
  - `/v1/blocks/{height}/proof`
  - `/v1/blocks/mmr_proof`
  - `/v1/blocks/block_proof`
- Stats: `/v1/stats/overview`, `/v1/stats/block/{height}`
- Ethereum root:
  - `/v1/ethereum/epoch`
  - `/v1/ethereum/epoch/{number}`
  - `/v1/ethereum/sync_committee`
- Ethereum beacon:
  - `/v1/ethereum/beacon/height`
  - `/v1/ethereum/beacon/snapshot`
  - `/v1/ethereum/beacon/mmr_root`
  - `/v1/ethereum/beacon/mmr_proof`
  - `/v1/ethereum/beacon/light_client_proof`
- Ethereum execution:
  - `/v1/ethereum/execution/height`
  - `/v1/ethereum/execution/snapshot`
  - `/v1/ethereum/execution/mmr_root`
  - `/v1/ethereum/execution/mmr_proof`
  - `/v1/ethereum/execution/light_client_proof`

### 2) Verify coverage

Verify mode checks cryptographic and contract-level consistency:

- Block proof hash consistency (`/v1/blocks/block_proof` vs `/v1/blocks/mmr_proof` consistency).
- Bankai MMR proof contract checks.
- Ethereum MMR proof verification via `bankai_verify::bankai::mmr::MmrVerifier`.
- Light-client proof bundle checks, including:
  - STWO payload parse + hash-output verification.
  - returned MMR proof consistency with requested headers and expected roots.

### 3) OpenAPI coverage guard

`compat_live_openapi_coverage` fetches `/v1/openapi.json` and compares documented endpoints to compat case endpoint metadata.

Important details:

- mapping source is case metadata (`CompatCaseDef.endpoint`) only.
- docs-only routes are ignored (`/v1/openapi.json`, `/v1/swagger`, `/v1/rapidoc`).
- test fails on:
  - documented endpoint missing compat mapping.
  - stale compat mapping not present in OpenAPI.

## Matrix Logic

Core matrix variants (required) and edge variants (optional) are generated from `context.rs`.

Core variants include:

- `BankaiBlockFilterDto`: `latest`, `justified`, `finalized`, explicit `bankai_block_number`.
- `ProofFormatDto`: `bin`, `json`.
- `BankaiTargetBlockSelectorDto`: by `block_number`, by `block_hash`.
- hashing function: `keccak`.

Optional edge variants include targeted conflict/error-shape checks:

- conflicting filter (`selector + bankai_block_number`).
- conflicting target selector (`block_number + block_hash`).
- `poseidon` hashing where treated as edge compatibility behavior.

The report prints both case counts and planned matrix-variant counts:

- `required/optional` case status
- `matrix variants (planned): <total> total (<required> required, <optional> optional)`
- per-category matrix variant breakdown

## Logical Invariants Checked

Invariants live in `assertions.rs` and are reused in decode/verify flows.

Current checks include:

- selector ordering invariants where applicable: `latest >= justified >= finalized`.
- snapshot structural bounds:
  - `start_height <= end_height`
  - `finalized_height <= justified_height <= end_height`
- MMR snapshot sanity:
  - `elements_count > 0`
  - `leafs_count > 0`
  - non-empty peak sets
- proof contract assertions:
  - request/response selector and target consistency
  - hash/root/index/count consistency
  - expected hashing function propagation in proof payloads

## Required vs Optional Cases

`required: true` cases fail the suite.

`required: false` cases are tracked and reported as optional coverage (pass/skip/fail stats), but do not fail the suite by default.

Typical reasons a case is optional:

- fixture-dependent API behavior (for example, sync committee availability for a specific term).
- intentionally strict edge conflict checks.

## How To Run

### 1) Run full live compat suite (decode + verify + OpenAPI)

From repo root:

```bash
./scripts/run-compat-tests.sh
```

Defaults:

- `COMPAT_API_BASE_URL=http://127.0.0.1:8081`
- `COMPAT_VERBOSE=0`
- `COMPAT_COLOR=1`

### 2) Run directly with cargo

```bash
cargo test -p bankai-sdk --test compat_live -- --ignored --nocapture
```

### 3) Run only OpenAPI coverage

```bash
COMPAT_API_BASE_URL=http://127.0.0.1:8081 \
cargo test -p bankai-sdk --test compat_live compat_live_openapi_coverage -- --ignored --nocapture
```

### 4) Helpful env knobs

```bash
COMPAT_API_BASE_URL=http://127.0.0.1:8081
COMPAT_VERBOSE=1
COMPAT_COLOR=0
```

If local proxy settings interfere with localhost networking, run with:

```bash
ALL_PROXY= HTTP_PROXY= HTTPS_PROXY= NO_PROXY=127.0.0.1,localhost
```

## Troubleshooting

- `openapi endpoint coverage check failed`: verify API is reachable and exposes `/v1/openapi.json`.
- `required compatibility cases failed`: check the `repro:` curl emitted per failed case.
- `target_out_of_range`: backend resolved reference/target relationship was invalid for the chosen selector; inspect the request payload in the repro output.

## Updating Coverage

When backend adds or changes endpoints:

1. Add or update compat cases in `api/*`.
2. Ensure each decode/error-shape endpoint case has `endpoint: Some(...)` metadata.
3. Re-run `compat_live_openapi_coverage`.
4. If new request DTO/filter semantics were added, extend matrix builders in `context.rs` and assertions in `assertions.rs`.
