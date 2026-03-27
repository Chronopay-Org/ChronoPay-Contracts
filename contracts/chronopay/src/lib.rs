#![no_std]
//! ChronoPay time token contract — stub for create_time_slot, mint_time_token, buy_time_token, redeem_time_token.

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
}

#[contract]
pub struct ChronoPayContract;

#[contractimpl]
impl ChronoPayContract {
    /// Create a time slot with an auto-incrementing slot id.
    /// Returns the newly assigned slot id.
    ///
    /// Security Assumption:
    /// - Only the professional can create slots for themselves.
    /// - Time intervals must be valid (start < end).
    pub fn create_time_slot(
        env: Env,
        professional: Address,
        start_time: u64,
        end_time: u64,
    ) -> u32 {
        // Enforce authorization: only the professional can create their own slot.
        professional.require_auth();

        // Security Invariant: Start time must be before end time.
        if start_time >= end_time {
            panic!("invalid time range: start_time must be less than end_time");
        }

        let current_seq: u32 = env
            .storage()
            .instance()
            .get(&DataKey::SlotSeq)
            .unwrap_or(0u32);

        // Security Check: Prevent overflow for slot IDs.
        let next_seq = current_seq.checked_add(1).expect("slot id overflow");

        env.storage().instance().set(&DataKey::SlotSeq, &next_seq);

        next_seq
    }

    /// Mint a time token for a slot (stub).
    pub fn mint_time_token(env: Env, slot_id: u32) -> Symbol {
        let _ = slot_id;
        // In production, this would verify that the slot_id exists and is owned by the caller.
        Symbol::new(&env, "TIME_TOKEN")
    }

    /// Buy / transfer time token (stub). In full implementation: token_id, buyer, seller, price.
    ///
    /// Security Assumption:
    /// - Buyer must authorize the purchase (handled by buyer.require_auth in production).
    pub fn buy_time_token(env: Env, token_id: Symbol, buyer: Address, seller: Address) -> bool {
        let _ = (token_id, seller); // seller is still unused in this stub

        // Simulating authorization for the buyer.
        buyer.require_auth();

        env.storage()
            .instance()
            .set(&DataKey::Owner, &env.current_contract_address());
        true
    }

    /// Redeem time token (stub).
    ///
    /// Security Assumption:
    /// - Tokens cannot be redeemed twice.
    /// - Only the current owner (or authorized professional) can redeem.
    pub fn redeem_time_token(env: Env, token_id: Symbol) -> bool {
        let _ = token_id;

        // Security Invariant: Check if already redeemed.
        let status: TimeTokenStatus = env
            .storage()
            .instance()
            .get(&DataKey::Status)
            .unwrap_or(TimeTokenStatus::Available);

        if status == TimeTokenStatus::Redeemed {
            panic!("token already redeemed");
        }

        env.storage()
            .instance()
            .set(&DataKey::Status, &TimeTokenStatus::Redeemed);
        true
    }

    /// Hello-style entrypoint for CI and SDK sanity check.
    pub fn hello(env: Env, to: String) -> Vec<String> {
        vec![&env, String::from_str(&env, "ChronoPay"), to]
    }
}

mod test;
