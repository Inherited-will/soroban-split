# Contributing to Soroban Split

This project participates in the [Stellar Wave Program](https://drips.network/wave/stellar) on Drips. Contributors earn USDC rewards for resolving issues.

## Prerequisites

- [Rust](https://rustup.rs/) 1.75+
- [Soroban CLI](https://developers.stellar.org/docs/smart-contracts/getting-started/setup)

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install WASM target
rustup target add wasm32-unknown-unknown
```

## Setup

```bash
git clone https://github.com/YOUR_USERNAME/soroban-split
cd soroban-split
cargo build
cargo test
```

All tests must pass before you start working on an issue.

## Workflow

1. Browse [open issues](https://github.com/Inherited-will/soroban-split/issues)
2. Comment on the issue you want to work on and wait to be assigned
3. Fork the repo and create a branch:

```bash
git checkout -b feat/your-feature-name
```

4. Make your changes
5. Run the full check suite:

```bash
cargo fmt          # format
cargo clippy       # lint — fix all warnings
cargo test         # all tests must pass
```

6. Commit using conventional commits and open a PR against `main`

## Commit Format

```
feat: add transfer ownership function
fix: correct rounding in execute_split
test: add edge case for zero-share recipient
docs: document basis points arithmetic
```

## PR Requirements

- [ ] `cargo fmt` passes
- [ ] `cargo clippy` passes with no warnings
- [ ] `cargo test` — all tests pass
- [ ] New functionality has tests
- [ ] PR references the issue (`Closes #123`)

## Key Concepts

**Basis points:** Shares are expressed in basis points where 10000 = 100%. So 70% = 7000, 20% = 2000, 10% = 1000. All shares for a split must sum to exactly 10000.

**Persistent storage:** Individual split records use `persistent()` storage. The global counter uses `instance()` storage.

**Auth:** Every state-changing function that requires owner permissions calls `owner.require_auth()` before any logic.
