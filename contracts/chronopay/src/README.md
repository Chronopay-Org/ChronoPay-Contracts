# ChronoPay Source Notes

## Domain types

`domain.rs` owns the Soroban contract domain enums that are shared across
entrypoints:

- `DataKey` defines the canonical instance-storage coordinates.
- `TimeTokenStatus` defines the persisted token lifecycle states.

This separation keeps `lib.rs` focused on contract behavior while preserving a
single source of truth for serialized contract types.

## Allocation strategy

`lib.rs` lazily caches immutable contract metadata in instance storage:

- `DataKey::ContractName` stores the `ChronoPay` display string used by
  `hello`.
- `DataKey::TokenSymbol` stores the `TIME_TOKEN` symbol used by
  `mint_time_token`.

This avoids rebuilding the same host `String` and `Symbol` values on every
call while keeping the external contract interface unchanged.

## Acceptance criteria

- All contract entrypoints compile and continue using the same storage keys.
- Domain types remain exported from the crate for tests and downstream module
  use.
- Repeated `hello` and `mint_time_token` calls reuse cached metadata instead of
  reconstructing literals on every invocation after the first cache miss.
- Failure behavior is unchanged:
  - `create_time_slot` traps on `u32` overflow instead of wrapping slot ids.
  - Stub entrypoints only mutate the storage keys they are responsible for.
  - Metadata caches are restored lazily when missing.

## Security notes

- Centralizing storage keys lowers the chance of accidental key drift that could
  orphan state or overwrite unrelated values.
- Immutable metadata is cached only in bounded instance storage, so the
  optimization does not introduce unbounded growth or user-controlled writes.
- Overflow handling remains explicit and fail-closed for slot sequencing.
- Tests assert persisted owner and status values so unauthorized or accidental
  state-shape regressions are easier to catch during review.
- If an upgrade or manual state mutation stores an unexpected type at a metadata
  cache key, Soroban deserialization will trap rather than silently returning an
  incorrect value.
