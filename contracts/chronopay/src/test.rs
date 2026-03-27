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

fn create_long_string(env: &Env) -> String {
    let mut s = [0u8; 65];
    for i in 0..65 {
        s[i] = b'A';
    }
    let st = core::str::from_utf8(&s).unwrap();
    String::from_str(env, st)
}

#[test]
#[should_panic(expected = "string_too_long")]
fn test_long_professional_panics() {
    let env = Env::default();
    let contract_id = env.register(ChronoPayContract, ());
    let client = ChronoPayContractClient::new(&env, &contract_id);
    let long_prof = create_long_string(&env);
    client.create_time_slot(&long_prof, &1000u64, &2000u64);
}

#[test]
#[should_panic(expected = "string_too_long")]
fn test_long_buyer_panics() {
    let env = Env::default();
    let contract_id = env.register(ChronoPayContract, ());
    let client = ChronoPayContractClient::new(&env, &contract_id);
    let long_buyer = create_long_string(&env);
    let valid_seller = String::from_str(&env, "seller");
    client.buy_time_token(
        &soroban_sdk::Symbol::new(&env, "TKN"),
        &long_buyer,
        &valid_seller,
    );
}

#[test]
#[should_panic(expected = "string_too_long")]
fn test_long_seller_panics() {
    let env = Env::default();
    let contract_id = env.register(ChronoPayContract, ());
    let client = ChronoPayContractClient::new(&env, &contract_id);
    let valid_buyer = String::from_str(&env, "buyer");
    let long_seller = create_long_string(&env);
    client.buy_time_token(
        &soroban_sdk::Symbol::new(&env, "TKN"),
        &valid_buyer,
        &long_seller,
    );
}

#[test]
#[should_panic(expected = "string_too_long")]
fn test_long_hello_panics() {
    let env = Env::default();
    let contract_id = env.register(ChronoPayContract, ());
    let client = ChronoPayContractClient::new(&env, &contract_id);
    let long_to = create_long_string(&env);
    client.hello(&long_to);
}
