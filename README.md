# chronopay-contracts

Soroban smart contracts for **ChronoPay** — time tokenization and scheduling on the Stellar network.

## What's in this repo

- **Time token contract** (`contracts/chronopay`): Stub implementations for:
  - `create_time_slot(professional, start_time, end_time)`
  - `mint_time_token(slot_id)`
  - `buy_time_token(token_id, buyer, seller)`
  - `redeem_time_token(token_id)`

## Defensive Arithmetic Notes

`contracts/chronopay/src` now applies explicit arithmetic validation in
`create_time_slot` so timestamp and sequence handling do not rely on Rust's
debug-only overflow behavior.

- `end_time` must be strictly greater than `start_time`
- slot id allocation fails cleanly when the internal sequence reaches `u32::MAX`
- invalid arithmetic paths return typed contract errors that are covered by tests

Acceptance criteria for this change:

- slot creation succeeds for valid increasing timestamps
- zero-length and inverted time ranges are rejected
- slot sequence overflow is rejected without mutating storage
- the existing mint / buy / redeem stubs continue to behave as before

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
- **Build**: `cargo build`
- **Tests**: `cargo test`

## License

MIT
