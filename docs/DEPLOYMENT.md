# Deployment Guide

## Testnet Deployment

### 1. Build

```bash
rustup target add wasm32-unknown-unknown
cargo build --target wasm32-unknown-unknown --release
```

Output: `target/wasm32-unknown-unknown/release/soroban_split.wasm`

### 2. Setup Testnet Identity

```bash
soroban keys generate --global deployer --network testnet
soroban keys fund deployer --network testnet
```

### 3. Deploy

```bash
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/soroban_split.wasm \
  --network testnet \
  --source deployer
```

Note the CONTRACT_ID output.

### 4. Initialize

```bash
soroban contract invoke \
  --id CONTRACT_ID \
  --network testnet \
  --source deployer \
  -- initialize \
  --admin $(soroban keys address deployer)
```

### 5. Create a Split

```bash
soroban contract invoke \
  --id CONTRACT_ID \
  --network testnet \
  --source my-account \
  -- create_split \
  --owner $(soroban keys address my-account) \
  --name "My Revenue Split" \
  --recipients '["GADDRESS1...", "GADDRESS2..."]' \
  --shares '[7000, 3000]'
```

### 6. Execute a Split

```bash
soroban contract invoke \
  --id CONTRACT_ID \
  --network testnet \
  --source caller-account \
  -- execute_split \
  --caller $(soroban keys address caller-account) \
  --split_id 1 \
  --token USDC_TOKEN_ADDRESS \
  --amount 1000000
```

### 7. Query a Split

```bash
soroban contract invoke \
  --id CONTRACT_ID \
  --network testnet \
  -- get_split \
  --split_id 1
```
