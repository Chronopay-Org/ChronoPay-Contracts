#![no_std]
//! ChronoPay time token contract with centralized constants module.

use soroban_sdk::{contract, contractimpl, contracttype, vec, Env, String, Symbol, Vec};

pub mod constants;

use constants::{CONTRACT_NAME, CONTRACT_VERSION, INITIAL_SLOT_SEQ};

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
    /// Contract schema version, set on first slot creation.
    Version,
}

#[contract]
pub struct ChronoPayContract;

#[contractimpl]
impl ChronoPayContract {
    /// Create a time slot with an auto-incrementing slot id.
    /// Returns the newly assigned slot id.
    pub fn create_time_slot(env: Env, professional: String, start_time: u64, end_time: u64) -> u32 {
        let _ = (professional, start_time, end_time);

        Self::ensure_version_set(&env);

        let current_seq: u32 = env
            .storage()
            .instance()
            .get(&DataKey::SlotSeq)
            .unwrap_or(INITIAL_SLOT_SEQ);

        let next_seq = current_seq.checked_add(1).expect("slot id overflow");

        env.storage().instance().set(&DataKey::SlotSeq, &next_seq);

        next_seq
    }

    /// Mint a time token for a slot (stub).
    pub fn mint_time_token(env: Env, slot_id: u32) -> Symbol {
        let _ = slot_id;
        Symbol::new(&env, "TIME_TOKEN")
    }

    /// Buy / transfer time token (stub).
    pub fn buy_time_token(env: Env, token_id: Symbol, buyer: String, seller: String) -> bool {
        let _ = (token_id, buyer, seller);
        env.storage()
            .instance()
            .set(&DataKey::Owner, &env.current_contract_address());
        true
    }

    /// Redeem time token (stub).
    pub fn redeem_time_token(env: Env, token_id: Symbol) -> bool {
        let _ = token_id;
        env.storage()
            .instance()
            .set(&DataKey::Status, &TimeTokenStatus::Redeemed);
        true
    }

    /// Hello-style entrypoint for CI and SDK sanity check.
    /// Uses [`CONTRACT_NAME`] from the constants module.
    pub fn hello(env: Env, to: String) -> Vec<String> {
        vec![&env, String::from_str(&env, CONTRACT_NAME), to]
    }

    /// Returns the contract schema version.
    pub fn version(env: Env) -> u32 {
        env.storage()
            .instance()
            .get(&DataKey::Version)
            .unwrap_or(CONTRACT_VERSION)
    }

    // ── Internal ──────────────────────────────────────────────────────

    /// Lazily write the contract version to storage on the first mutation.
    fn ensure_version_set(env: &Env) {
        if !env.storage().instance().has(&DataKey::Version) {
            env.storage()
                .instance()
                .set(&DataKey::Version, &CONTRACT_VERSION);
        }
    }
}

mod test;
