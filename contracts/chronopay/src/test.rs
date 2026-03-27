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
    let owner = Address::generate(&env);

    let slot_id = client.create_time_slot(&String::from_str(&env, "pro"), &1000u64, &2000u64);
    let token = client.mock_all_auths().mint_time_token(&slot_id, &owner);
    assert_eq!(token, soroban_sdk::Symbol::new(&env, "TIME_TOKEN"));

    let redeemed = client.mock_all_auths().redeem_time_token(&token, &owner);
    assert!(redeemed);
}

#[test]
#[should_panic(expected = "Error(Contract, #4)")]
fn test_redeem_time_token_rejects_non_owner() {
    let env = Env::default();
    let contract_id = env.register(ChronoPayContract, ());
    let client = ChronoPayContractClient::new(&env, &contract_id);
    let owner = Address::generate(&env);
    let attacker = Address::generate(&env);

    let slot_id = client.create_time_slot(&String::from_str(&env, "pro"), &1000u64, &2000u64);
    let token = client.mock_all_auths().mint_time_token(&slot_id, &owner);

    client.mock_all_auths().redeem_time_token(&token, &attacker);
}

#[test]
#[should_panic(expected = "Error(Contract, #5)")]
fn test_redeem_time_token_rejects_double_redeem() {
    let env = Env::default();
    let contract_id = env.register(ChronoPayContract, ());
    let client = ChronoPayContractClient::new(&env, &contract_id);
    let owner = Address::generate(&env);

    let slot_id = client.create_time_slot(&String::from_str(&env, "pro"), &1000u64, &2000u64);
    let token = client.mock_all_auths().mint_time_token(&slot_id, &owner);

    assert!(client.mock_all_auths().redeem_time_token(&token, &owner));
    client.mock_all_auths().redeem_time_token(&token, &owner);
}

#[test]
fn test_buy_transfers_ownership_and_allows_new_owner_redemption() {
    let env = Env::default();
    let contract_id = env.register(ChronoPayContract, ());
    let client = ChronoPayContractClient::new(&env, &contract_id);
    let seller = Address::generate(&env);
    let buyer = Address::generate(&env);

    let slot_id = client.create_time_slot(&String::from_str(&env, "pro"), &1000u64, &2000u64);
    let token = client.mock_all_auths().mint_time_token(&slot_id, &seller);

    assert!(client
        .mock_all_auths()
        .buy_time_token(&token, &buyer, &seller));
    assert!(client.mock_all_auths().redeem_time_token(&token, &buyer));
}

#[test]
#[should_panic(expected = "Error(Contract, #1)")]
fn test_mint_time_token_rejects_unknown_slot() {
    let env = Env::default();
    let contract_id = env.register(ChronoPayContract, ());
    let client = ChronoPayContractClient::new(&env, &contract_id);
    let owner = Address::generate(&env);

    client.mock_all_auths().mint_time_token(&999u32, &owner);
}
