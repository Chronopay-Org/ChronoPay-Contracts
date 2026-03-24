# chronopay-contracts

Soroban smart contracts for **ChronoPay** — time tokenization and scheduling on the Stellar network.

## What's in this repo

- **Time token contract** (`contracts/chronopay`): Stub implementations for:
  - `create_time_slot(professional, start_time, end_time)`
  - `mint_time_token(slot_id)`
  - `buy_time_token(token_id, buyer, seller)`
  - `redeem_time_token(token_id)`

## Integration test harness

The contract test module includes an integration-style harness that centralizes
environment setup and client calls across tests.

- Harness location: `contracts/chronopay/src/test.rs`
- Scope: lifecycle flow (`create -> mint -> buy -> redeem`) plus invalid inputs
- Goal: deterministic tests that are easy to extend and review

### Failure-mode handling covered by tests

- Empty `professional` is rejected.
- Invalid time ranges (`start_time >= end_time`) are rejected.
- `slot_id = 0` for minting is rejected.
- Unsupported token IDs are rejected for buy/redeem.
- Self-trades (`buyer == seller`) are rejected.
- Redeem is only valid from `Sold` state and replay attempts are rejected.

### Security assumptions (current architecture)

- This contract stores a simplified global owner/status for early development.
- Input validation is fail-fast using explicit panics for invalid state changes.
- Authorization and per-token ownership are intentionally minimal in this phase
  and should be expanded before production deployment.

## Prerequisites

- [Rust](https://www.rust-lang.org/) (stable)
- `rustfmt`: `rustup component add rustfmt`
- For deployment: [Stellar CLI](https://developers.stellar.org/docs/tools/developer-tools#stellar-cli) (optional)

## Setup

```bash
# Clone the repo (or use your fork)
git clone <repo-url>
cd chronopay-contracts

# Build
cargo build

# Run tests
cargo test

# Check formatting
cargo fmt --all -- --check
```

## Project layout

```
chronopay-contracts/
├── Cargo.toml              # Workspace definition
├── contracts/
│   └── chronopay/
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs      # Contract logic
│           └── test.rs     # Unit tests
└── .github/workflows/
    └── ci.yml              # CI: fmt, build, test
```

## Contributing

1. Fork the repo and create a branch from `main`.
2. Make changes; keep formatting clean: `cargo fmt`.
3. Ensure tests pass: `cargo test`.
4. Open a pull request. CI must pass (fmt check, build, tests).

## CI/CD

On every push and pull request to `main`, GitHub Actions runs:

- **Format**: `cargo fmt --all -- --check`
- **Clippy**: `cargo clippy --all-targets -- -D warnings`
- **Tests**: `cargo test`

## Acceptance criteria for this increment

- Integration harness exists and is used by contract tests.
- Happy path and failure paths are both covered.
- `cargo fmt --all -- --check` passes.
- `cargo clippy --all-targets -- -D warnings` passes.
- `cargo test` passes.

## License

MIT
