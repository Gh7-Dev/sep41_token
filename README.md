# BlockTicket (BKT) — Soroban SEP-41 Token

A learning project implementing a fungible token contract on Stellar using Rust and the Soroban SDK. The contract exposes SEP-41-style token operations and includes an admin-controlled mint function.

> **Status:** Experimental / educational. Review and test the contract before using it with real assets. This repository has not been presented as a security-audited production token.

## Implemented functionality

- Initialize token metadata: name, symbol, decimals, admin, and initial supply.
- Mint tokens to an address, authorized by the configured admin.
- Query balances and allowances.
- Approve a spender to transfer or burn tokens within an allowance.
- Transfer tokens and transfer on behalf of an owner.
- Burn tokens directly or through an allowance.
- Query token name, symbol, and decimals. Total supply is maintained internally, but the current contract does not expose a public total-supply getter.

The current test setup initializes a token named `BlockTicket` with symbol `BKT`, 7 decimals, and an initial supply of 1,000,000 units.

## Technology

- Rust
- Soroban SDK
- Stellar smart-contract tooling
- Cargo workspace

## Repository structure

```text
.
├── Cargo.toml
├── contracts/
│   └── sep41_token/
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs
│           └── test.rs
└── README.md
```

## Build and test

Install Rust and the Stellar CLI / Soroban tooling first.

```bash
cargo test
cargo build --target wasm32v1-none --release
```

If you use the Stellar CLI to build the contract, follow the current Stellar CLI instructions for your installed version.

## Important implementation notes

- The admin address is stored during initialization and must authorize minting.
- The current implementation stores token balances, allowances, and metadata in Soroban persistent storage.
- The current `approve` implementation accepts an expiration-ledger argument but does not enforce it when reading or spending the allowance.
- Before production use, review initialization safety, amount validation, storage lifecycle, and full SEP-41 compatibility; the presence of token interface methods alone is not a security or compliance guarantee.

## Learning goals

This project explores Soroban contract structure, authorization, persistent storage, token balances, allowances, transfers, and supply accounting.
