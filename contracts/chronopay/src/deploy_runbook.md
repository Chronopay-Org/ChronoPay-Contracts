# ChronoPay Soroban Deployment Runbook

This guide outlines the production-grade deployment process for the ChronoPay smart contract on the Stellar network using Soroban.

## Prerequisites
- [Stellar CLI](https://developers.stellar.org/docs/tools/developer-tools#stellar-cli) installed globally.
- Rust toolchain configured for the `wasm32-unknown-unknown` target.
- Configured network and identity (e.g., `testnet` or `mainnet`, and an `admin` identity).

## 1. Build & Optimize
Compile the contract into a WASM binary and optimize it for deployment to minimize size and cost:

```bash
cargo build --target wasm32-unknown-unknown --release
soroban contract optimize --wasm target/wasm32-unknown-unknown/release/chronopay.wasm
```

## 2. Deploy
Deploy the optimized WASM to the target network. Run this command from the workspace root:

```bash
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/chronopay.wasm \
  --source admin \
  --network testnet
```
*Make sure to record the returned `CONTRACT_ID` for the next steps.*

## 3. Initialization (Mandatory)
The contract **must** be initialized immediately after deployment to bind the Admin address and secure the contract state. **Failure to do so will leave the contract vulnerable to unauthorized initialization.**

```bash
soroban contract invoke \
  --id <CONTRACT_ID> \
  --source admin \
  --network testnet \
  -- \
  init --admin <ADMIN_PUBLIC_KEY>
```

### Security Assumptions & Failure-Mode Handling
- **Double-Initialization Prevention**: The `init` function uses `env.storage().instance().has(&DataKey::Admin)`. If called a second time, the transaction will explicitly panic with `already initialized`. This protects the contract from takeover attempts.
- **Out of Gas / Insufficient Fees**: If the initialization transaction fails due to fees, the ledger rolls back safely. Ensure you have a sufficient XLM balance on the deploying account.
- **State Rollback**: Because `init` is the sole mechanism establishing ownership, if the `init` transaction fails (e.g., wrong network congestion), retry it. The contract state remains completely uninitialized until a successful `init` transaction is confirmed.

## 4. Acceptance Criteria
To confirm a secure and complete deployment:
1. The deployed WASM hash matches your locally compiled binary hash.
2. The `init` invocation succeeds and the transaction is immutably recorded on the ledger.
3. Invoking `init` a second time with any arguments predictably fails with an `already initialized` panic.
