# chronopay-contracts

Soroban smart contracts for **ChronoPay** — time tokenization and scheduling on the Stellar network.

## What's in this repo

- **Time token contract** (`contracts/chronopay`): Soroban contract for:
  - `create_time_slot(professional, start_time, end_time)`
  - `mint_time_token(slot_id, owner)`
  - `buy_time_token(token_id, buyer, seller)`
  - `redeem_time_token(token_id, redeemer)`

## Access control and failure modes

The contract currently enforces owner-based authorization for token lifecycle operations in `contracts/chronopay/src`.

- `mint_time_token` requires `owner` authorization and rejects unknown slots.
- `buy_time_token` requires both `buyer` and `seller` authorization and rejects transfers from non-owners.
- `redeem_time_token` requires `redeemer` authorization and allows redemption only by the current owner.
- Redeemed tokens cannot be redeemed again.
- Missing tokens fail explicitly instead of mutating contract state.

Acceptance criteria for the redeem access-control flow:

- Only the current token owner can redeem a token.
- Ownership transfer changes who is allowed to redeem.
- Repeated redemption attempts fail.
- Invalid slot or token identifiers fail without changing ownership or status.
- Automated tests cover success, authorization failure, invalid input, and repeat-action failure paths.

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
