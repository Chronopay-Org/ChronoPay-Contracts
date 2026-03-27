#![no_std]
//! ChronoPay time-token contract with settlement timeout handling.
//!
//! Lifecycle: Available → Booked (settlement deadline set) → Settled | TimedOut
//!
//! When a slot is booked a settlement deadline is recorded. The buyer must
//! call `settle_slot` before the deadline. After the deadline anyone can call
//! `timeout_slot` to reclaim the slot for the professional.

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, vec, Address, Env, String,
    Symbol, Vec,
};

// ── Enums ─────────────────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SlotStatus {
    Available,
    Booked,
    Settled,
    TimedOut,
    Cancelled,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TimeTokenStatus {
    Available,
    Sold,
    Redeemed,
}

// ── Structs ───────────────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TimeSlot {
    pub id: u32,
    pub professional: Address,
    pub start_time: u64,
    pub end_time: u64,
    pub status: SlotStatus,
    pub created_at: u64,
}

/// Tracks the settlement obligation for a booked slot.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Settlement {
    pub slot_id: u32,
    pub buyer: Address,
    pub booked_at: u64,
    pub deadline: u64,
}

// ── Storage Keys ──────────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    SlotSeq,
    Slot(u32),
    Settlement(u32),
    /// Default settlement timeout duration (seconds). Set via `set_timeout`.
    TimeoutDuration,
    /// Admin address (set once via `init`).
    Admin,
}

/// Default settlement timeout: 24 hours.
const DEFAULT_TIMEOUT_SECS: u64 = 86_400;

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
    SettlementNotFound = 6,
    NotBuyer = 7,
    DeadlineNotReached = 8,
    DeadlineExpired = 9,
    AlreadySettled = 10,
    NotAdmin = 11,
    AlreadyInitialized = 12,
    NotInitialized = 13,
    ZeroTimeout = 14,
}

// ── Contract ──────────────────────────────────────────────────────────────

#[contract]
pub struct ChronoPayContract;

#[contractimpl]
impl ChronoPayContract {
    /// One-time admin initialization. Must be called before `set_timeout`.
    pub fn init(env: Env, admin: Address) -> Result<(), SlotError> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(SlotError::AlreadyInitialized);
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage()
            .instance()
            .set(&DataKey::TimeoutDuration, &DEFAULT_TIMEOUT_SECS);
        Ok(())
    }

    /// Admin-only: configure the settlement timeout duration (in seconds).
    pub fn set_timeout(env: Env, admin: Address, duration_secs: u64) -> Result<(), SlotError> {
        admin.require_auth();
        let stored_admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(SlotError::NotInitialized)?;
        if admin != stored_admin {
            return Err(SlotError::NotAdmin);
        }
        if duration_secs == 0 {
            return Err(SlotError::ZeroTimeout);
        }
        env.storage()
            .instance()
            .set(&DataKey::TimeoutDuration, &duration_secs);
        Ok(())
    }

    /// Read the current settlement timeout duration.
    pub fn get_timeout(env: Env) -> u64 {
        env.storage()
            .instance()
            .get(&DataKey::TimeoutDuration)
            .unwrap_or(DEFAULT_TIMEOUT_SECS)
    }

    /// Create a time slot owned by `professional`.
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
            status: SlotStatus::Available,
            created_at: env.ledger().timestamp(),
        };

        env.storage()
            .persistent()
            .set(&DataKey::Slot(slot_id), &slot);

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

    /// Book a time slot. Creates a settlement record with a deadline.
    /// Only `Available` slots can be booked.
    pub fn book_slot(env: Env, buyer: Address, slot_id: u32) -> Result<Settlement, SlotError> {
        buyer.require_auth();

        let mut slot: TimeSlot = env
            .storage()
            .persistent()
            .get(&DataKey::Slot(slot_id))
            .ok_or(SlotError::SlotNotFound)?;

        if slot.status != SlotStatus::Available {
            return Err(SlotError::SlotNotAvailable);
        }

        slot.status = SlotStatus::Booked;
        env.storage()
            .persistent()
            .set(&DataKey::Slot(slot_id), &slot);

        let timeout_dur: u64 = env
            .storage()
            .instance()
            .get(&DataKey::TimeoutDuration)
            .unwrap_or(DEFAULT_TIMEOUT_SECS);

        let now = env.ledger().timestamp();
        let settlement = Settlement {
            slot_id,
            buyer: buyer.clone(),
            booked_at: now,
            deadline: now + timeout_dur,
        };

        env.storage()
            .persistent()
            .set(&DataKey::Settlement(slot_id), &settlement);

        env.events().publish(
            (symbol_short!("slot"), symbol_short!("booked")),
            (slot_id, buyer, settlement.deadline),
        );

        Ok(settlement)
    }

    /// Retrieve the settlement record for a slot.
    pub fn get_settlement(env: Env, slot_id: u32) -> Result<Settlement, SlotError> {
        env.storage()
            .persistent()
            .get(&DataKey::Settlement(slot_id))
            .ok_or(SlotError::SettlementNotFound)
    }

    /// Settle a booked slot before the deadline. Only the buyer can settle.
    pub fn settle_slot(env: Env, buyer: Address, slot_id: u32) -> Result<(), SlotError> {
        buyer.require_auth();

        let mut slot: TimeSlot = env
            .storage()
            .persistent()
            .get(&DataKey::Slot(slot_id))
            .ok_or(SlotError::SlotNotFound)?;

        if slot.status != SlotStatus::Booked {
            return Err(SlotError::AlreadySettled);
        }

        let settlement: Settlement = env
            .storage()
            .persistent()
            .get(&DataKey::Settlement(slot_id))
            .ok_or(SlotError::SettlementNotFound)?;

        if settlement.buyer != buyer {
            return Err(SlotError::NotBuyer);
        }

        let now = env.ledger().timestamp();
        if now > settlement.deadline {
            return Err(SlotError::DeadlineExpired);
        }

        slot.status = SlotStatus::Settled;
        env.storage()
            .persistent()
            .set(&DataKey::Slot(slot_id), &slot);

        env.events().publish(
            (symbol_short!("slot"), symbol_short!("settled")),
            (slot_id, buyer),
        );

        Ok(())
    }

    /// Mark a booked slot as timed out after the settlement deadline.
    /// Anyone can call this — it's a permissionless cleanup operation.
    /// The slot reverts to `TimedOut` so the professional can reclaim it.
    pub fn timeout_slot(env: Env, slot_id: u32) -> Result<(), SlotError> {
        let mut slot: TimeSlot = env
            .storage()
            .persistent()
            .get(&DataKey::Slot(slot_id))
            .ok_or(SlotError::SlotNotFound)?;

        if slot.status != SlotStatus::Booked {
            return Err(SlotError::AlreadySettled);
        }

        let settlement: Settlement = env
            .storage()
            .persistent()
            .get(&DataKey::Settlement(slot_id))
            .ok_or(SlotError::SettlementNotFound)?;

        let now = env.ledger().timestamp();
        if now <= settlement.deadline {
            return Err(SlotError::DeadlineNotReached);
        }

        slot.status = SlotStatus::TimedOut;
        env.storage()
            .persistent()
            .set(&DataKey::Slot(slot_id), &slot);

        env.events().publish(
            (symbol_short!("slot"), symbol_short!("timeout")),
            (slot_id, settlement.buyer),
        );

        Ok(())
    }

    /// Cancel an `Available` slot. Only the professional can cancel.
    pub fn cancel_slot(env: Env, professional: Address, slot_id: u32) -> Result<(), SlotError> {
        professional.require_auth();

        let mut slot: TimeSlot = env
            .storage()
            .persistent()
            .get(&DataKey::Slot(slot_id))
            .ok_or(SlotError::SlotNotFound)?;

        if slot.professional != professional {
            return Err(SlotError::NotSlotOwner);
        }
        if slot.status == SlotStatus::Cancelled {
            return Err(SlotError::AlreadyCancelled);
        }
        if slot.status != SlotStatus::Available {
            return Err(SlotError::SlotNotAvailable);
        }

        slot.status = SlotStatus::Cancelled;
        env.storage()
            .persistent()
            .set(&DataKey::Slot(slot_id), &slot);

        env.events().publish(
            (symbol_short!("slot"), symbol_short!("cancel")),
            (slot_id, professional),
        );

        Ok(())
    }

    /// Return the total number of slots ever created.
    pub fn get_slot_count(env: Env) -> u32 {
        env.storage()
            .instance()
            .get(&DataKey::SlotSeq)
            .unwrap_or(0u32)
    }

    // ── Downstream stubs (to be implemented in later SCs) ─────────────

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
