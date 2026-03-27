#![no_std]
//! ChronoPay time-token contract.
//!
//! Every mutating entry point requires Soroban `Address::require_auth` so that
//! only authorised callers can modify ledger state. The `initialize` function
//! establishes a one-time admin; subsequent calls panic.

use soroban_sdk::{contract, contractimpl, contracttype, vec, Address, Env, String, Symbol, Vec};

// ---------------------------------------------------------------------------
// Storage keys – O(1) lookup per operation
// ---------------------------------------------------------------------------

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    /// Auto-incrementing slot counter.
    SlotSeq,
    /// Contract administrator address.
    Admin,
    /// Owner address of a specific token (keyed by slot id).
    TokenOwner(u32),
    /// Status of a specific token (keyed by slot id).
    TokenStatus(u32),
    /// Professional who created a slot (keyed by slot id).
    SlotProfessional(u32),
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TimeTokenStatus {
    Available,
    Sold,
    Redeemed,
}

// ---------------------------------------------------------------------------
// Contract
// ---------------------------------------------------------------------------

#[contract]
pub struct ChronoPayContract;

#[contractimpl]
impl ChronoPayContract {
    // -----------------------------------------------------------------------
    // Admin bootstrap — callable once
    // -----------------------------------------------------------------------

    /// Initialise the contract with an admin address.
    /// Panics on repeat invocations (one-time setup).
    pub fn initialize(env: Env, admin: Address) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("already initialized");
        }
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
    }

    // -----------------------------------------------------------------------
    // Slot management
    // -----------------------------------------------------------------------

    /// Create a time slot. Only the `professional` themselves can call.
    /// Returns the auto-incremented slot id.
    pub fn create_time_slot(
        env: Env,
        professional: Address,
        start_time: u64,
        end_time: u64,
    ) -> u32 {
        professional.require_auth();

        if start_time >= end_time {
            panic!("start_time must be before end_time");
        }

        let current_seq: u32 = env
            .storage()
            .instance()
            .get(&DataKey::SlotSeq)
            .unwrap_or(0u32);

        let next_seq = current_seq.checked_add(1).expect("slot id overflow");

        env.storage().instance().set(&DataKey::SlotSeq, &next_seq);
        env.storage()
            .instance()
            .set(&DataKey::SlotProfessional(next_seq), &professional);

        next_seq
    }

    // -----------------------------------------------------------------------
    // Token lifecycle
    // -----------------------------------------------------------------------

    /// Mint a time token for a slot. Requires admin authorisation.
    /// Sets the token status to `Available`.
    pub fn mint_time_token(env: Env, admin: Address, slot_id: u32) -> Symbol {
        Self::require_admin(&env, &admin);

        env.storage()
            .instance()
            .set(&DataKey::TokenStatus(slot_id), &TimeTokenStatus::Available);

        Symbol::new(&env, "TIME_TOKEN")
    }

    /// Buy / transfer a time token. Requires the buyer to authorise.
    /// Transitions status from `Available` → `Sold`.
    pub fn buy_time_token(env: Env, buyer: Address, slot_id: u32) -> bool {
        buyer.require_auth();

        let status: TimeTokenStatus = env
            .storage()
            .instance()
            .get(&DataKey::TokenStatus(slot_id))
            .expect("token not minted");

        if status != TimeTokenStatus::Available {
            panic!("token not available");
        }

        env.storage()
            .instance()
            .set(&DataKey::TokenOwner(slot_id), &buyer);
        env.storage()
            .instance()
            .set(&DataKey::TokenStatus(slot_id), &TimeTokenStatus::Sold);

        true
    }

    /// Redeem a time token. Only the current token owner may redeem.
    /// Transitions status from `Sold` → `Redeemed`.
    pub fn redeem_time_token(env: Env, redeemer: Address, slot_id: u32) -> bool {
        redeemer.require_auth();

        let owner: Address = env
            .storage()
            .instance()
            .get(&DataKey::TokenOwner(slot_id))
            .expect("no owner for token");

        if redeemer != owner {
            panic!("caller is not the token owner");
        }

        let status: TimeTokenStatus = env
            .storage()
            .instance()
            .get(&DataKey::TokenStatus(slot_id))
            .expect("token not minted");

        if status != TimeTokenStatus::Sold {
            panic!("token not in sold state");
        }

        env.storage()
            .instance()
            .set(&DataKey::TokenStatus(slot_id), &TimeTokenStatus::Redeemed);

        true
    }

    // -----------------------------------------------------------------------
    // Read-only helpers
    // -----------------------------------------------------------------------

    /// Hello-style entry point for CI / SDK sanity check.
    pub fn hello(env: Env, to: String) -> Vec<String> {
        vec![&env, String::from_str(&env, "ChronoPay"), to]
    }

    // -----------------------------------------------------------------------
    // Internal
    // -----------------------------------------------------------------------

    /// Assert `caller` matches the stored admin and has provided auth.
    fn require_admin(env: &Env, caller: &Address) {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .expect("contract not initialized");

        if *caller != admin {
            panic!("caller is not admin");
        }
        caller.require_auth();
    }
}

mod test;
