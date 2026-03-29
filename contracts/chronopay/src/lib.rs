#![no_std]
//! ChronoPay time token contract.
//!
//! Implements a per-slot availability state machine with three states:
//!   Available → Sold → Redeemed
//!
//! Every slot is initialised to `Available` on creation.  Transitions are
//! strictly guarded; any attempt to move to an invalid next-state is rejected
//! with `Error::InvalidTransition`.  Missing slots return `Error::SlotNotFound`.

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, vec, Env, String, Symbol, Vec,
};

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// Contract-level errors returned by state-machine operations.
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    /// The requested slot id has never been created.
    SlotNotFound = 1,
    /// The requested state transition is not allowed from the current state.
    InvalidTransition = 2,
}

// ---------------------------------------------------------------------------
// State
// ---------------------------------------------------------------------------

/// Availability lifecycle of a time slot.
///
/// Valid transitions:
///   `Available` → `Sold`
///   `Sold`      → `Redeemed`
///
/// All other transitions are rejected.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SlotStatus {
    /// Slot has been created and is open for purchase.
    Available,
    /// Slot has been purchased; awaiting redemption.
    Sold,
    /// Slot has been redeemed; terminal state.
    Redeemed,
}

// ---------------------------------------------------------------------------
// Storage keys
// ---------------------------------------------------------------------------

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    /// Auto-incrementing sequence counter (instance storage).
    SlotSeq,
    /// Maps slot_id → owner string (persistent storage).
    SlotOwner(u32),
    /// Maps slot_id → SlotStatus (persistent storage).
    SlotStatus(u32),
}

// ---------------------------------------------------------------------------
// Contract
// ---------------------------------------------------------------------------

#[contract]
pub struct ChronoPayContract;

#[contractimpl]
impl ChronoPayContract {
    /// Create a time slot with an auto-incrementing id.
    ///
    /// Persists the professional's identity as the slot owner and initialises
    /// the slot status to `SlotStatus::Available`.
    ///
    /// Returns the newly assigned slot id (1-based).
    pub fn create_time_slot(env: Env, professional: String, start_time: u64, end_time: u64) -> u32 {
        // start_time / end_time are recorded off-chain via events in the full
        // implementation; suppressed here to keep the stub minimal.
        let _ = (start_time, end_time);

        let current_seq: u32 = env
            .storage()
            .instance()
            .get(&DataKey::SlotSeq)
            .unwrap_or(0u32);

        let id = current_seq.checked_add(1).expect("slot id overflow");

        env.storage().instance().set(&DataKey::SlotSeq, &id);

        // Persist owner and initial status.
        env.storage()
            .persistent()
            .set(&DataKey::SlotOwner(id), &professional);
        env.storage()
            .persistent()
            .set(&DataKey::SlotStatus(id), &SlotStatus::Available);

        id
    }

    /// Return the current availability status of a slot.
    ///
    /// # Errors
    /// - `Error::SlotNotFound` – slot id was never created.
    pub fn get_slot_status(env: Env, slot_id: u32) -> Result<SlotStatus, Error> {
        env.storage()
            .persistent()
            .get(&DataKey::SlotStatus(slot_id))
            .ok_or(Error::SlotNotFound)
    }

    /// Transition a slot from `Available` to `Sold`.
    ///
    /// # Errors
    /// - `Error::SlotNotFound`      – slot id was never created.
    /// - `Error::InvalidTransition` – slot is not currently `Available`.
    pub fn sell_slot(env: Env, slot_id: u32) -> Result<(), Error> {
        let status: SlotStatus = env
            .storage()
            .persistent()
            .get(&DataKey::SlotStatus(slot_id))
            .ok_or(Error::SlotNotFound)?;

        if status != SlotStatus::Available {
            return Err(Error::InvalidTransition);
        }

        env.storage()
            .persistent()
            .set(&DataKey::SlotStatus(slot_id), &SlotStatus::Sold);

        Ok(())
    }

    /// Transition a slot from `Sold` to `Redeemed`.
    ///
    /// # Errors
    /// - `Error::SlotNotFound`      – slot id was never created.
    /// - `Error::InvalidTransition` – slot is not currently `Sold`.
    pub fn redeem_slot(env: Env, slot_id: u32) -> Result<(), Error> {
        let status: SlotStatus = env
            .storage()
            .persistent()
            .get(&DataKey::SlotStatus(slot_id))
            .ok_or(Error::SlotNotFound)?;

        if status != SlotStatus::Sold {
            return Err(Error::InvalidTransition);
        }

        env.storage()
            .persistent()
            .set(&DataKey::SlotStatus(slot_id), &SlotStatus::Redeemed);

        Ok(())
    }

    /// Mint a time token for a slot (stub — returns a synthetic token symbol).
    pub fn mint_time_token(env: Env, slot_id: u32) -> Symbol {
        let _ = slot_id;
        Symbol::new(&env, "TIME_TOKEN")
    }

    /// Hello-style entrypoint for CI and SDK sanity check.
    pub fn hello(env: Env, to: String) -> Vec<String> {
        vec![&env, String::from_str(&env, "ChronoPay"), to]
    }
}

mod test;
