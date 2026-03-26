# ProofChain Soroban Smart Contract

This repository contains the Stellar Soroban smart contract for **ProofChain**, focused only on on-chain ownership and transaction proof.

The contract acts as the tamper-proof ownership ledger for digital licenses:

- Mint a new license record.
- Transfer ownership for exclusive sales.
- Record non-exclusive sales without changing ownership.
- Query creator and current owner.

## Why this contract exists

In the full ProofChain system, image hashing, similarity checks, storage, and buyer dashboards happen off-chain. This contract keeps the part that must be immutable:

- who created a license,
- who currently owns it,
- and what transaction type was recorded.

## Contract overview

Contract: `ProofChainContract`

### Persistent storage

The contract stores values keyed by `license_id`:

- `Owner(license_id) -> Address`
- `Creator(license_id) -> Address`
- `LicenseType(license_id) -> u32`
- `Price(license_id) -> i128`

### Transaction types

`TxType` enum values:

- `Mint = 0`
- `Exclusive = 1`
- `NonExclusive = 2`

## Public methods

### `mint(env, license_id, creator, license_type, price)`

Creates a new on-chain license record.

Behavior:

- Requires `creator` authorization.
- Fails if the `license_id` already exists.
- Sets owner and creator to `creator`.
- Saves `license_type` and `price`.
- Emits a `mint` event.

### `transfer_exclusive(env, license_id, seller, buyer, price)`

Transfers ownership for an exclusive sale.

Behavior:

- Requires `seller` authorization.
- Fails if license does not exist.
- Fails if `seller` is not the current owner.
- Updates owner to `buyer`.
- Updates stored `price`.
- Emits a `sale` event tagged as `Exclusive`.

### `record_nonexclusive(env, license_id, seller, buyer, price)`

Records a non-exclusive purchase without changing owner.

Behavior:

- Requires `seller` authorization.
- Fails if license does not exist.
- Fails if `seller` is not the current owner.
- Keeps ownership unchanged.
- Emits a `sale` event tagged as `NonExclusive`.

### `owner_of(env, license_id) -> Address`

Returns current owner for a license.

### `creator_of(env, license_id) -> Address`

Returns original creator for a license.

## Event model

The contract emits two event topics:

- `("mint", license_id)` with payload `(TxType::Mint, creator, price)`
- `("sale", license_id)` with payload `(tx_type, seller, buyer, price)`

Where `tx_type` is:

- `1` for exclusive sales
- `2` for non-exclusive sales

## Local development

### Prerequisites

- Rust stable toolchain
- Cargo

### Run tests

```bash
cargo test
```

### Build contract artifact (WASM)

```bash
cargo build --target wasm32-unknown-unknown --release
```

Output artifact:

- `target/wasm32-unknown-unknown/release/proofchain_contract.wasm`

## Current test coverage

Unit tests cover:

1. Mint sets owner and creator correctly.
2. Exclusive transfer updates owner.
3. Non-exclusive record does not change owner.

## Notes

- `license_type` is currently stored as `u32` for flexibility with backend-defined mapping.
- Contract panics on invalid states (`license already exists`, `license not found`, `not current owner`).
- This contract intentionally does not implement payment transfer logic; payment handling can be done in backend workflows or separate token/payment contracts.
