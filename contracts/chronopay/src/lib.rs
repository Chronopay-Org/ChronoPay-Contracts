#![no_std]

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
pub struct SlotInfo {
    pub professional: String,
    pub start_time: u64,
    pub end_time: u64,
}

/// # Upgrade Safety
///
/// Soroban encodes variants by their discriminant index (ordinal position in
/// the enum). Reordering, removing, or changing payload types silently
/// corrupts existing storage after an upgrade. New variants must be appended
/// at the end.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    SlotSeq,         // u32  — global monotonic slot-ID counter
    Slot(u32),       // SlotInfo — creation metadata per slot
    SlotOwner(u32),  // String — current owner per slot
    SlotStatus(u32), // TimeTokenStatus — lifecycle status per slot
}

#[contract]
pub struct ChronoPayContract;

#[contractimpl]
impl ChronoPayContract {
    pub fn create_time_slot(env: Env, professional: String, start_time: u64, end_time: u64) -> u32 {
        assert!(
            end_time > start_time,
            "end_time must be greater than start_time"
        );

        let current_seq: u32 = env
            .storage()
            .instance()
            .get(&DataKey::SlotSeq)
            .unwrap_or(0u32);

        let next_seq = current_seq.checked_add(1).expect("slot id overflow");

        let info = SlotInfo {
            professional,
            start_time,
            end_time,
        };

        // Write per-slot entries before the global counter so a failure leaves
        // the sequencer unchanged and no orphaned slot metadata is reachable.
        env.storage()
            .instance()
            .set(&DataKey::Slot(next_seq), &info);
        env.storage()
            .instance()
            .set(&DataKey::SlotStatus(next_seq), &TimeTokenStatus::Available);
        env.storage().instance().set(&DataKey::SlotSeq, &next_seq);

        next_seq
    }

    pub fn mint_time_token(env: Env, slot_id: u32) -> Symbol {
        assert!(
            env.storage().instance().has(&DataKey::Slot(slot_id)),
            "slot not found"
        );
        Symbol::new(&env, "TIME_TOKEN")
    }

    pub fn buy_time_token(env: Env, slot_id: u32, buyer: String, seller: String) -> bool {
        let _ = seller; // reserved for future seller-authorisation logic

        assert!(
            env.storage().instance().has(&DataKey::Slot(slot_id)),
            "slot not found"
        );

        let status: TimeTokenStatus = env
            .storage()
            .instance()
            .get(&DataKey::SlotStatus(slot_id))
            .unwrap_or(TimeTokenStatus::Available);

        assert!(
            status == TimeTokenStatus::Available,
            "slot is not available for purchase"
        );

        env.storage()
            .instance()
            .set(&DataKey::SlotOwner(slot_id), &buyer);
        env.storage()
            .instance()
            .set(&DataKey::SlotStatus(slot_id), &TimeTokenStatus::Sold);

        true
    }

    pub fn redeem_time_token(env: Env, slot_id: u32) -> bool {
        assert!(
            env.storage().instance().has(&DataKey::Slot(slot_id)),
            "slot not found"
        );

        let status: TimeTokenStatus = env
            .storage()
            .instance()
            .get(&DataKey::SlotStatus(slot_id))
            .unwrap_or(TimeTokenStatus::Available);

        assert!(
            status == TimeTokenStatus::Sold,
            "slot must be in Sold status to redeem"
        );

        env.storage()
            .instance()
            .set(&DataKey::SlotStatus(slot_id), &TimeTokenStatus::Redeemed);

        true
    }

    pub fn get_slot_info(env: Env, slot_id: u32) -> SlotInfo {
        env.storage()
            .instance()
            .get(&DataKey::Slot(slot_id))
            .expect("slot not found")
    }

    pub fn get_slot_owner(env: Env, slot_id: u32) -> String {
        assert!(
            env.storage().instance().has(&DataKey::Slot(slot_id)),
            "slot not found"
        );
        // Returns empty string if the slot has never been purchased.
        env.storage()
            .instance()
            .get(&DataKey::SlotOwner(slot_id))
            .unwrap_or_else(|| String::from_str(&env, ""))
    }

    pub fn get_slot_status(env: Env, slot_id: u32) -> TimeTokenStatus {
        assert!(
            env.storage().instance().has(&DataKey::Slot(slot_id)),
            "slot not found"
        );
        env.storage()
            .instance()
            .get(&DataKey::SlotStatus(slot_id))
            .unwrap_or(TimeTokenStatus::Available)
    }

    pub fn hello(env: Env, to: String) -> Vec<String> {
        vec![&env, String::from_str(&env, "ChronoPay"), to]
    }
}

mod test;
