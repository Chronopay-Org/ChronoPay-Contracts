# Batch Attestation Submission

## Overview
ChronoPay supports batch creation of time slots in a single contract interaction.
All-or-nothing (atomic) semantics are enforced: if any slot in a batch fails,
no state changes from that batch are persisted.

## Assumptions
- Slot IDs are auto-incrementing and never reused.
- Each `Env` instance starts its counter from zero (fresh state).
- Batch operations are sequential calls to `create_time_slot`.
- Minting, buying, and redeeming are valid for every slot created in a batch.

## Expected Behavior
| Scenario | Expected Outcome |
|---|---|
| All slots valid | All slot IDs committed, sequential |
| Single-slot batch | Behaves identically to a direct call |
| Counter after batch | Persists correctly for subsequent calls |
| Independent environments | Each starts counter from 1, no leakage |
| Mint after batch | Every batch slot is mintable |
| Redeem after batch mint | Every token is redeemable |
| Buy after batch mint | Every token is buyable |

## Security Assumptions
- Slot ID overflow is caught by `checked_add` and panics safely.
- No cross-environment state leakage between test runs.
- Batch operations do not bypass owner or status checks.

## Test Coverage
All atomicity tests live in `contracts/chronopay/src/test.rs` under the
`BATCH SUBMISSION FAILURE ATOMICITY TESTS` section (Issue #127).
