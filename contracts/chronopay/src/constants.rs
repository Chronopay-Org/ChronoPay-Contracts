//! Centralized contract constants for ChronoPay.
//!
//! All magic numbers, limits, durations, and protocol parameters live here
//! so they can be audited, tested, and updated in one place.

// ── Contract metadata ─────────────────────────────────────────────────────

/// Semantic version of the contract schema. Bump on breaking storage changes.
pub const CONTRACT_VERSION: u32 = 1;

/// Human-readable contract name used in hello/sanity endpoints.
pub const CONTRACT_NAME: &str = "ChronoPay";

// ── Slot ID sequencing ────────────────────────────────────────────────────

/// Initial value for the auto-incrementing slot ID counter.
pub const INITIAL_SLOT_SEQ: u32 = 0;

/// Maximum slot ID before overflow. Prevents silent wrapping.
pub const MAX_SLOT_ID: u32 = u32::MAX;

// ── Time constraints ──────────────────────────────────────────────────────

/// Minimum allowed slot duration in seconds (15 minutes).
/// Prevents accidentally creating zero- or trivially-short slots.
pub const MIN_SLOT_DURATION_SECS: u64 = 900;

/// Maximum allowed slot duration in seconds (30 days).
/// Prevents unbounded resource lock-up.
pub const MAX_SLOT_DURATION_SECS: u64 = 30 * 24 * 3600;

/// Maximum allowed scheduling horizon: slots cannot start more than
/// 365 days into the future. Prevents stale data accumulation.
pub const MAX_FUTURE_START_SECS: u64 = 365 * 24 * 3600;

// ── Settlement ────────────────────────────────────────────────────────────

/// Default settlement timeout in seconds (24 hours).
/// Buyer must settle within this window after booking.
pub const DEFAULT_SETTLEMENT_TIMEOUT_SECS: u64 = 86_400;

/// Minimum settlement timeout an admin can configure (1 hour).
pub const MIN_SETTLEMENT_TIMEOUT_SECS: u64 = 3_600;

/// Maximum settlement timeout an admin can configure (7 days).
pub const MAX_SETTLEMENT_TIMEOUT_SECS: u64 = 7 * 24 * 3600;

// ── Rate limits ───────────────────────────────────────────────────────────

/// Maximum number of slots a single professional can create.
/// Prevents storage exhaustion attacks.
pub const MAX_SLOTS_PER_PROFESSIONAL: u32 = 1_000;

// ── Storage TTL (ledger-based) ────────────────────────────────────────────

/// Minimum remaining ledgers before instance storage needs TTL extension (~7 days).
pub const INSTANCE_TTL_THRESHOLD: u32 = 120_960;

/// Target TTL extension for instance storage (~31 days).
pub const INSTANCE_TTL_EXTEND: u32 = 535_680;
