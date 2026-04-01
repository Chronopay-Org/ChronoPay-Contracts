#![cfg(test)]
//! Negative + positive tests for the ChronoPay contract.
//!
//! ## Coverage matrix
//!
//! | Entry point        | Positive path | Unauth caller | Wrong role | Bad state |
//! |--------------------|:---:|:---:|:---:|:---:|
//! | `initialize`       | ✓ | ✓ | — | ✓ (re-init) |
//! | `create_time_slot` | ✓ | ✓ | — | ✓ (bad range) |
//! | `mint_time_token`  | ✓ | ✓ | ✓ (non-admin) | — |
//! | `buy_time_token`   | ✓ | ✓ | — | ✓ (unminted/sold) |
//! | `redeem_time_token`| ✓ | ✓ | ✓ (non-owner) | ✓ (not sold) |
//! | `hello`            | ✓ | — | — | — |

use super::*;
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{vec, Address, Env, String, Symbol};

fn setup() -> Env {
    Env::default()
}

fn setup(env: &Env) -> ChronoPayContractClient<'_> {
    env.mock_all_auths();
    let contract_id = env.register(ChronoPayContract, ());
    ChronoPayContractClient::new(env, &contract_id)
}

// ── Hello uses CONTRACT_NAME ──────────────────────────────────────────────

#[test]
fn test_hello() {
    let env = setup();
    let contract_id = env.register(ChronoPayContract, ());
    let client = ChronoPayContractClient::new(&env, &contract_id);
    (env, contract_id, client)
}

/// Setup with an already-initialised admin.
fn setup_with_admin() -> (Env, ChronoPayContractClient<'static>, Address) {
    let (env, _cid, client) = setup();
    let admin = Address::generate(&env);
    client.initialize(&admin);
    (env, client, admin)
}

/// Full setup: admin initialised, slot created, token minted and bought.
fn setup_full_lifecycle() -> (
    Env,
    ChronoPayContractClient<'static>,
    Address, // admin
    Address, // professional
    Address, // buyer
    u32,     // slot_id
) {
    let (env, client, admin) = setup_with_admin();
    let professional = Address::generate(&env);
    let buyer = Address::generate(&env);

    let slot_id = client.create_time_slot(&professional, &1000u64, &2000u64);
    client.mint_time_token(&admin, &slot_id);
    client.buy_time_token(&buyer, &slot_id);

    (env, client, admin, professional, buyer, slot_id)
}

// ===========================================================================
// 1. `hello` — sanity (no auth required)
// ===========================================================================

#[test]
fn test_hello() {
    let (env, _cid, client) = setup();

// ── hello ─────────────────────────────────────────────────────────────────────

#[test]
fn test_hello() {
    let (env, client) = setup();
    let words = client.hello(&String::from_str(&env, "Dev"));
    assert_eq!(
        words,
        vec![&env, String::from_str(&env, "ChronoPay"), String::from_str(&env, "Dev"),]
    );
}

// ── create_time_slot: happy paths ─────────────────────────────────────────────

#[test]
fn test_initialize_and_metadata() {
    let env = setup();
    let contract_id = env.register(ChronoPayContract, ());
    let client = ChronoPayContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let name = String::from_str(&env, "ChronoPay Time Tokens");
    let symbol = String::from_str(&env, "TIME");

    client.initialize(&admin, &name, &symbol);

    let metadata = client
        .get_collection_metadata()
        .expect("metadata should exist");
    assert_eq!(metadata.name, name);
    assert_eq!(metadata.symbol, symbol);
}

#[test]
#[should_panic(expected = "already initialized")]
fn test_initialize_twice_panics() {
    let env = setup();
    let contract_id = env.register(ChronoPayContract, ());
    let client = ChronoPayContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let name = String::from_str(&env, "Name");
    let symbol = String::from_str(&env, "SYM");

    client.initialize(&admin, &name, &symbol);
    client.initialize(&admin, &name, &symbol);
}

#[test]
fn test_create_time_slot_persists() {
    let env = setup();
    env.mock_all_auths();
    let contract_id = env.register(ChronoPayContract, ());
    let client = ChronoPayContractClient::new(&env, &contract_id);

    let professional = Address::generate(&env);
    let slot_id = client.create_time_slot(&professional, &1_000u64, &2_000u64);
    assert_eq!(slot_id, 1);

    let slot = client.get_time_slot(&slot_id).expect("slot should exist");
    assert_eq!(slot.professional, professional);
    assert_eq!(slot.start_time, 1_000u64);
    assert_eq!(slot.end_time, 2_000u64);
    assert!(slot.token.is_none());
}

#[test]
fn test_version_default_before_any_call() {
    let env = Env::default();
    let contract_id = env.register(ChronoPayContract, ());
    let client = ChronoPayContractClient::new(&env, &contract_id);

    let v = client.get_contract_version();
    assert_eq!(v, 0);
}

#[test]
fn test_version_initialized_on_first_call() {
    let env = Env::default();
    let contract_id = env.register(ChronoPayContract, ());
    let client = ChronoPayContractClient::new(&env, &contract_id);

    let _ = client.hello(&String::from_str(&env, "X"));
    let v = client.get_contract_version();
    assert_eq!(v, 1);
}

#[test]
fn test_version_stable_across_calls() {
    let env = Env::default();
    let contract_id = env.register(ChronoPayContract, ());
    let client = ChronoPayContractClient::new(&env, &contract_id);

    let _ = client.create_time_slot(&String::from_str(&env, "pro"), &1u64, &2u64);
    let _ = client.hello(&String::from_str(&env, "Y"));
    let _ = client.mint_time_token(&1u32);

    let v = client.get_contract_version();
    assert_eq!(v, 1);
}

#[test]
fn test_create_time_slot_auto_increments() {
    let env = setup();
    env.mock_all_auths();
    let contract_id = env.register(ChronoPayContract, ());
    let client = ChronoPayContractClient::new(&env, &contract_id);

    let prof = Address::generate(&env);

    let slot_id_1 = client.create_time_slot(&prof, &1000u64, &2000u64);
    let slot_id_2 = client.create_time_slot(&prof, &3000u64, &4000u64);
    let slot_id_3 = client.create_time_slot(&prof, &5000u64, &6000u64);

    assert_eq!(slot_id_1, 1);
    assert_eq!(slot_id_2, 2);

    // Query existing slot (From feature/sc-038 logic)
    let slot = client.get_time_slot(&slot_id_1).expect("slot should exist");
    assert_eq!(slot.professional, professional);
    assert_eq!(slot.start_time, 1000u64);
    assert_eq!(slot.end_time, 2000u64);
    assert!(slot.token.is_none());

    // Query non-existent slot
    let non_existent = client.get_time_slot(&999u32);
    assert!(non_existent.is_none());
}

#[test]
#[should_panic(expected = "end_time must be after start_time")]
fn test_create_time_slot_rejects_invalid_times() {
    let env = setup();
    env.mock_all_auths();
    let contract_id = env.register(ChronoPayContract, ());
    let client = ChronoPayContractClient::new(&env, &contract_id);
    let professional = Address::generate(&env);
    let _ = client.create_time_slot(&professional, &10u64, &10u64);
}

#[test]
fn test_mint_buy_redeem_lifecycle() {
    let env = setup();
    env.mock_all_auths();
    let contract_id = env.register(ChronoPayContract, ());
    let client = ChronoPayContractClient::new(&env, &contract_id);

    let professional = Address::generate(&env);
    let slot_id = client.create_time_slot(&professional, &100u64, &200u64);

    let token_metadata = TokenMetadata {
        name: String::from_str(&env, "Consultation #1"),
        description: String::from_str(&env, "Expert consultation"),
        image_uri: String::from_str(&env, "ipfs://hash"),
    };

    let token = client.mint_time_token(&slot_id, &token_metadata);
    assert_eq!(token, Symbol::new(&env, "TIME_1"));

    // metadata after mint
    let metadata = client
        .get_token_metadata(&token)
        .expect("metadata should exist");
    assert_eq!(metadata.slot_id, slot_id);
    assert_eq!(metadata.status, TimeTokenStatus::Available);
    assert_eq!(metadata.current_owner, professional);
    assert_eq!(metadata.metadata.name, token_metadata.name);

    // buy / transfer
    let buyer = Address::generate(&env);
    let purchased = client.buy_time_token(&token, &buyer);
    assert!(purchased);

    let metadata_after_buy = client.get_token_metadata(&token).unwrap();
    assert_eq!(metadata_after_buy.status, TimeTokenStatus::Sold);
    assert_eq!(metadata_after_buy.current_owner, buyer);

    // redeem
    let redeemed = client.redeem_time_token(&token);
    assert!(redeemed);
    let metadata_after_redeem = client.get_token_metadata(&token).unwrap();
    assert_eq!(metadata_after_redeem.status, TimeTokenStatus::Redeemed);
}

#[test]
#[should_panic(expected = "token already minted for slot")]
fn test_mint_twice_panics() {
    let env = setup();
    env.mock_all_auths();
    let contract_id = env.register(ChronoPayContract, ());
    let client = ChronoPayContractClient::new(&env, &contract_id);

    let professional = Address::generate(&env);
    let slot_id = client.create_time_slot(&professional, &10u64, &20u64);

    let token_metadata = TokenMetadata {
        name: String::from_str(&env, "T"),
        description: String::from_str(&env, "D"),
        image_uri: String::from_str(&env, "I"),
    };

    let _ = client.mint_time_token(&slot_id, &token_metadata);
    let _ = client.mint_time_token(&slot_id, &token_metadata);
}

#[test]
#[should_panic(expected = "token already redeemed")]
fn test_buy_redeemed_panics() {
    let env = setup();
    env.mock_all_auths();
    let contract_id = env.register(ChronoPayContract, ());
    let client = ChronoPayContractClient::new(&env, &contract_id);

    let professional = Address::generate(&env);
    let slot_id = client.create_time_slot(&professional, &10u64, &20u64);

    let token_metadata = TokenMetadata {
        name: String::from_str(&env, "T"),
        description: String::from_str(&env, "D"),
        image_uri: String::from_str(&env, "I"),
    };

    let token = client.mint_time_token(&slot_id, &token_metadata);
    let buyer = Address::generate(&env);
    let _ = client.buy_time_token(&token, &buyer);
    let _ = client.redeem_time_token(&token);

    // Buying again after redemption should fail
    let buyer2 = Address::generate(&env);
    let _ = client.buy_time_token(&token, &buyer2);
}

#[test]
#[should_panic(expected = "token already redeemed")]
fn test_redeem_twice_panics() {
    let env = setup();
    env.mock_all_auths();
    let contract_id = env.register(ChronoPayContract, ());
    let client = ChronoPayContractClient::new(&env, &contract_id);

    let professional = Address::generate(&env);
    let slot_id = client.create_time_slot(&professional, &10u64, &20u64);

    let token_metadata = TokenMetadata {
        name: String::from_str(&env, "T"),
        description: String::from_str(&env, "D"),
        image_uri: String::from_str(&env, "I"),
    };

    let token = client.mint_time_token(&slot_id, &token_metadata);
    let _ = client.redeem_time_token(&token);
    let _ = client.redeem_time_token(&token);
}

#[test]
#[should_panic(expected = "buyer is already the owner")]
fn test_buy_requires_distinct_parties() {
    let env = setup();
    env.mock_all_auths();
    let contract_id = env.register(ChronoPayContract, ());
    let client = ChronoPayContractClient::new(&env, &contract_id);

    let professional = Address::generate(&env);
    let slot_id = client.create_time_slot(&professional, &10u64, &20u64);

    let token_metadata = TokenMetadata {
        name: String::from_str(&env, "T"),
        description: String::from_str(&env, "D"),
        image_uri: String::from_str(&env, "I"),
    };

    let token = client.mint_time_token(&slot_id, &token_metadata);
    let _ = client.buy_time_token(&token, &professional);
}

#[test]
#[should_panic(expected = "Error(Contract, #5)")]
fn test_create_time_slot_start_time_in_past() {
    let env = Env::default();
    // Do NOT mock auths — real auth enforcement.
    let contract_id = env.register(ChronoPayContract, ());
    let client = ChronoPayContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    client.init(&admin);
    let result = client.try_init(&admin);
    assert_eq!(result, Err(Ok(SlotError::AlreadyInitialized)));
}

// ── set_timeout ───────────────────────────────────────────────────────────

#[test]
fn test_set_timeout_success() {
    let env = Env::default();
    let (admin, _, client) = setup(&env);
    client.set_timeout(&admin, &3600);
    assert_eq!(client.get_timeout(), 3600);
}

#[test]
fn test_set_timeout_wrong_admin() {
    let env = Env::default();
    let (_, _, client) = setup(&env);
    let imposter = Address::generate(&env);
    let result = client.try_set_timeout(&imposter, &3600);
    assert_eq!(result, Err(Ok(SlotError::NotAdmin)));
}

    let current_time = env.ledger().timestamp();
    client.create_time_slot(
        &String::from_str(&env, "pro"),
        &(current_time.saturating_sub(100)),
        &(current_time + 1000),
    );
}

#[test]
#[should_panic(expected = "Error(Contract, #6)")]
fn test_create_time_slot_invalid_time_range() {
    let env = Env::default();
    let contract_id = env.register(ChronoPayContract, ());
    let client = ChronoPayContractClient::new(&env, &contract_id);

    let current_time = env.ledger().timestamp();
    client.create_time_slot(
        &String::from_str(&env, "pro"),
        &(current_time + 1000),
        &(current_time + 500),
    );
}

#[test]
fn test_create_time_slot_valid() {
    let env = Env::default();
    let contract_id = env.register(ChronoPayContract, ());
    let client = ChronoPayContractClient::new(&env, &contract_id);

    let current_time = env.ledger().timestamp();
    let result = client.create_time_slot(
        &String::from_str(&env, "pro"),
        &(current_time + 1000),
        &(current_time + 2000),
    );
    assert_eq!(result, 1);
}

#[test]
fn test_redeem_by_owner_succeeds() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(ChronoPayContract, ());
    let client = ChronoPayContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    // initialize itself requires auth, so this will panic.
    client.initialize(&admin);
}

    let current_time = env.ledger().timestamp();
    let slot_id = client.create_time_slot(
        &String::from_str(&env, "pro"),
        &(current_time + 1000),
        &(current_time + 2000),
    );
    let token = client.mint_time_token(&slot_id);
    assert_eq!(token, soroban_sdk::Symbol::new(&env, "TIME_TOKEN"));
}

    let owner = Address::generate(&env);
    let seller = Address::generate(&env);
    client.buy_time_token(&token, &owner, &seller);
    client.redeem_time_token(&token);
}

#[test]
#[should_panic(expected = "Error(Contract, #2)")]
fn test_redeem_nonexistent_token_fails() {
    let env = Env::default();
    let contract_id = env.register(ChronoPayContract, ());
    let client = ChronoPayContractClient::new(&env, &contract_id);

    let fake_token = soroban_sdk::Symbol::new(&env, "FAKE");
    client.redeem_time_token(&fake_token);
}

#[test]
#[should_panic(expected = "Error(Contract, #4)")]
fn test_redeem_after_redeem_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(ChronoPayContract, ());
    let client = ChronoPayContractClient::new(&env, &contract_id);

    let current_time = env.ledger().timestamp();
    let slot_id = client.create_time_slot(
        &String::from_str(&env, "pro"),
        &(current_time + 1000),
        &(current_time + 2000),
    );
    let token = client.mint_time_token(&slot_id);

    let owner = Address::generate(&env);
    let seller = Address::generate(&env);
    client.buy_time_token(&token, &owner, &seller);
    client.redeem_time_token(&token);
    client.redeem_time_token(&token);
}
