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
    CancelWindowSecs,
    SlotBuyer(u32),
    SlotStartTime(u32),
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

        let next_seq = current_seq
            .checked_add(1)
            .expect("slot id overflow");

        env.storage()
            .instance()
            .set(&DataKey::SlotSeq, &next_seq);

        env.storage()
            .instance()
            .set(&DataKey::SlotStartTime(next_seq), &start_time);

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

    /// Buy / reserve a specific time slot (new robust handler)
    pub fn buy_time_slot(env: Env, slot_id: u32, buyer: String) -> bool {
        env.storage()
            .instance()
            .set(&DataKey::SlotBuyer(slot_id), &buyer);
        env.storage()
            .instance()
            .set(&DataKey::Status, &TimeTokenStatus::Sold);
        true
    }

    /// Set cancellation window in seconds. Defaults to 3600 (1 hour).
    pub fn set_cancel_window(env: Env, seconds: u64) {
        if seconds == 0 {
            panic!("invalid_window");
        }
        env.storage()
            .instance()
            .set(&DataKey::CancelWindowSecs, &seconds);
    }

    /// Cancel a bought time slot. Must be within the cancellation window.
    pub fn cancel_time_slot(env: Env, slot_id: u32, _buyer: String) -> bool {
        // Find slot start time
        let start_time: u64 = env
            .storage()
            .instance()
            .get(&DataKey::SlotStartTime(slot_id))
            .unwrap_or_else(|| panic!("slot_not_found"));

        // Ensure slot was sold
        let _buyer_record: String = env
            .storage()
            .instance()
            .get(&DataKey::SlotBuyer(slot_id))
            .unwrap_or_else(|| panic!("slot_not_sold"));

        // Get window (default 3600 seconds)
        let window_secs: u64 = env
            .storage()
            .instance()
            .get(&DataKey::CancelWindowSecs)
            .unwrap_or(3600u64);

        // Get current ledger time
        let current_time = env.ledger().timestamp();

        // Safe subtraction: if start_time < window_secs, it's already too late.
        if start_time < window_secs || current_time >= start_time - window_secs {
            panic!("too_late_to_cancel");
        }

        // Cancel the slot
        env.storage()
            .instance()
            .remove(&DataKey::SlotBuyer(slot_id));
        env.storage()
            .instance()
            .set(&DataKey::Status, &TimeTokenStatus::Available);

        true
    }

    /// Redeem time token (stub). In full implementation: token_id, marks as redeemed.
    pub fn redeem_time_token(env: Env, token_id: Symbol) -> bool {
        let _ = token_id;
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
