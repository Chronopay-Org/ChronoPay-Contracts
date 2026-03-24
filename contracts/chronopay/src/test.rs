#![cfg(test)]

use super::*;
use soroban_sdk::{vec, Address, Env, String};
use soroban_sdk::testutils::Address as _;

// ---------------------------------------------------------------------------
// Existing tests (unchanged behaviour)
// ---------------------------------------------------------------------------

#[test]
fn test_hello() {
    let env = Env::default();
    let contract_id = env.register(ChronoPayContract, ());
    let client = ChronoPayContractClient::new(&env, &contract_id);

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

#[test]
fn test_create_time_slot_auto_increments() {
    let env = Env::default();
    let contract_id = env.register(ChronoPayContract, ());
    let client = ChronoPayContractClient::new(&env, &contract_id);

    let slot_id_1 = client.create_time_slot(
        &String::from_str(&env, "professional_alice"),
        &1000u64,
        &2000u64,
    );
    let slot_id_2 = client.create_time_slot(
        &String::from_str(&env, "professional_alice"),
        &3000u64,
        &4000u64,
    );

    assert_eq!(slot_id_1, 1);
    assert_eq!(slot_id_2, 2);

    let slot_data = client.get_slot(&1).unwrap();
    assert_eq!(slot_data.professional, String::from_str(&env, "professional_alice"));
    assert_eq!(slot_data.status, TimeTokenStatus::Available);
    assert_eq!(slot_data.owner, None);
}

#[test]
fn test_buy_and_redeem_flow() {
    let env = Env::default();
    let contract_id = env.register(ChronoPayContract, ());
    let client = ChronoPayContractClient::new(&env, &contract_id);
    let buyer = Address::generate(&env);

    let slot_id = client.create_time_slot(&String::from_str(&env, "pro"), &1000u64, &2000u64);

    // Initial state
    let data_init = client.get_slot(&slot_id).unwrap();
    assert_eq!(data_init.status, TimeTokenStatus::Available);

    // Buy
    let bought = client.buy_time_token(&slot_id, &buyer);
    assert!(bought);

    let data_bought = client.get_slot(&slot_id).unwrap();
    assert_eq!(data_bought.status, TimeTokenStatus::Sold);
    assert_eq!(data_bought.owner, Some(buyer.clone()));

    // Redeem
    let redeemed = client.redeem_time_token(&slot_id);
    assert!(redeemed);

    let data_redeem = client.get_slot(&slot_id).unwrap();
    assert_eq!(data_redeem.status, TimeTokenStatus::Redeemed);
}

#[test]
fn test_invalid_transitions() {
    let env = Env::default();
    let contract_id = env.register(ChronoPayContract, ());
    let client = ChronoPayContractClient::new(&env, &contract_id);
    let buyer = Address::generate(&env);

    let slot_id = client.create_time_slot(&String::from_str(&env, "pro"), &1000u64, &2000u64);

    // Cannot redeem before buying
    let redeemed_early = client.redeem_time_token(&slot_id);
    assert!(!redeemed_early);

    // Buy
    client.buy_time_token(&slot_id, &buyer);

    // Cannot buy again once Sold
    let buy_again = client.buy_time_token(&slot_id, &buyer);
    assert!(!buy_again);
}

#[test]
fn test_mint_time_token() {
    let env = Env::default();
    let contract_id = env.register(ChronoPayContract, ());
    let client = ChronoPayContractClient::new(&env, &contract_id);

    let slot_id = client.create_time_slot(&String::from_str(&env, "pro"), &1000u64, &2000u64);
    let token = client.mint_time_token(&slot_id);
    assert_eq!(token, soroban_sdk::Symbol::new(&env, "TIME_TOKEN"));
}

#[test]
#[should_panic(expected = "Slot does not exist")]
fn test_mint_invalid_slot() {
    let env = Env::default();
    let contract_id = env.register(ChronoPayContract, ());
    let client = ChronoPayContractClient::new(&env, &contract_id);
    client.mint_time_token(&999);
}

// ---------------------------------------------------------------------------
// New tests – lightweight query & additional branch coverage
// ---------------------------------------------------------------------------

/// `get_slot_status` tracks status at every lifecycle stage without
/// deserialising metadata bytes.
#[test]
fn test_get_slot_status_lightweight() {
    let env = Env::default();
    let contract_id = env.register(ChronoPayContract, ());
    let client = ChronoPayContractClient::new(&env, &contract_id);
    let buyer = Address::generate(&env);

    let slot_id = client.create_time_slot(&String::from_str(&env, "alice"), &500u64, &1500u64);

    // Available after creation
    assert_eq!(client.get_slot_status(&slot_id), Some(TimeTokenStatus::Available));

    client.buy_time_token(&slot_id, &buyer);
    // Sold after buy
    assert_eq!(client.get_slot_status(&slot_id), Some(TimeTokenStatus::Sold));

    client.redeem_time_token(&slot_id);
    // Redeemed after redeem
    assert_eq!(client.get_slot_status(&slot_id), Some(TimeTokenStatus::Redeemed));
}

/// `get_slot` and `get_slot_status` must return `None` for unknown ids.
#[test]
fn test_slot_not_found_returns_none() {
    let env = Env::default();
    let contract_id = env.register(ChronoPayContract, ());
    let client = ChronoPayContractClient::new(&env, &contract_id);

    assert_eq!(client.get_slot(&42), None);
    assert_eq!(client.get_slot_status(&42), None);
}

/// Calling redeem twice must return `false` on the second attempt.
#[test]
fn test_redeem_already_redeemed() {
    let env = Env::default();
    let contract_id = env.register(ChronoPayContract, ());
    let client = ChronoPayContractClient::new(&env, &contract_id);
    let buyer = Address::generate(&env);

    let slot_id = client.create_time_slot(&String::from_str(&env, "bob"), &0u64, &1000u64);
    client.buy_time_token(&slot_id, &buyer);
    let first_redeem = client.redeem_time_token(&slot_id);
    assert!(first_redeem);

    // Second redeem must be rejected
    let second_redeem = client.redeem_time_token(&slot_id);
    assert!(!second_redeem);
}

/// `get_slot` correctly re-assembles metadata + state into `SlotData`.
#[test]
fn test_get_slot_combines_meta_and_state() {
    let env = Env::default();
    let contract_id = env.register(ChronoPayContract, ());
    let client = ChronoPayContractClient::new(&env, &contract_id);
    let buyer = Address::generate(&env);

    let slot_id = client.create_time_slot(&String::from_str(&env, "carol"), &100u64, &200u64);
    client.buy_time_token(&slot_id, &buyer);

    let data = client.get_slot(&slot_id).unwrap();
    assert_eq!(data.professional, String::from_str(&env, "carol"));
    assert_eq!(data.start_time, 100u64);
    assert_eq!(data.end_time, 200u64);
    assert_eq!(data.owner, Some(buyer));
    assert_eq!(data.status, TimeTokenStatus::Sold);
}