#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, vec, Address, Env, String};

fn setup() -> (Env, ChronoPayContractClient<'static>, Address) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(ChronoPayContract, ());
    let client = ChronoPayContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    client.initialize(&admin);
    (env, client, admin)
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
fn test_create_time_slot_auto_increments() {
    let (env, client, _) = setup();

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
    let (env, client, _) = setup();

    let slot_id = client.create_time_slot(&String::from_str(&env, "pro"), &1000u64, &2000u64);
    let token = client.mint_time_token(&slot_id);
    assert_eq!(token, soroban_sdk::Symbol::new(&env, "TIME_TOKEN"));

    let redeemed = client.redeem_time_token(&token);
    assert!(redeemed);
}

#[test]
fn test_initialize_not_paused() {
    let (_, client, _) = setup();
    assert!(!client.is_paused());
}

#[test]
fn test_pause_and_unpause() {
    let (_, client, admin) = setup();
    assert!(!client.is_paused());
    client.pause(&admin);
    assert!(client.is_paused());
    client.unpause(&admin);
    assert!(!client.is_paused());
}

#[test]
fn test_pause_is_idempotent() {
    let (_, client, admin) = setup();
    client.pause(&admin);
    client.pause(&admin);
    assert!(client.is_paused());
}

#[test]
fn test_unpause_is_idempotent() {
    let (_, client, admin) = setup();
    client.unpause(&admin);
    assert!(!client.is_paused());
}

#[test]
#[should_panic(expected = "contract is paused")]
fn test_create_time_slot_blocked_when_paused() {
    let (env, client, admin) = setup();
    client.pause(&admin);
    client.create_time_slot(&String::from_str(&env, "pro"), &1000u64, &2000u64);
}

#[test]
#[should_panic(expected = "contract is paused")]
fn test_mint_time_token_blocked_when_paused() {
    let (_, client, admin) = setup();
    client.pause(&admin);
    client.mint_time_token(&1u32);
}

#[test]
#[should_panic(expected = "contract is paused")]
fn test_buy_time_token_blocked_when_paused() {
    let (env, client, admin) = setup();
    client.pause(&admin);
    client.buy_time_token(
        &soroban_sdk::Symbol::new(&env, "TOKEN"),
        &String::from_str(&env, "buyer"),
        &String::from_str(&env, "seller"),
    );
}

#[test]
#[should_panic(expected = "contract is paused")]
fn test_redeem_time_token_blocked_when_paused() {
    let (env, client, admin) = setup();
    client.pause(&admin);
    client.redeem_time_token(&soroban_sdk::Symbol::new(&env, "TOKEN"));
}

#[test]
fn test_hello_works_when_paused() {
    let (env, client, admin) = setup();
    client.pause(&admin);
    let words = client.hello(&String::from_str(&env, "Dev"));
    assert_eq!(words.len(), 2);
}

#[test]
fn test_operations_resume_after_unpause() {
    let (env, client, admin) = setup();
    client.pause(&admin);
    client.unpause(&admin);
    let slot_id =
        client.create_time_slot(&String::from_str(&env, "pro"), &1000u64, &2000u64);
    assert_eq!(slot_id, 1);
}

#[test]
#[should_panic(expected = "unauthorized: caller is not admin")]
fn test_non_admin_cannot_pause() {
    let (env, client, _) = setup();
    let attacker = Address::generate(&env);
    client.pause(&attacker);
}

#[test]
#[should_panic(expected = "unauthorized: caller is not admin")]
fn test_non_admin_cannot_unpause() {
    let (env, client, admin) = setup();
    let attacker = Address::generate(&env);
    client.pause(&admin);
    client.unpause(&attacker);
}

#[test]
#[should_panic(expected = "already initialized")]
fn test_double_initialize_panics() {
    let (env, client, _) = setup();
    let second_admin = Address::generate(&env);
    client.initialize(&second_admin);
}