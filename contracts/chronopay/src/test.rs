#![cfg(test)]

use super::*;
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{vec, Address, Env, String, Symbol};

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
fn test_purchase_nonce_defaults_to_zero() {
    let env = Env::default();
    let contract_id = env.register(ChronoPayContract, ());
    let client = ChronoPayContractClient::new(&env, &contract_id);

    let buyer = Address::generate(&env);
    assert_eq!(client.get_purchase_nonce(&buyer), 0);
}

#[test]
fn test_buy_time_token_consumes_nonce_and_requires_auth() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(ChronoPayContract, ());
    let client = ChronoPayContractClient::new(&env, &contract_id);

    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let token = Symbol::new(&env, "TIME_TOKEN");

    assert_eq!(client.get_purchase_nonce(&buyer), 0);

    let bought = client.buy_time_token(&token, &buyer, &seller, &0u64);
    assert!(bought);

    // `mock_all_auths()` records every `require_auth` call.
    let auths = env.auths();
    assert_eq!(auths.len(), 1);
    assert_eq!(auths[0].0, buyer);

    assert_eq!(client.get_purchase_nonce(&buyer), 1);
}

#[test]
fn test_buy_time_token_rejects_invalid_nonce() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(ChronoPayContract, ());
    let client = ChronoPayContractClient::new(&env, &contract_id);

    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let token = Symbol::new(&env, "TIME_TOKEN");

    // The first expected nonce is 0.
    let res = client.try_buy_time_token(&token, &buyer, &seller, &1u64);
    assert_eq!(res, Err(Ok(Error::InvalidPurchaseNonce)));
}

#[test]
fn test_buy_time_token_rejects_replay_nonce() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(ChronoPayContract, ());
    let client = ChronoPayContractClient::new(&env, &contract_id);

    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let token = Symbol::new(&env, "TIME_TOKEN");

    assert!(client.buy_time_token(&token, &buyer, &seller, &0u64));

    // Replaying nonce 0 should fail now that the expected nonce is 1.
    let res = client.try_buy_time_token(&token, &buyer, &seller, &0u64);
    assert_eq!(res, Err(Ok(Error::InvalidPurchaseNonce)));
}

#[test]
fn test_purchase_nonce_is_scoped_to_buyer() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(ChronoPayContract, ());
    let client = ChronoPayContractClient::new(&env, &contract_id);

    let buyer_a = Address::generate(&env);
    let buyer_b = Address::generate(&env);
    let seller = Address::generate(&env);
    let token = Symbol::new(&env, "TIME_TOKEN");

    assert!(client.buy_time_token(&token, &buyer_a, &seller, &0u64));

    assert_eq!(client.get_purchase_nonce(&buyer_a), 1);
    assert_eq!(client.get_purchase_nonce(&buyer_b), 0);
}

#[test]
fn test_buy_time_token_rejects_nonce_overflow() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(ChronoPayContract, ());
    let client = ChronoPayContractClient::new(&env, &contract_id);

    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let token = Symbol::new(&env, "TIME_TOKEN");

    // Force the nonce to its maximum value to exercise overflow protection.
    env.as_contract(&contract_id, || {
        env.storage()
            .instance()
            .set(&DataKey::PurchaseNonce(buyer.clone()), &u64::MAX);
    });

    let res = client.try_buy_time_token(&token, &buyer, &seller, &u64::MAX);
    assert_eq!(res, Err(Ok(Error::PurchaseNonceOverflow)));
}
