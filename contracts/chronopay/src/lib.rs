#![no_std]
//! ChronoPay time token contract — stub for create_time_slot, mint_time_token, buy_time_token, redeem_time_token.
//!
//! MIGRATION NOTES
//! - Versioning: this contract persistently stores a `contract_version` as `u32`. The initial
//!   version is `1`. Always increment for storage layout changes. Never reset or remove.
//! - Storage layout: keys are defined via `DataKey` enum. Do not rename existing variants.
//!   Adding new variants is backward-compatible; removing or renaming breaks compatibility.
//! - Field rules:
//!   - Additive-only changes are safe: add new `DataKey` variants; keep old keys intact.
//!   - If a stored struct/enum is introduced later, prefer new keys rather than changing
//!     the type or encoding under an existing key.
//! - Backward compatibility: readers should tolerate missing keys by applying defaults.
//! - Upgrade safety: avoid logic that assumes non-empty state without checking presence.
//! - Version tracking: gate new logic paths on `contract_version` so old states remain valid.
//! - Failure scenarios to avoid:
//!   - Incompatible state due to renamed keys.
//!   - Logic relying on fields not present in earlier versions.
//!   - Unauthorized upgrades (introduce admin gating if adding active migrations).
//! - Expected migration approach:
//!   - Introduce new keys and logic guarded by `>=` version checks.
//!   - Provide one-way migration that fills new keys lazily when accessed.
//!   - Keep read paths tolerant of both old and new shapes during transition windows.

use soroban_sdk::{contract, contractimpl, contracttype, vec, Env, String, Symbol, Vec};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TimeTokenStatus {
    Available,
    Sold,
    Redeemed,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    SlotSeq,
    Owner,
    Status,
    /// Contract storage/logic version (u32). Initial: 1.
    ContractVersion,
}

#[contract]
pub struct ChronoPayContract;

#[contractimpl]
impl ChronoPayContract {
    /// Ensure the contract has a version set; initialize to 1 if missing.
    fn ensure_version(env: &Env) -> u32 {
        let v: u32 = env
            .storage()
            .instance()
            .get(&DataKey::ContractVersion)
            .unwrap_or(0u32);
        if v == 0 {
            let initial = 1u32;
            env.storage()
                .instance()
                .set(&DataKey::ContractVersion, &initial);
            initial
        } else {
            v
        }
    }

    /// Get the current contract version (0 if not initialized by any entrypoint yet).
    pub fn get_contract_version(env: Env) -> u32 {
        env.storage()
            .instance()
            .get(&DataKey::ContractVersion)
            .unwrap_or(0u32)
    }

    /// Create a time slot with an auto-incrementing slot id.
    /// Returns the newly assigned slot id.
    pub fn create_time_slot(env: Env, professional: String, start_time: u64, end_time: u64) -> u32 {
        let _ = Self::ensure_version(&env);
        let _ = (professional, start_time, end_time);

        let current_seq: u32 = env
            .storage()
            .instance()
            .get(&DataKey::SlotSeq)
            .unwrap_or(0u32);

        let next_seq = current_seq.checked_add(1).expect("slot id overflow");

        env.storage().instance().set(&DataKey::SlotSeq, &next_seq);

        next_seq
    }

    /// Mint a time token for a slot (stub).
    pub fn mint_time_token(env: Env, slot_id: u32) -> Symbol {
        let _ = Self::ensure_version(&env);
        let _ = slot_id;
        Symbol::new(&env, "TIME_TOKEN")
    }

    /// Buy / transfer time token (stub). In full implementation: token_id, buyer, seller, price.
    pub fn buy_time_token(env: Env, token_id: Symbol, buyer: String, seller: String) -> bool {
        let _ = Self::ensure_version(&env);
        let _ = (token_id, buyer, seller);
        env.storage()
            .instance()
            .set(&DataKey::Owner, &env.current_contract_address());
        true
    }

    /// Redeem time token (stub). In full implementation: token_id, marks as redeemed.
    pub fn redeem_time_token(env: Env, token_id: Symbol) -> bool {
        let _ = Self::ensure_version(&env);
        let _ = token_id;
        env.storage()
            .instance()
            .set(&DataKey::Status, &TimeTokenStatus::Redeemed);
        true
    }

    /// Hello-style entrypoint for CI and SDK sanity check.
    pub fn hello(env: Env, to: String) -> Vec<String> {
        let _ = Self::ensure_version(&env);
        vec![&env, String::from_str(&env, "ChronoPay"), to]
    }
}

mod test;
