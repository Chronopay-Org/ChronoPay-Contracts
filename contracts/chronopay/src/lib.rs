#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, vec, Address, Env, String,
    Symbol, Vec,
};

// ── Time Slot Status ──────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TimeSlotStatus {
    Available,
    Booked,
    Cancelled,
}

// ── Time Slot ─────────────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TimeSlot {
    pub id: u32,
    pub professional: Address,
    pub start_time: u64,
    pub end_time: u64,
    pub status: TimeSlotStatus,
    pub created_at: u64,
}

// ── Time Token Status (kept for downstream use) ──────────────────────────

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TimeTokenStatus {
    Available,
    Sold,
    Redeemed,
}

// ── Storage Keys ──────────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    SlotSeq,
    Slot(u32),
    ProfessionalSlots(Address),
}

// ── Errors ────────────────────────────────────────────────────────────────

#[contracterror]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum SlotError {
    InvalidTimeRange = 1,
    SlotNotFound = 2,
    NotSlotOwner = 3,
    SlotNotAvailable = 4,
    AlreadyCancelled = 5,
}

// ── Contract ──────────────────────────────────────────────────────────────

#[contract]
pub struct ChronoPayContract;

#[contractimpl]
impl ChronoPayContract {
    /// Create a time slot owned by `professional`.
    ///
    /// Validates that `start_time < end_time`, authenticates the professional,
    /// and persists the slot in storage. Returns the auto-incremented slot ID.
    pub fn create_time_slot(
        env: Env,
        professional: Address,
        start_time: u64,
        end_time: u64,
    ) -> Result<u32, SlotError> {
        professional.require_auth();

        if start_time >= end_time {
            return Err(SlotError::InvalidTimeRange);
        }

        let slot_id = Self::next_slot_id(&env);

        let slot = TimeSlot {
            id: slot_id,
            professional: professional.clone(),
            start_time,
            end_time,
            status: TimeSlotStatus::Available,
            created_at: env.ledger().timestamp(),
        };

        env.storage()
            .persistent()
            .set(&DataKey::Slot(slot_id), &slot);

        let prof_key = DataKey::ProfessionalSlots(professional.clone());
        let mut ids: Vec<u32> = env
            .storage()
            .persistent()
            .get(&prof_key)
            .unwrap_or(Vec::new(&env));
        ids.push_back(slot_id);
        env.storage().persistent().set(&prof_key, &ids);

        env.events().publish(
            (symbol_short!("slot"), symbol_short!("created")),
            (slot_id, professional, start_time, end_time),
        );

        Ok(slot_id)
    }

    /// Retrieve a time slot by ID.
    pub fn get_time_slot(env: Env, slot_id: u32) -> Result<TimeSlot, SlotError> {
        env.storage()
            .persistent()
            .get(&DataKey::Slot(slot_id))
            .ok_or(SlotError::SlotNotFound)
    }

    /// Cancel a time slot. Only the professional who owns it can cancel.
    /// Only `Available` slots can be cancelled.
    pub fn cancel_time_slot(
        env: Env,
        professional: Address,
        slot_id: u32,
    ) -> Result<(), SlotError> {
        professional.require_auth();

        let mut slot: TimeSlot = env
            .storage()
            .persistent()
            .get(&DataKey::Slot(slot_id))
            .ok_or(SlotError::SlotNotFound)?;

        if slot.professional != professional {
            return Err(SlotError::NotSlotOwner);
        }

        if slot.status == TimeSlotStatus::Cancelled {
            return Err(SlotError::AlreadyCancelled);
        }

        if slot.status != TimeSlotStatus::Available {
            return Err(SlotError::SlotNotAvailable);
        }

        slot.status = TimeSlotStatus::Cancelled;
        env.storage()
            .persistent()
            .set(&DataKey::Slot(slot_id), &slot);

        env.events().publish(
            (symbol_short!("slot"), symbol_short!("cancel")),
            (slot_id, professional),
        );

        Ok(())
    }

    /// List all slot IDs belonging to a professional.
    pub fn get_slots_by_professional(env: Env, professional: Address) -> Vec<u32> {
        let prof_key = DataKey::ProfessionalSlots(professional);
        env.storage()
            .persistent()
            .get(&prof_key)
            .unwrap_or(Vec::new(&env))
    }

    /// Return the total number of time slots ever created.
    pub fn get_slot_count(env: Env) -> u32 {
        env.storage()
            .instance()
            .get(&DataKey::SlotSeq)
            .unwrap_or(0u32)
    }

    /// Mark a slot as booked. Intended for internal use by the token-minting flow.
    pub fn book_time_slot(env: Env, professional: Address, slot_id: u32) -> Result<(), SlotError> {
        professional.require_auth();

        let mut slot: TimeSlot = env
            .storage()
            .persistent()
            .get(&DataKey::Slot(slot_id))
            .ok_or(SlotError::SlotNotFound)?;

        if slot.professional != professional {
            return Err(SlotError::NotSlotOwner);
        }

        if slot.status != TimeSlotStatus::Available {
            return Err(SlotError::SlotNotAvailable);
        }

        slot.status = TimeSlotStatus::Booked;
        env.storage()
            .persistent()
            .set(&DataKey::Slot(slot_id), &slot);

        env.events().publish(
            (symbol_short!("slot"), symbol_short!("booked")),
            (slot_id, professional),
        );

        Ok(())
    }

    // ── Legacy / downstream stubs ─────────────────────────────────────

    /// Mint a time token for a slot (stub for SC-002).
    pub fn mint_time_token(env: Env, slot_id: u32) -> Symbol {
        let _ = slot_id;
        Symbol::new(&env, "TIME_TOKEN")
    }

    /// Buy / transfer time token (stub for SC-003).
    pub fn buy_time_token(env: Env, token_id: Symbol, buyer: String, seller: String) -> bool {
        let _ = (token_id, buyer, seller);
        true
    }

    /// Redeem time token (stub for SC-004).
    pub fn redeem_time_token(env: Env, token_id: Symbol) -> bool {
        let _ = token_id;
        true
    }

    /// Hello-style entrypoint for CI and SDK sanity check.
    pub fn hello(env: Env, to: String) -> Vec<String> {
        vec![&env, String::from_str(&env, "ChronoPay"), to]
    }

    // ── Internal ──────────────────────────────────────────────────────

    fn next_slot_id(env: &Env) -> u32 {
        let current: u32 = env
            .storage()
            .instance()
            .get(&DataKey::SlotSeq)
            .unwrap_or(0u32);
        let next = current.checked_add(1).expect("slot id overflow");
        env.storage().instance().set(&DataKey::SlotSeq, &next);
        next
    }
}

mod test;
