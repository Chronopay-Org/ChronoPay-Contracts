#![no_std]
//! ChronoPay time token contract.
//!
//! Includes pause/unpause emergency switch controlled by the contract admin.
//! When paused, all state-mutating operations are blocked.

use soroban_sdk::{contract, contractimpl, contracttype, vec, Address, Env, String, Symbol, Vec};

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
    /// Stores the admin Address for pause/unpause authorization.
    Admin,
    /// Stores the paused state as a bool.
    Paused,
}

#[contract]
pub struct ChronoPayContract;

#[contractimpl]
impl ChronoPayContract {
    /// Initialize the contract with an admin address.
    /// Must be called once before any other operations.
    /// Panics if already initialized.
    pub fn initialize(env: Env, admin: Address) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("already initialized");
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Paused, &false);
    }

    /// Pause the contract. Only callable by admin.
    /// When paused, all state-mutating operations are blocked.
    pub fn pause(env: Env, admin: Address) {
        admin.require_auth();
        Self::require_admin(&env, &admin);
        env.storage().instance().set(&DataKey::Paused, &true);
    }

    /// Unpause the contract. Only callable by admin.
    pub fn unpause(env: Env, admin: Address) {
        admin.require_auth();
        Self::require_admin(&env, &admin);
        env.storage().instance().set(&DataKey::Paused, &false);
    }

    /// Returns true if the contract is currently paused.
    pub fn is_paused(env: Env) -> bool {
        env.storage()
            .instance()
            .get(&DataKey::Paused)
            .unwrap_or(false)
    }

    /// Create a time slot with an auto-incrementing slot id.
    /// Returns the newly assigned slot id.
    /// Panics if contract is paused.
    pub fn create_time_slot(
        env: Env,
        professional: String,
        start_time: u64,
        end_time: u64,
    ) -> u32 {
        Self::require_not_paused(&env);
        let _ = (professional, start_time, end_time);

        let current_seq: u32 = env
            .storage()
            .instance()
            .get(&DataKey::SlotSeq)
            .unwrap_or(0u32);

        let next_seq = current_seq.checked_add(1).expect("slot id overflow");

        env.storage()
            .instance()
            .set(&DataKey::SlotSeq, &next_seq);

        next_seq
    }

    /// Mint a time token for a slot (stub).
    /// Panics if contract is paused.
    pub fn mint_time_token(env: Env, slot_id: u32) -> Symbol {
        Self::require_not_paused(&env);
        let _ = slot_id;
        Symbol::new(&env, "TIME_TOKEN")
    }

    /// Buy / transfer time token (stub).
    /// Panics if contract is paused.
    pub fn buy_time_token(env: Env, token_id: Symbol, buyer: String, seller: String) -> bool {
        Self::require_not_paused(&env);
        let _ = (token_id, buyer, seller);
        env.storage()
            .instance()
            .set(&DataKey::Owner, &env.current_contract_address());
        true
    }

    /// Redeem time token (stub).
    /// Panics if contract is paused.
    pub fn redeem_time_token(env: Env, token_id: Symbol) -> bool {
        Self::require_not_paused(&env);
        let _ = token_id;
        env.storage()
            .instance()
            .set(&DataKey::Status, &TimeTokenStatus::Redeemed);
        true
    }

    /// Hello-style entrypoint for CI and SDK sanity check.
    /// Not affected by pause — read-only.
    pub fn hello(env: Env, to: String) -> Vec<String> {
        vec![&env, String::from_str(&env, "ChronoPay"), to]
    }

    /// Panics if the contract is paused.
    fn require_not_paused(env: &Env) {
        let paused: bool = env
            .storage()
            .instance()
            .get(&DataKey::Paused)
            .unwrap_or(false);
        if paused {
            panic!("contract is paused");
        }
    }

    /// Panics if the provided address is not the stored admin.
    fn require_admin(env: &Env, caller: &Address) {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .expect("not initialized");
        if admin != *caller {
            panic!("unauthorized: caller is not admin");
        }
    }
}

mod test;