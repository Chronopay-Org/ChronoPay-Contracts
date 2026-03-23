#![no_std]
//! ChronoPay time token contract — scheduling and time tokenization.

use soroban_sdk::{
    contract, contractimpl, contracttype, vec, Address, Env, String, Symbol, Vec,
};

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
    Slot(u32),
    ProfessionalSlots(Address),
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TimeSlot {
    pub professional: Address,
    pub start_time: u64,
    pub end_time: u64,
    pub token: Option<Symbol>,
}

#[contract]
pub struct ChronoPayContract;

#[contractimpl]
impl ChronoPayContract {
    /// Create a time slot and persist it.
    /// Fails if end_time is not after start_time.
    pub fn create_time_slot(env: Env, professional: Address, start_time: u64, end_time: u64) -> u32 {
        professional.require_auth();

        if start_time >= end_time {
            panic!("end_time must be after start_time");
        }

        let slot_id = next_sequence(&env, DataKey::SlotSeq);
        let slot = TimeSlot {
            professional: professional.clone(),
            start_time,
            end_time,
            token: None,
        };

        env.storage().persistent().set(&DataKey::Slot(slot_id), &slot);

        // Update professional slots index
        let mut prof_slots: Vec<u32> = env
            .storage()
            .persistent()
            .get(&DataKey::ProfessionalSlots(professional.clone()))
            .unwrap_or(vec![&env]);
        prof_slots.push_back(slot_id);
        env.storage()
            .persistent()
            .set(&DataKey::ProfessionalSlots(professional.clone()), &prof_slots);

        env.events().publish(
            (Symbol::new(&env, "slot_created"), professional),
            slot_id
        );

        slot_id
    }

    /// Query a time slot by its ID.
    pub fn get_time_slot(env: Env, slot_id: u32) -> Option<TimeSlot> {
        env.storage().persistent().get(&DataKey::Slot(slot_id))
    }

    /// Query all slots for a specific professional.
    /// Returns a list of slot IDs.
    pub fn get_professional_slots(env: Env, professional: Address) -> Vec<u32> {
        env.storage()
            .persistent()
            .get(&DataKey::ProfessionalSlots(professional))
            .unwrap_or(vec![&env])
    }

    /// Mint a time token for a slot (stub).
    pub fn mint_time_token(env: Env, slot_id: u32) -> Symbol {
        let _ = slot_id;
        Symbol::new(&env, "TIME_TOKEN")
    }

    /// Buy / transfer time token (stub).
    pub fn buy_time_token(env: Env, token_id: Symbol, buyer: Address, seller: Address) -> bool {
        let _ = (token_id, buyer, seller);
        true
    }

    /// Redeem time token (stub).
    pub fn redeem_time_token(env: Env, token_id: Symbol) -> bool {
        let _ = token_id;
        true
    }

    /// Hello-style entrypoint for CI and SDK sanity check.
    pub fn hello(env: Env, to: String) -> Vec<String> {
        vec![&env, String::from_str(&env, "ChronoPay"), to]
    }
}

fn next_sequence(env: &Env, key: DataKey) -> u32 {
    let next = env
        .storage()
        .instance()
        .get(&key)
        .unwrap_or(0u32)
        .saturating_add(1);
    env.storage().instance().set(&key, &next);
    next
}

mod test;
