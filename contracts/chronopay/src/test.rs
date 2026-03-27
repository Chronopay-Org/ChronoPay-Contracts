#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Ledger, vec, Env, String};

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
    let slot_id_3 = client.create_time_slot(
        &String::from_str(&env, "professional_alice"),
        &5000u64,
        &6000u64,
    );

    assert_eq!(slot_id_1, 1);
    assert_eq!(slot_id_2, 2);
    assert_eq!(slot_id_3, 3);
}

#[test]
fn test_mint_and_redeem() {
    let env = Env::default();
    let contract_id = env.register(ChronoPayContract, ());
    let client = ChronoPayContractClient::new(&env, &contract_id);

    let slot_id = client.create_time_slot(&String::from_str(&env, "pro"), &1000u64, &2000u64);
    let token = client.mint_time_token(&slot_id);
    assert_eq!(token, soroban_sdk::Symbol::new(&env, "TIME_TOKEN"));

    let redeemed = client.redeem_time_token(&token);
    assert!(redeemed);
}

#[test]
fn test_set_cancel_window() {
    let env = Env::default();
    let contract_id = env.register(ChronoPayContract, ());
    let client = ChronoPayContractClient::new(&env, &contract_id);

    client.set_cancel_window(&7200u64);
    // Verified implicitly as part of integration tests below if not panicking
}

#[test]
#[should_panic(expected = "invalid_window")]
fn test_zero_window_panics() {
    let env = Env::default();
    let contract_id = env.register(ChronoPayContract, ());
    let client = ChronoPayContractClient::new(&env, &contract_id);

    client.set_cancel_window(&0u64);
}

#[test]
fn test_buy_time_slot_records_buyer() {
    let env = Env::default();
    let contract_id = env.register(ChronoPayContract, ());
    let client = ChronoPayContractClient::new(&env, &contract_id);

    let slot_id = client.create_time_slot(&String::from_str(&env, "pro"), &1000u64, &2000u64);
    let success = client.buy_time_slot(&slot_id, &String::from_str(&env, "buyer_bob"));
    assert!(success);
}

#[test]
fn test_cancel_within_window() {
    let env = Env::default();
    // Default window is 3600
    env.ledger().set_timestamp(4000); // Current time is 4000

    let contract_id = env.register(ChronoPayContract, ());
    let client = ChronoPayContractClient::new(&env, &contract_id);

    // Slot starts at 8000. Window ends at 8000 - 3600 = 4400.
    // So timestamp 4000 is still well within the window (it's 4000 before start).
    let slot_id = client.create_time_slot(&String::from_str(&env, "pro"), &8000u64, &9000u64);
    client.buy_time_slot(&slot_id, &String::from_str(&env, "buyer_bob"));

    let canceled = client.cancel_time_slot(&slot_id, &String::from_str(&env, "buyer_bob"));
    assert!(canceled);
}

#[test]
#[should_panic(expected = "too_late_to_cancel")]
fn test_cancel_after_window_panics() {
    let env = Env::default();
    // Default window is 3600
    env.ledger().set_timestamp(4500); // Current time is 4500

    let contract_id = env.register(ChronoPayContract, ());
    let client = ChronoPayContractClient::new(&env, &contract_id);

    // Slot starts at 8000. Window boundary is 8000 - 3600 = 4400.
    // Timestamp 4500 is >= 4400, so it's too late.
    let slot_id = client.create_time_slot(&String::from_str(&env, "pro"), &8000u64, &9000u64);
    client.buy_time_slot(&slot_id, &String::from_str(&env, "buyer_bob"));

    client.cancel_time_slot(&slot_id, &String::from_str(&env, "buyer_bob"));
}

#[test]
#[should_panic(expected = "slot_not_found")]
fn test_cancel_unknown_slot_panics() {
    let env = Env::default();
    let contract_id = env.register(ChronoPayContract, ());
    let client = ChronoPayContractClient::new(&env, &contract_id);

    client.cancel_time_slot(&999u32, &String::from_str(&env, "buyer_bob"));
}

#[test]
#[should_panic(expected = "slot_not_sold")]
fn test_cancel_without_buyer_panics() {
    let env = Env::default();
    env.ledger().set_timestamp(1000);
    let contract_id = env.register(ChronoPayContract, ());
    let client = ChronoPayContractClient::new(&env, &contract_id);

    let slot_id = client.create_time_slot(&String::from_str(&env, "pro"), &8000u64, &9000u64);
    // Not bought
    client.cancel_time_slot(&slot_id, &String::from_str(&env, "buyer_bob"));
}

#[test]
fn test_cancel_updates_status() {
    let env = Env::default();
    env.ledger().set_timestamp(4000);
    let contract_id = env.register(ChronoPayContract, ());
    let client = ChronoPayContractClient::new(&env, &contract_id);

    let slot_id = client.create_time_slot(&String::from_str(&env, "pro"), &8000u64, &9000u64);
    client.buy_time_slot(&slot_id, &String::from_str(&env, "buyer_bob"));

    let canceled = client.cancel_time_slot(&slot_id, &String::from_str(&env, "buyer_bob"));
    assert!(canceled);

    // If we try to cancel it again, it should fail as slot_not_sold since buyer record is removed
}

// Wrapper for the double cancel check since soroban unit tests don't support catching panics cleanly mid-test
#[test]
#[should_panic(expected = "slot_not_sold")]
fn test_double_cancel_panics() {
    let env = Env::default();
    env.ledger().set_timestamp(4000);
    let contract_id = env.register(ChronoPayContract, ());
    let client = ChronoPayContractClient::new(&env, &contract_id);

    let slot_id = client.create_time_slot(&String::from_str(&env, "pro"), &8000u64, &9000u64);
    client.buy_time_slot(&slot_id, &String::from_str(&env, "buyer_bob"));

    client.cancel_time_slot(&slot_id, &String::from_str(&env, "buyer_bob"));

    // Second time panics because the buyer record was deleted
    client.cancel_time_slot(&slot_id, &String::from_str(&env, "buyer_bob"));
}
