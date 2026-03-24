#![no_std]
//! ChronoPay time token contract — stub for create_time_slot, mint_time_token, buy_time_token, redeem_time_token.

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
    /// Returns the newly assigned slot id.
    pub fn create_time_slot(env: Env, professional: String, start_time: u64, end_time: u64) -> u32 {
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
        let _ = slot_id;
        Symbol::new(&env, "TIME_TOKEN")
    }

    /// Buy / transfer time token (stub). In full implementation: token_id, buyer, seller, price.
    pub fn buy_time_token(env: Env, token_id: Symbol, buyer: String, seller: String) -> bool {
        let _ = (token_id, buyer, seller);
        env.storage()
            .instance()
            .set(&DataKey::Owner, &env.current_contract_address());
        true
    }

    /// Redeem time token. In full implementation: token_id, marks as redeemed.
    /// Emits a redemption event for indexers to track the completion of slots.
    pub fn redeem_time_token(env: Env, token_id: Symbol) -> bool {
        // ASSUMPTION: The token is valid and caller is authorized to redeem it.
        // CONSTRAINT: Can only be redeemed once (authorization logic to be fully implemented).
        env.storage()
            .instance()
            .set(&DataKey::Status, &TimeTokenStatus::Redeemed);

        // BEHAVIOR: Emit a redemption event for indexers to track finalized slots.
        env.events()
            .publish((Symbol::new(&env, "redemption"),), token_id);

        true
    }

    /// Hello-style entrypoint for CI and SDK sanity check.
    pub fn hello(env: Env, to: String) -> Vec<String> {
        vec![&env, String::from_str(&env, "ChronoPay"), to]
    }
}

mod test;
