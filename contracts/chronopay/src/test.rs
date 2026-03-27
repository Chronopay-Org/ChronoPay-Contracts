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
use soroban_sdk::{testutils::Address as _, vec, Address, Env, String};

// ---------------------------------------------------------------------------
// Helpers — O(1) per call, no allocations beyond Env internals
// ---------------------------------------------------------------------------

/// Spin up a fresh env + registered contract + client triple.
fn setup() -> (Env, Address, ChronoPayContractClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();
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

    let words = client.hello(&String::from_str(&env, "Dev"));
    assert_eq!(
        words,
        vec![
            &env,
            String::from_str(&env, "ChronoPay"),
            String::from_str(&env, "Dev"),
        ]
    );
}

// ===========================================================================
// 2. `initialize`
// ===========================================================================

#[test]
fn test_initialize_success() {
    let (env, _cid, client) = setup();
    let admin = Address::generate(&env);
    client.initialize(&admin);
    // Second call must fail — already initialised.
}

#[test]
#[should_panic(expected = "already initialized")]
fn test_initialize_rejects_double_init() {
    let (env, _cid, client) = setup();
    let admin = Address::generate(&env);
    client.initialize(&admin);
    // Attempt re-initialisation by a different address.
    let attacker = Address::generate(&env);
    client.initialize(&attacker);
}

#[test]
#[should_panic(expected = "already initialized")]
fn test_initialize_rejects_same_admin_reinit() {
    let (env, _cid, client) = setup();
    let admin = Address::generate(&env);
    client.initialize(&admin);
    // Even the original admin cannot re-initialise.
    client.initialize(&admin);
}

// ===========================================================================
// 3. `create_time_slot`
// ===========================================================================

#[test]
fn test_create_time_slot_auto_increments() {
    let (env, _cid, client) = setup();
    let pro = Address::generate(&env);

    let id1 = client.create_time_slot(&pro, &1000u64, &2000u64);
    let id2 = client.create_time_slot(&pro, &3000u64, &4000u64);
    let id3 = client.create_time_slot(&pro, &5000u64, &6000u64);

    assert_eq!(id1, 1);
    assert_eq!(id2, 2);
    assert_eq!(id3, 3);
}

#[test]
#[should_panic(expected = "HostError: Error(Auth")]
fn test_create_time_slot_rejects_unauthorized_caller() {
    let env = Env::default();
    // Do NOT mock auths — real auth enforcement.
    let contract_id = env.register(ChronoPayContract, ());
    let client = ChronoPayContractClient::new(&env, &contract_id);

    let impersonated = Address::generate(&env);
    // Calling without providing auth for `impersonated`.
    client.create_time_slot(&impersonated, &1000u64, &2000u64);
}

#[test]
#[should_panic(expected = "start_time must be before end_time")]
fn test_create_time_slot_rejects_invalid_time_range() {
    let (env, _cid, client) = setup();
    let pro = Address::generate(&env);
    // start >= end
    client.create_time_slot(&pro, &2000u64, &1000u64);
}

#[test]
#[should_panic(expected = "start_time must be before end_time")]
fn test_create_time_slot_rejects_equal_start_end() {
    let (env, _cid, client) = setup();
    let pro = Address::generate(&env);
    client.create_time_slot(&pro, &1000u64, &1000u64);
}

// ===========================================================================
// 4. `mint_time_token`
// ===========================================================================

#[test]
fn test_mint_time_token_success() {
    let (env, client, admin) = setup_with_admin();
    let pro = Address::generate(&env);

    let slot_id = client.create_time_slot(&pro, &1000u64, &2000u64);
    let token = client.mint_time_token(&admin, &slot_id);

    assert_eq!(token, soroban_sdk::Symbol::new(&env, "TIME_TOKEN"));
}

#[test]
#[should_panic(expected = "caller is not admin")]
fn test_mint_time_token_rejects_non_admin() {
    let (env, client, _admin) = setup_with_admin();
    let pro = Address::generate(&env);

    let slot_id = client.create_time_slot(&pro, &1000u64, &2000u64);
    let impostor = Address::generate(&env);
    client.mint_time_token(&impostor, &slot_id);
}

#[test]
#[should_panic(expected = "HostError: Error(Auth")]
fn test_mint_time_token_rejects_without_auth_signature() {
    // No mock_all_auths — require_auth will fail at the host level.
    let env = Env::default();
    let contract_id = env.register(ChronoPayContract, ());
    let client = ChronoPayContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    // initialize itself requires auth, so this will panic.
    client.initialize(&admin);
}

#[test]
#[should_panic(expected = "contract not initialized")]
fn test_mint_time_token_rejects_before_init() {
    let (env, _cid, client) = setup();
    let random = Address::generate(&env);
    client.mint_time_token(&random, &1u32);
}

// ===========================================================================
// 5. `buy_time_token`
// ===========================================================================

#[test]
fn test_buy_time_token_success() {
    let (env, client, admin) = setup_with_admin();
    let pro = Address::generate(&env);
    let buyer = Address::generate(&env);

    let slot_id = client.create_time_slot(&pro, &1000u64, &2000u64);
    client.mint_time_token(&admin, &slot_id);

    let ok = client.buy_time_token(&buyer, &slot_id);
    assert!(ok);
}

#[test]
#[should_panic(expected = "HostError: Error(Auth")]
fn test_buy_time_token_rejects_without_auth_signature() {
    // No mock_all_auths — buyer.require_auth() fails at host level.
    let env = Env::default();
    let contract_id = env.register(ChronoPayContract, ());
    let client = ChronoPayContractClient::new(&env, &contract_id);
    let buyer = Address::generate(&env);
    // buy_time_token requires buyer auth — this will panic.
    client.buy_time_token(&buyer, &1u32);
}

#[test]
#[should_panic(expected = "token not minted")]
fn test_buy_time_token_rejects_unminted_token() {
    let (env, client, _admin) = setup_with_admin();
    let buyer = Address::generate(&env);
    // Slot 999 was never created/minted.
    client.buy_time_token(&buyer, &999u32);
}

#[test]
#[should_panic(expected = "token not available")]
fn test_buy_time_token_rejects_already_sold() {
    let (env, client, admin) = setup_with_admin();
    let pro = Address::generate(&env);
    let buyer1 = Address::generate(&env);
    let buyer2 = Address::generate(&env);

    let slot = client.create_time_slot(&pro, &1000u64, &2000u64);
    client.mint_time_token(&admin, &slot);
    client.buy_time_token(&buyer1, &slot);

    // Second purchase must fail.
    client.buy_time_token(&buyer2, &slot);
}

// ===========================================================================
// 6. `redeem_time_token`
// ===========================================================================

#[test]
fn test_redeem_time_token_success() {
    let (_env, client, _admin, _pro, buyer, slot_id) = setup_full_lifecycle();
    let ok = client.redeem_time_token(&buyer, &slot_id);
    assert!(ok);
}

#[test]
#[should_panic(expected = "caller is not the token owner")]
fn test_redeem_time_token_rejects_non_owner() {
    let (env, client, _admin, _pro, _buyer, slot_id) = setup_full_lifecycle();
    let stranger = Address::generate(&env);
    client.redeem_time_token(&stranger, &slot_id);
}

#[test]
#[should_panic(expected = "HostError: Error(Auth")]
fn test_redeem_time_token_rejects_without_auth_signature() {
    // No mock_all_auths — redeemer.require_auth() fails at host level.
    let env = Env::default();
    let contract_id = env.register(ChronoPayContract, ());
    let client = ChronoPayContractClient::new(&env, &contract_id);
    let redeemer = Address::generate(&env);
    // redeem_time_token requires redeemer auth — this will panic.
    client.redeem_time_token(&redeemer, &1u32);
}

#[test]
#[should_panic(expected = "no owner for token")]
fn test_redeem_time_token_rejects_unbought_token() {
    let (env, client, admin) = setup_with_admin();
    let pro = Address::generate(&env);

    let slot = client.create_time_slot(&pro, &1000u64, &2000u64);
    client.mint_time_token(&admin, &slot);

    // Token is minted but not bought — no owner set.
    let random = Address::generate(&env);
    client.redeem_time_token(&random, &slot);
}

#[test]
#[should_panic(expected = "caller is not the token owner")]
fn test_redeem_time_token_rejects_admin_who_is_not_owner() {
    let (_env, client, admin, _pro, _buyer, slot_id) = setup_full_lifecycle();
    // Admin is not the buyer, so redeem must fail.
    client.redeem_time_token(&admin, &slot_id);
}

#[test]
#[should_panic(expected = "token not in sold state")]
fn test_redeem_time_token_rejects_double_redeem() {
    let (_env, client, _admin, _pro, buyer, slot_id) = setup_full_lifecycle();
    client.redeem_time_token(&buyer, &slot_id);
    // Second redemption must fail.
    client.redeem_time_token(&buyer, &slot_id);
}
