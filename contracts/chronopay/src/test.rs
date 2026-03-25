#![cfg(test)]

use super::*;
use soroban_sdk::{vec, Env, String};

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

    let current_time = env.ledger().timestamp();
    let slot_id_1 = client.create_time_slot(
        &String::from_str(&env, "professional_alice"),
        &(current_time + 1000u64),
        &(current_time + 2000u64),
    );
    let slot_id_2 = client.create_time_slot(
        &String::from_str(&env, "professional_alice"),
        &(current_time + 3000u64),
        &(current_time + 4000u64),
    );
    let slot_id_3 = client.create_time_slot(
        &String::from_str(&env, "professional_alice"),
        &(current_time + 5000u64),
        &(current_time + 6000u64),
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

    let current_time = env.ledger().timestamp();
    let slot_id = client.create_time_slot(
        &String::from_str(&env, "pro"),
        &(current_time + 1000u64),
        &(current_time + 2000u64),
    );
    let token = client.mint_time_token(&slot_id);
    assert_eq!(token, soroban_sdk::Symbol::new(&env, "TIME_TOKEN"));

    let redeemed = client.redeem_time_token(&token);
    assert!(redeemed);
}

#[test]
#[should_panic(expected = "Error(Contract, #5)")]
fn test_create_time_slot_start_time_in_past() {
    let env = Env::default();
    let contract_id = env.register(ChronoPayContract, ());
    let client = ChronoPayContractClient::new(&env, &contract_id);

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
