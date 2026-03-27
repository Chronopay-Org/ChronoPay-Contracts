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

    let alice = Address::generate(&env);
    env.mock_all_auths();

    let slot_id_1 = client.create_time_slot(&alice, &1000u64, &2000u64);
    let slot_id_2 = client.create_time_slot(&alice, &3000u64, &4000u64);
    let slot_id_3 = client.create_time_slot(&alice, &5000u64, &6000u64);

    assert_eq!(slot_id_1, 1);
    assert_eq!(slot_id_2, 2);
    assert_eq!(slot_id_3, 3);
}

#[test]
fn test_mint_and_redeem() {
    let env = Env::default();
    let contract_id = env.register(ChronoPayContract, ());
    let client = ChronoPayContractClient::new(&env, &contract_id);

    let alice = Address::generate(&env);
    env.mock_all_auths();

    let slot_id = client.create_time_slot(&alice, &1000u64, &2000u64);
    let token = client.mint_time_token(&slot_id);
    assert_eq!(token, soroban_sdk::Symbol::new(&env, "TIME_TOKEN"));

    let redeemed = client.redeem_time_token(&slot_id);
    assert!(redeemed);
}

#[test]
fn test_ownership_transfer() {
    let env = Env::default();
    let contract_id = env.register(ChronoPayContract, ());
    let client = ChronoPayContractClient::new(&env, &contract_id);

    let seller = Address::generate(&env);
    let buyer = Address::generate(&env);

    env.mock_all_auths();

    // 1. Seller creates a time slot
    let slot_id = client.create_time_slot(&seller, &1000u64, &2000u64);

    // 2. Buyer buys the time token from Seller
    let success = client.buy_time_token(&slot_id, &buyer, &seller);
    assert!(success);

    // 3. Buyer should now be the owner and can redeem
    let redeemed = client.redeem_time_token(&slot_id);
    assert!(redeemed);
}

#[test]
#[should_panic(expected = "seller is not the owner")]
fn test_buy_fails_if_not_owner() {
    let env = Env::default();
    let contract_id = env.register(ChronoPayContract, ());
    let client = ChronoPayContractClient::new(&env, &contract_id);

    let seller = Address::generate(&env);
    let malicious = Address::generate(&env);
    let buyer = Address::generate(&env);

    env.mock_all_auths();

    let slot_id = client.create_time_slot(&seller, &1000u64, &2000u64);

    // malicous tries to sell it
    client.buy_time_token(&slot_id, &buyer, &malicious);
}

#[test]
#[should_panic(expected = "token already redeemed")]
fn test_redeem_fails_if_already_redeemed() {
    let env = Env::default();
    let contract_id = env.register(ChronoPayContract, ());
    let client = ChronoPayContractClient::new(&env, &contract_id);

    let alice = Address::generate(&env);
    env.mock_all_auths();

    let slot_id = client.create_time_slot(&alice, &1000u64, &2000u64);
    client.redeem_time_token(&slot_id);
    client.redeem_time_token(&slot_id);
}
