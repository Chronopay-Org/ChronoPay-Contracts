//! Domain types shared by ChronoPay contract entrypoints.
//!
//! Acceptance criteria for this module:
//! - Storage keys stay centralized and reused across entrypoints.
//! - Token status values remain serializable Soroban contract types.
//! - Failure behavior stays explicit at the call sites that consume these types.

use soroban_sdk::contracttype;

/// Lifecycle states for a ChronoPay time token.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TimeTokenStatus {
    Available,
    Sold,
    Redeemed,
}

/// Canonical instance-storage keys used by the ChronoPay contract.
///
/// Keeping these keys in one module reduces the risk of duplicate or
/// inconsistent storage coordinates during future feature work.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    SlotSeq,
    Owner,
    Status,
    ContractName,
    TokenSymbol,
}
