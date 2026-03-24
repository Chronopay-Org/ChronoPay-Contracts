#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, vec, Address, Env, String, Symbol, Vec};

// ---------------------------------------------------------------------------
// Storage lifecycle constants (~24 h at 5 s/ledger = 17 280 ledgers)
// ---------------------------------------------------------------------------
const LEDGER_BUMP: u32 = 17_280;
const LEDGER_THRESHOLD: u32 = 16_000;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TimeTokenStatus {
    Available,
    Sold,
    Redeemed,
}

/// Immutable metadata written once at slot creation.
/// Stored under `DataKey::SlotMeta(id)`.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SlotMeta {
    pub professional: String,
    pub start_time: u64,
    pub end_time: u64,
}

/// Mutable state – changes on buy and redeem.
/// Stored under `DataKey::SlotState(id)` – intentionally small to minimise
/// per-mutation byte-cost charged by Soroban rent.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SlotState {
    pub owner: Option<Address>,
    pub status: TimeTokenStatus,
}

/// Combined view returned by `get_slot` for backwards-compatible inspection.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SlotData {
    pub professional: String,
    pub start_time: u64,
    pub end_time: u64,
    pub owner: Option<Address>,
    pub status: TimeTokenStatus,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    /// Auto-incrementing sequence counter (lives in Instance storage).
    SlotSeq,
    /// Immutable metadata for a slot.
    SlotMeta(u32),
    /// Mutable state for a slot.
    SlotState(u32),
}

// ---------------------------------------------------------------------------
// Contract
// ---------------------------------------------------------------------------

#[contract]
pub struct ChronoPayContract;

#[contractimpl]
impl ChronoPayContract {
    // -----------------------------------------------------------------------
    // Internal helpers
    // -----------------------------------------------------------------------

    /// Extend the TTL of a `Persistent` entry so it doesn't silently expire.
    fn bump_persistent(env: &Env, key: &DataKey) {
        env.storage()
            .persistent()
            .extend_ttl(key, LEDGER_THRESHOLD, LEDGER_BUMP);
    }

    /// Extend the Instance TTL (covers SlotSeq and any instance-scoped data).
    fn bump_instance(env: &Env) {
        env.storage()
            .instance()
            .extend_ttl(LEDGER_THRESHOLD, LEDGER_BUMP);
    }

    // -----------------------------------------------------------------------
    // Public entry points
    // -----------------------------------------------------------------------

    /// Create a time slot with an auto-incrementing slot id.
    /// Returns the newly assigned slot id.
    ///
    /// **Storage optimisation**: immutable metadata (`SlotMeta`) is written
    /// once; mutable state (`SlotState`) is a separate, smaller entry updated
    /// by buy/redeem without touching metadata bytes.
    pub fn create_time_slot(env: Env, professional: String, start_time: u64, end_time: u64) -> u32 {
        // Read + bump sequence counter from Instance storage.
        let current_seq: u32 = env
            .storage()
            .instance()
            .get(&DataKey::SlotSeq)
            .unwrap_or(0u32);

        let slot_id = current_seq
            .checked_add(1)
            .expect("slot id overflow");

        env.storage()
            .instance()
            .set(&DataKey::SlotSeq, &slot_id);

        // --- Write immutable metadata (paid once, never re-written) ---------
        let meta_key = DataKey::SlotMeta(slot_id);
        env.storage().persistent().set(
            &meta_key,
            &SlotMeta {
                professional,
                start_time,
                end_time,
            },
        );
        Self::bump_persistent(&env, &meta_key);

        // --- Write initial mutable state ------------------------------------
        let state_key = DataKey::SlotState(slot_id);
        env.storage().persistent().set(
            &state_key,
            &SlotState {
                owner: None,
                status: TimeTokenStatus::Available,
            },
        );
        Self::bump_persistent(&env, &state_key);

        // Bump instance so the counter doesn't expire either.
        Self::bump_instance(&env);

        slot_id
    }

    /// Mint a time token for a slot.
    /// Returns a Symbol representing the token (e.g. `TIME_TOKEN`).
    pub fn mint_time_token(env: Env, slot_id: u32) -> Symbol {
        if !env.storage().persistent().has(&DataKey::SlotMeta(slot_id)) {
            panic!("Slot does not exist");
        }
        Symbol::new(&env, "TIME_TOKEN")
    }

    /// Buy / transfer a time token.
    /// Only reads and writes `SlotState` – the smaller, hot entry.
    pub fn buy_time_token(env: Env, slot_id: u32, buyer: Address) -> bool {
        let state_key = DataKey::SlotState(slot_id);
        let mut state: SlotState = env
            .storage()
            .persistent()
            .get(&state_key)
            .expect("Slot not found");

        if state.status != TimeTokenStatus::Available {
            return false;
        }

        state.owner = Some(buyer);
        state.status = TimeTokenStatus::Sold;

        env.storage().persistent().set(&state_key, &state);
        Self::bump_persistent(&env, &state_key);
        true
    }

    /// Redeem a time token.
    /// Only reads and writes `SlotState`.
    pub fn redeem_time_token(env: Env, slot_id: u32) -> bool {
        let state_key = DataKey::SlotState(slot_id);
        let mut state: SlotState = env
            .storage()
            .persistent()
            .get(&state_key)
            .expect("Slot not found");

        if state.status != TimeTokenStatus::Sold {
            return false;
        }

        state.status = TimeTokenStatus::Redeemed;

        env.storage().persistent().set(&state_key, &state);
        Self::bump_persistent(&env, &state_key);
        true
    }

    /// Retrieve combined slot data for external inspection.
    /// Returns `None` if the slot does not exist.
    pub fn get_slot(env: Env, slot_id: u32) -> Option<SlotData> {
        let meta: SlotMeta = env
            .storage()
            .persistent()
            .get(&DataKey::SlotMeta(slot_id))?;
        let state: SlotState = env
            .storage()
            .persistent()
            .get(&DataKey::SlotState(slot_id))?;

        Some(SlotData {
            professional: meta.professional,
            start_time: meta.start_time,
            end_time: meta.end_time,
            owner: state.owner,
            status: state.status,
        })
    }

    /// Lightweight status-only query – avoids deserialising metadata bytes.
    ///
    /// Returns `None` if the slot does not exist.
    pub fn get_slot_status(env: Env, slot_id: u32) -> Option<TimeTokenStatus> {
        let state: SlotState = env
            .storage()
            .persistent()
            .get(&DataKey::SlotState(slot_id))?;
        Some(state.status)
    }

    /// Hello-style entry point for CI and SDK sanity checks.
    pub fn hello(env: Env, to: String) -> Vec<String> {
        vec![&env, String::from_str(&env, "ChronoPay"), to]
    }
}

mod test;
