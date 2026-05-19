# Architecture

## Overview

Soroban Split is a single smart contract deployed on Stellar. All split configurations live on-chain. There is no backend or database — everything is stored in Soroban's persistent storage.

## Core Concept

A **Split** defines: who receives payment (recipients), how much each receives (shares in basis points), and who controls the split (owner). Once created, anyone can execute the split by sending a token payment — the contract handles the distribution math and transfers.

## Data Model

### Split struct

| Field | Type | Description |
|---|---|---|
| `split_id` | `u64` | Unique identifier |
| `owner` | `Address` | Who controls this split |
| `name` | `String` | Human-readable label |
| `recipients` | `Vec<Address>` | Payment recipients |
| `shares` | `Vec<u32>` | Shares in basis points (must sum to 10000) |
| `active` | `bool` | Whether the split can be executed |

### Storage Layout

| Key | Storage Type | Description |
|---|---|---|
| `Admin` | `instance` | Contract admin address |
| `Counter` | `instance` | Global split ID counter |
| `Split(split_id)` | `persistent` | Individual split records |

`instance` storage shares a TTL with the contract — used for global config.
`persistent` storage has independent TTL per key — used for split records that must survive long-term.

## Share Math

```
recipient_amount = total_amount × share / 10_000
```

Example: 10,000 USDC with shares [7000, 2000, 1000]:
- Recipient 1: 10,000 × 7000 / 10,000 = 7,000 USDC
- Recipient 2: 10,000 × 2000 / 10,000 = 2,000 USDC
- Recipient 3: 10,000 × 1000 / 10,000 = 1,000 USDC

Rounding: integer division may leave dust (< 1 stroop) for small amounts. Zero-amount transfers are skipped.

## Access Control

| Function | Auth Required |
|---|---|
| `initialize` | Admin |
| `create_split` | Owner (signer) |
| `execute_split` | Caller (any signer) |
| `update_split` | Owner (must match split.owner) |
| `deactivate_split` | Owner (must match split.owner) |
| `reactivate_split` | Owner (must match split.owner) |
| All `get_*` functions | None (read-only) |

## Events

| Event | Data |
|---|---|
| `initialized` | admin |
| `split_created` | split_id, owner |
| `split_executed` | split_id, caller, amount |
| `split_updated` | split_id, owner |
| `split_deactivated` | split_id, owner |
| `split_reactivated` | split_id, owner |
