#![cfg(test)]

use super::*;
use soroban_sdk::{vec, Env, String, Symbol};

/// Helper function to create a test token symbol
fn create_test_token(env: &Env) -> Symbol {
    Symbol::new(env, "TIME_TOKEN")
}

/// Helper function to create an empty string for testing
fn empty_string(env: &Env) -> String {
    String::from_str(env, "")
}

/// Helper function to create a valid non-empty string
fn valid_string(env: &Env, s: &str) -> String {
    String::from_str(env, s)
}

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
fn test_create_time_slot() {
    let env = Env::default();
    let contract_id = env.register(ChronoPayContract, ());
    let client = ChronoPayContractClient::new(&env, &contract_id);

    let slot_id = client.create_time_slot(
        &String::from_str(&env, "professional_alice"),
        &1000u64,
        &2000u64,
    );
    assert_eq!(slot_id, 1);
}

#[test]
fn test_mint_and_redeem() {
    let env = Env::default();
    let contract_id = env.register(ChronoPayContract, ());
    let client = ChronoPayContractClient::new(&env, &contract_id);

    let _slot_id = client.create_time_slot(&String::from_str(&env, "pro"), &1000u64, &2000u64);
    let token = client.mint_time_token(&1);
    assert_eq!(token, soroban_sdk::Symbol::new(&env, "TIME_TOKEN"));

    // Now redeem returns Result<(), ChronoPayError>, client returns Ok value directly
    // or panics on error
    client.redeem_time_token(&token);
}

#[test]
fn test_redeem_time_token_returns_ok_with_valid_token() {
    let env = Env::default();
    let contract_id = env.register(ChronoPayContract, ());
    let client = ChronoPayContractClient::new(&env, &contract_id);

    // Create a time slot and mint a token
    let _slot_id = client.create_time_slot(
        &String::from_str(&env, "professional_alice"),
        &1000u64,
        &2000u64,
    );
    let token = client.mint_time_token(&1);

    // Redeem should succeed - client returns Ok value directly
    client.redeem_time_token(&token);
}

#[test]
#[should_panic(expected = "Error(Contract, #5)")]
fn test_redeem_time_token_returns_err_when_token_not_found() {
    let env = Env::default();
    let contract_id = env.register(ChronoPayContract, ());
    let client = ChronoPayContractClient::new(&env, &contract_id);

    // Try to redeem with an empty Symbol (which is the default/empty value)
    let empty_token = Symbol::new(&env, "");
    // This should panic with TokenNotFound error
    client.redeem_time_token(&empty_token);
}

#[test]
fn test_buy_time_token_returns_ok_with_valid_inputs() {
    let env = Env::default();
    let contract_id = env.register(ChronoPayContract, ());
    let client = ChronoPayContractClient::new(&env, &contract_id);

    let token = create_test_token(&env);
    let buyer = valid_string(&env, "buyer_address");
    let seller = valid_string(&env, "seller_address");

    // Buy should succeed with valid inputs - client returns Ok value directly
    client.buy_time_token(&token, &buyer, &seller);
}

#[test]
#[should_panic(expected = "Error(Contract, #2)")]
fn test_buy_time_token_returns_err_when_buyer_is_invalid() {
    let env = Env::default();
    let contract_id = env.register(ChronoPayContract, ());
    let client = ChronoPayContractClient::new(&env, &contract_id);

    let token = create_test_token(&env);
    let empty_buyer = empty_string(&env);
    let seller = valid_string(&env, "seller_address");

    // Buy should fail with empty buyer - client panics on error
    client.buy_time_token(&token, &empty_buyer, &seller);
}

#[test]
#[should_panic(expected = "Error(Contract, #2)")]
fn test_buy_time_token_returns_err_when_seller_is_invalid() {
    let env = Env::default();
    let contract_id = env.register(ChronoPayContract, ());
    let client = ChronoPayContractClient::new(&env, &contract_id);

    let token = create_test_token(&env);
    let buyer = valid_string(&env, "buyer_address");
    let empty_seller = empty_string(&env);

    // Buy should fail with empty seller - client panics on error
    client.buy_time_token(&token, &buyer, &empty_seller);
}

#[test]
#[should_panic(expected = "Error(Contract, #2)")]
fn test_buy_time_token_returns_err_when_both_addresses_invalid() {
    let env = Env::default();
    let contract_id = env.register(ChronoPayContract, ());
    let client = ChronoPayContractClient::new(&env, &contract_id);

    let token = create_test_token(&env);
    let empty_buyer = empty_string(&env);
    let empty_seller = empty_string(&env);

    // Buy should fail with both addresses empty - client panics on error
    client.buy_time_token(&token, &empty_buyer, &empty_seller);
}
