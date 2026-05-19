# Soroban Split

A payment splitting smart contract on Stellar Soroban. Define a split once — specify recipients and percentage shares — then anyone can execute it to automatically distribute a payment on-chain.

[![CI](https://github.com/Inherited-will/soroban-split/actions/workflows/ci.yml/badge.svg)](https://github.com/Inherited-will/soroban-split/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## What It Does

Instead of manually transferring funds to multiple parties, Soroban Split lets you define the distribution logic once on-chain. Anyone can then call `execute_split` and the contract handles the math and transfers automatically.

**Example:** A freelancer creates a split — 70% to themselves, 20% to a business partner, 10% to a tax wallet. Every client payment goes through `execute_split` and distributes automatically. No manual transfers. No trust required.

**Use cases:**
- Freelancer revenue sharing
- DAO treasury distributions
- Protocol fee splits between teams
- Grant disbursements
- Team payment automation

## How Shares Work

Shares are in **basis points** — 10000 = 100%:

| Percentage | Basis Points |
|---|---|
| 70% | 7000 |
| 20% | 2000 |
| 10% | 1000 |
| 100% | 10000 |

All shares for a split must sum to exactly **10000**.

## Contract Functions

| Function | Who Calls It | Description |
|---|---|---|
| `initialize(admin)` | Deployer | Initialize the contract |
| `create_split(owner, name, recipients, shares)` | Anyone | Define a new payment split |
| `execute_split(caller, split_id, token, amount)` | Anyone | Distribute a payment according to the split |
| `update_split(owner, split_id, recipients, shares)` | Owner | Update recipients and shares |
| `deactivate_split(owner, split_id)` | Owner | Stop a split from being executed |
| `reactivate_split(owner, split_id)` | Owner | Re-enable a deactivated split |
| `get_split(split_id)` | Anyone | Get full split details |
| `get_recipients(split_id)` | Anyone | Get recipient addresses |
| `get_shares(split_id)` | Anyone | Get share values |
| `get_count()` | Anyone | Get total number of splits created |

## Split Lifecycle

```
Owner                      Contract                    Recipients
  |                            |                           |
  |-- create_split() --------> |                           |
  |                            | (split stored on-chain)   |
  |                            |                           |
Caller                         |                           |
  |-- execute_split() -------> |                           |
  |   (sends token payment)    |-- transfer 70% ---------> r1
  |                            |-- transfer 20% ---------> r2
  |                            |-- transfer 10% ---------> r3
  |                            |                           |
Owner                          |                           |
  |-- update_split() --------> | (new recipients/shares)  |
  |-- deactivate_split() ----> | (no more executions)     |
  |-- reactivate_split() ----> | (re-enabled)             |
```

## Build

**Prerequisites:**
- Rust 1.75+
- Soroban CLI ([install guide](https://developers.stellar.org/docs/smart-contracts/getting-started/setup))

```bash
# Install WASM target
rustup target add wasm32-unknown-unknown

# Clone and build
git clone https://github.com/Inherited-will/soroban-split
cd soroban-split
cargo build

# Run all tests
cargo test

# Build WASM contract
cargo build --target wasm32-unknown-unknown --release
```

## Deploy to Testnet

```bash
# Generate testnet identity (first time only)
soroban keys generate --global deployer --network testnet

# Fund from friendbot
soroban keys fund deployer --network testnet

# Deploy
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/soroban_split.wasm \
  --network testnet \
  --source deployer

# Initialize (replace CONTRACT_ID)
soroban contract invoke \
  --id CONTRACT_ID \
  --network testnet \
  --source deployer \
  -- initialize \
  --admin $(soroban keys address deployer)
```

## Project Structure

```
soroban-split/
├── src/
│   └── lib.rs           # Contract implementation + 13 tests
├── docs/
│   ├── ARCHITECTURE.md  # Data model and storage layout
│   └── DEPLOYMENT.md    # Testnet and mainnet deployment guide
├── .github/
│   └── workflows/
│       └── ci.yml       # Test + lint + WASM build
├── CONTRIBUTING.md
├── SECURITY.md
├── Cargo.toml
└── README.md
```

## Contributing

This project participates in the [Stellar Wave Program](https://drips.network/wave/stellar) on Drips. Contributors earn USDC rewards for resolving issues.

See [CONTRIBUTING.md](./CONTRIBUTING.md) for setup and contribution guidelines.

Issues are labeled by complexity:
- 🟢 `good first issue` — Tests, docs, small fixes (100 pts)
- 🟡 `enhancement` — New features, improvements (150 pts)
- 🔴 `high complexity` — Architecture changes (200 pts)

## License

MIT — see [LICENSE](./LICENSE) for details.

