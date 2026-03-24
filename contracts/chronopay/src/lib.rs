#![no_std]
//! ChronoPay time token contract.
//!
//! This module intentionally keeps a compact state model suitable for early
//! integration testing while still enforcing basic security and correctness
//! constraints on inputs and state transitions.

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
}

#[contract]
pub struct ChronoPayContract;

#[contractimpl]
impl ChronoPayContract {
    /// Create a time slot with an auto-incrementing slot id.
    ///
    /// Failure modes:
    /// - panics if `professional` is empty
    /// - panics if `start_time >= end_time`
    /// - panics on slot sequence overflow
    /// Returns the newly assigned slot id.
    pub fn create_time_slot(env: Env, professional: String, start_time: u64, end_time: u64) -> u32 {
        if professional.is_empty() {
            panic!("professional cannot be empty");
        }
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

        next_seq
    }

    /// Mint a time token for a slot.
    ///
    /// Failure modes:
    /// - panics if `slot_id == 0`
    pub fn mint_time_token(env: Env, slot_id: u32) -> Symbol {
        if slot_id == 0 {
            panic!("slot id must be non-zero");
        }
        Symbol::new(&env, "TIME_TOKEN")
    }

    /// Buy / transfer time token.
    ///
    /// Failure modes:
    /// - panics if token id is not supported
    /// - panics if buyer or seller is empty
    /// - panics if buyer == seller
    pub fn buy_time_token(env: Env, token_id: Symbol, buyer: String, seller: String) -> bool {
        if token_id != Symbol::new(&env, "TIME_TOKEN") {
            panic!("unsupported token id");
        }
        if buyer.is_empty() || seller.is_empty() {
            panic!("buyer and seller must be non-empty");
        }
        if buyer == seller {
            panic!("buyer and seller must differ");
        }

        env.storage().instance().set(&DataKey::Owner, &buyer);
        env.storage()
            .instance()
            .set(&DataKey::Status, &TimeTokenStatus::Sold);
        true
    }

    /// Redeem time token.
    ///
    /// Failure modes:
    /// - panics if token id is not supported
    /// - panics unless the token status is `Sold`
    pub fn redeem_time_token(env: Env, token_id: Symbol) -> bool {
        if token_id != Symbol::new(&env, "TIME_TOKEN") {
            panic!("unsupported token id");
        }

        let status: TimeTokenStatus = env
            .storage()
            .instance()
            .get(&DataKey::Status)
            .unwrap_or(TimeTokenStatus::Available);
        if status != TimeTokenStatus::Sold {
            panic!("token must be sold before redemption");
        }

        env.storage()
            .instance()
            .set(&DataKey::Status, &TimeTokenStatus::Redeemed);
        true
    }

    /// Read the current owner value recorded by `buy_time_token`.
    pub fn current_owner(env: Env) -> Option<String> {
        env.storage().instance().get(&DataKey::Owner)
    }

    /// Read the current token lifecycle status.
    pub fn current_status(env: Env) -> Option<TimeTokenStatus> {
        env.storage().instance().get(&DataKey::Status)
    }

    /// Hello-style entrypoint for CI and SDK sanity check.
    pub fn hello(env: Env, to: String) -> Vec<String> {
        vec![&env, String::from_str(&env, "ChronoPay"), to]
    }
}

mod test;
