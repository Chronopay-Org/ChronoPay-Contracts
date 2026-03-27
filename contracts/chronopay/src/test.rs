#![cfg(test)]

use super::*;
use crate::constants;
use soroban_sdk::{vec, Env, String};

fn setup(env: &Env) -> ChronoPayContractClient<'_> {
    env.mock_all_auths();
    let contract_id = env.register(ChronoPayContract, ());
    ChronoPayContractClient::new(env, &contract_id)
}

// ── Hello uses CONTRACT_NAME ──────────────────────────────────────────────

#[test]
fn test_hello() {
    let env = Env::default();
    let client = setup(&env);
    let words = client.hello(&String::from_str(&env, "Dev"));
    assert_eq!(
        words,
        vec![
            &env,
            String::from_str(&env, constants::CONTRACT_NAME),
            String::from_str(&env, "Dev"),
        ]
    );
}

// ── Slot ID sequencing uses INITIAL_SLOT_SEQ ──────────────────────────────

#[test]
fn test_create_time_slot_starts_from_initial_seq() {
    let env = Env::default();
    let client = setup(&env);
    let id = client.create_time_slot(&String::from_str(&env, "alice"), &1000u64, &2000u64);
    assert_eq!(id, constants::INITIAL_SLOT_SEQ + 1);
}

#[test]
fn test_create_time_slot_auto_increments() {
    let env = Env::default();
    let client = setup(&env);
    let id1 = client.create_time_slot(&String::from_str(&env, "alice"), &1000u64, &2000u64);
    let id2 = client.create_time_slot(&String::from_str(&env, "alice"), &3000u64, &4000u64);
    let id3 = client.create_time_slot(&String::from_str(&env, "alice"), &5000u64, &6000u64);
    assert_eq!(id1, 1);
    assert_eq!(id2, 2);
    assert_eq!(id3, 3);
}

// ── Contract version ──────────────────────────────────────────────────────

#[test]
fn test_version_returns_contract_version() {
    let env = Env::default();
    let client = setup(&env);
    assert_eq!(client.version(), constants::CONTRACT_VERSION);
}

#[test]
fn test_version_persisted_after_first_slot() {
    let env = Env::default();
    let client = setup(&env);
    client.create_time_slot(&String::from_str(&env, "alice"), &1000u64, &2000u64);
    assert_eq!(client.version(), constants::CONTRACT_VERSION);
}

// ── Constants module values ───────────────────────────────────────────────

#[test]
fn test_constant_contract_version_is_positive() {
    assert!(constants::CONTRACT_VERSION >= 1);
}

#[test]
fn test_constant_contract_name_not_empty() {
    assert!(!constants::CONTRACT_NAME.is_empty());
}

#[test]
fn test_constant_initial_slot_seq_is_zero() {
    assert_eq!(constants::INITIAL_SLOT_SEQ, 0);
}

#[test]
fn test_constant_min_slot_duration() {
    assert_eq!(constants::MIN_SLOT_DURATION_SECS, 900);
    assert!(constants::MIN_SLOT_DURATION_SECS > 0);
}

#[test]
fn test_constant_max_slot_duration() {
    assert_eq!(constants::MAX_SLOT_DURATION_SECS, 30 * 24 * 3600);
    assert!(constants::MAX_SLOT_DURATION_SECS > constants::MIN_SLOT_DURATION_SECS);
}

#[test]
fn test_constant_max_future_start() {
    assert_eq!(constants::MAX_FUTURE_START_SECS, 365 * 24 * 3600);
    assert!(constants::MAX_FUTURE_START_SECS > constants::MAX_SLOT_DURATION_SECS);
}

#[test]
fn test_constant_settlement_timeout_defaults() {
    assert_eq!(constants::DEFAULT_SETTLEMENT_TIMEOUT_SECS, 86_400);
    assert!(constants::DEFAULT_SETTLEMENT_TIMEOUT_SECS >= constants::MIN_SETTLEMENT_TIMEOUT_SECS);
    assert!(constants::DEFAULT_SETTLEMENT_TIMEOUT_SECS <= constants::MAX_SETTLEMENT_TIMEOUT_SECS);
}

#[test]
fn test_constant_settlement_timeout_bounds() {
    assert_eq!(constants::MIN_SETTLEMENT_TIMEOUT_SECS, 3_600);
    assert_eq!(constants::MAX_SETTLEMENT_TIMEOUT_SECS, 7 * 24 * 3600);
    assert!(constants::MIN_SETTLEMENT_TIMEOUT_SECS < constants::MAX_SETTLEMENT_TIMEOUT_SECS);
}

#[test]
fn test_constant_max_slots_per_professional() {
    assert_eq!(constants::MAX_SLOTS_PER_PROFESSIONAL, 1_000);
    assert!(constants::MAX_SLOTS_PER_PROFESSIONAL > 0);
}

#[test]
fn test_constant_instance_ttl_values() {
    assert!(constants::INSTANCE_TTL_EXTEND > constants::INSTANCE_TTL_THRESHOLD);
    assert!(constants::INSTANCE_TTL_THRESHOLD > 0);
}

// ── Legacy stubs ──────────────────────────────────────────────────────────

#[test]
fn test_mint_and_redeem() {
    let env = Env::default();
    let client = setup(&env);
    let slot_id = client.create_time_slot(&String::from_str(&env, "pro"), &1000u64, &2000u64);
    let token = client.mint_time_token(&slot_id);
    assert_eq!(token, soroban_sdk::Symbol::new(&env, "TIME_TOKEN"));
    assert!(client.redeem_time_token(&token));
}
