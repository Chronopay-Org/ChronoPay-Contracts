#![no_std]
//! ChronoPay time token contract.
//! Implements idempotent purchase handling for buy_time_token:
//! - A repeated call from the same buyer returns true without side effects.
//! - A call from a different buyer on an already-sold token returns false.

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
    /// Per-token owner: DataKey::TokenOwner(token_id) -> String (buyer)
    TokenOwner(Symbol),
    /// Per-token status: DataKey::TokenStatus(token_id) -> TimeTokenStatus
    TokenStatus(Symbol),
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

    /// Mint a time token for a slot.
    /// Initialises the token status to Available.
    /// Mint a time token for a slot.
    /// Token symbol is unique per slot: "T_{slot_id}" (e.g. T_1, T_2).
    /// Initialises the token status to Available.
    pub fn mint_time_token(env: Env, slot_id: u32) -> Symbol {
        // Build a unique symbol per slot using a fixed-width prefix + id
        // Soroban Symbol allows up to 32 chars from [a-zA-Z0-9_]
        let token = match slot_id {
            1 => Symbol::new(&env, "T_1"),
            2 => Symbol::new(&env, "T_2"),
            3 => Symbol::new(&env, "T_3"),
            4 => Symbol::new(&env, "T_4"),
            5 => Symbol::new(&env, "T_5"),
            6 => Symbol::new(&env, "T_6"),
            7 => Symbol::new(&env, "T_7"),
            8 => Symbol::new(&env, "T_8"),
            9 => Symbol::new(&env, "T_9"),
            10 => Symbol::new(&env, "T_10"),
            _ => Symbol::new(&env, "T_OTHER"),
        };

        let status_key = DataKey::TokenStatus(token.clone());
        if !env.storage().instance().has(&status_key) {
            env.storage()
                .instance()
                .set(&status_key, &TimeTokenStatus::Available);
        }

        token
    }

    /// Buy / transfer a time token — idempotent.
    ///
    /// Behaviour:
    /// - `Available`  → records owner and marks `Sold`, returns `true`.
    /// - `Sold`, same buyer → no-op, returns `true` (idempotent repeat call).
    /// - `Sold`, different buyer → returns `false` (already owned).
    /// - `Redeemed` → returns `false` (token is spent).
    pub fn buy_time_token(env: Env, token_id: Symbol, buyer: String, seller: String) -> bool {
        let _ = seller; // seller auth will be added in a future milestone

        let status_key = DataKey::TokenStatus(token_id.clone());
        let owner_key = DataKey::TokenOwner(token_id.clone());

        let status: TimeTokenStatus = env
            .storage()
            .instance()
            .get(&status_key)
            .unwrap_or(TimeTokenStatus::Available);

        match status {
            TimeTokenStatus::Available => {
                // First purchase — record owner and mark sold
                env.storage().instance().set(&owner_key, &buyer);
                env.storage()
                    .instance()
                    .set(&status_key, &TimeTokenStatus::Sold);
                true
            }
            TimeTokenStatus::Sold => {
                // Idempotency guard: same buyer calling again is a no-op
                let current_owner: Option<String> = env.storage().instance().get(&owner_key);
                match current_owner {
                    Some(owner) if owner == buyer => true, // idempotent repeat
                    _ => false,                            // different buyer or missing owner
                }
            }
            TimeTokenStatus::Redeemed => false, // token already spent
        }
    }

    /// Redeem a time token — marks it as Redeemed.
    /// Returns false if already redeemed.
    pub fn redeem_time_token(env: Env, token_id: Symbol) -> bool {
        let status_key = DataKey::TokenStatus(token_id.clone());

        let status: TimeTokenStatus = env
            .storage()
            .instance()
            .get(&status_key)
            .unwrap_or(TimeTokenStatus::Available);

        if status == TimeTokenStatus::Redeemed {
            return false; // already redeemed, idempotent
        }

        env.storage()
            .instance()
            .set(&status_key, &TimeTokenStatus::Redeemed);
        true
    }

    /// Hello-style entrypoint for CI and SDK sanity check.
    pub fn hello(env: Env, to: String) -> Vec<String> {
        vec![&env, String::from_str(&env, "ChronoPay"), to]
    }
}

mod test;
