# ChronoPay Source Notes

## Domain types

`domain.rs` owns the Soroban contract domain enums that are shared across
entrypoints:

- `DataKey` defines the canonical instance-storage coordinates.
- `TimeTokenStatus` defines the persisted token lifecycle states.

This separation keeps `lib.rs` focused on contract behavior while preserving a
single source of truth for serialized contract types.

## Acceptance criteria

- All contract entrypoints compile and continue using the same storage keys.
- Domain types remain exported from the crate for tests and downstream module
  use.
- Failure behavior is unchanged:
  - `create_time_slot` traps on `u32` overflow instead of wrapping slot ids.
  - Stub entrypoints only mutate the storage keys they are responsible for.

## Security notes

- Centralizing storage keys lowers the chance of accidental key drift that could
  orphan state or overwrite unrelated values.
- Overflow handling remains explicit and fail-closed for slot sequencing.
- Tests assert persisted owner and status values so unauthorized or accidental
  state-shape regressions are easier to catch during review.
