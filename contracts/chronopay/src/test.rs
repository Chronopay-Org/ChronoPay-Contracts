#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env, String};

#[test]
fn test_initialize_and_update_fee() {
    let env = Env::default();
    let contract_id = env.register(ChronoPayContract, ());
    let client = ChronoPayContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin, &250); // 2.5%

    // Update fee
    env.mock_all_auths();
    client.update_fee(&500); // 5%
    
    // Check buy_time_token fee calculation
    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let token = Symbol::new(&env, "TEST");
    
    let fee_amount = client.buy_time_token(&token, &buyer, &seller, &10000);
    assert_eq!(fee_amount, 500);
}

#[test]
#[should_panic(expected = "already initialized")]
fn test_initialize_twice_fails() {
    let env = Env::default();
    let contract_id = env.register(ChronoPayContract, ());
    let client = ChronoPayContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin, &250);
    client.initialize(&admin, &250);
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