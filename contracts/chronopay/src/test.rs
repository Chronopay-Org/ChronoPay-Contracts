#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, vec, Address, Env, String};

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

    let professional = Address::generate(&env);

    // Mock auth for the professional
    env.mock_all_auths();

    let slot_id_1 = client.create_time_slot(&professional, &1000u64, &2000u64);
    let slot_id_2 = client.create_time_slot(&professional, &3000u64, &4000u64);
    let slot_id_3 = client.create_time_slot(&professional, &5000u64, &6000u64);

    assert_eq!(slot_id_1, 1);
    assert_eq!(slot_id_2, 2);
    assert_eq!(slot_id_3, 3);
}

#[test]
#[should_panic(expected = "invalid time range")]
fn test_create_time_slot_invalid_range() {
    let env = Env::default();
    let contract_id = env.register(ChronoPayContract, ());
    let client = ChronoPayContractClient::new(&env, &contract_id);

    let professional = Address::generate(&env);
    env.mock_all_auths();

    client.create_time_slot(&professional, &2000u64, &1000u64);
}

#[test]
fn test_buy_time_token_with_auth() {
    let env = Env::default();
    let contract_id = env.register(ChronoPayContract, ());
    let client = ChronoPayContractClient::new(&env, &contract_id);

    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let token = Symbol::new(&env, "TEST_TOKEN");

    env.mock_all_auths();

    let success = client.buy_time_token(&token, &buyer, &seller);
    assert!(success);
}

#[test]
fn test_mint_and_redeem() {
    let env = Env::default();
    let contract_id = env.register(ChronoPayContract, ());
    let client = ChronoPayContractClient::new(&env, &contract_id);

    let pro = Address::generate(&env);
    env.mock_all_auths();

    let slot_id = client.create_time_slot(&pro, &1000u64, &2000u64);
    let token = client.mint_time_token(&slot_id);
    assert_eq!(token, Symbol::new(&env, "TIME_TOKEN"));

    let redeemed = client.redeem_time_token(&token);
    assert!(redeemed);
}

#[test]
#[should_panic(expected = "token already redeemed")]
fn test_double_redemption_fails() {
    let env = Env::default();
    let contract_id = env.register(ChronoPayContract, ());
    let client = ChronoPayContractClient::new(&env, &contract_id);

    let token = Symbol::new(&env, "TEST_TOKEN");

    // First redemption
    client.redeem_time_token(&token);

    // Second redemption should panic
    client.redeem_time_token(&token);
}