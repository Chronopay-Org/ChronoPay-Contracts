#![cfg(test)]

use super::*;
use soroban_sdk::{vec, Env, String, Address, Symbol};

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
    env.mock_all_auths(); // for auth

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
    env.mock_all_auths();

    let contract_id = env.register(ChronoPayContract, ());
    let client = ChronoPayContractClient::new(&env, &contract_id);

    let slot_id = client.create_time_slot(&String::from_str(&env, "pro"), &1000u64, &2000u64);
    let token = client.mint_time_token(&slot_id);
    assert_eq!(token, Symbol::new(&env, "TIME_TOKEN"));

    let redeemed = client.redeem_time_token(&token);
    assert!(redeemed);

    // Ensure status is updated
    let status: TimeTokenStatus = env
        .storage()
        .instance()
        .get(&DataKey::Status)
        .unwrap();
    assert_eq!(status, TimeTokenStatus::Redeemed);
}

#[test]
#[should_panic]
fn test_slot_sequence_overflow() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(ChronoPayContract, ());
    let client = ChronoPayContractClient::new(&env, &contract_id);

    env.storage()
        .instance()
        .set(&DataKey::SlotSeq, &u32::MAX);

    client.create_time_slot(
        &String::from_str(&env, "pro"),
        &1000u64,
        &2000u64,
    );
}

#[test]
fn test_buy_time_token_sets_owner() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(ChronoPayContract, ());
    let client = ChronoPayContractClient::new(&env, &contract_id);

    let token = Symbol::new(&env, "TIME_TOKEN");
    let buyer = String::from_str(&env, "buyer");
    let seller = String::from_str(&env, "seller");

    let success = client.buy_time_token(&token, &buyer, &seller);
    assert!(success);

    let owner: String = env.storage().instance().get(&DataKey::Owner).unwrap();
    assert_eq!(owner, buyer);
}

#[test]
#[should_panic]
fn test_create_slot_when_paused() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);

    let contract_id = env.register(ChronoPayContract, ());
    let client = ChronoPayContractClient::new(&env, &contract_id);

    client.initialize(admin.clone());

    client.pause();

    client.create_time_slot(
        &String::from_str(&env, "pro"),
        &1000u64,
        &2000u64,
    ); // should panic
}

#[test]
fn test_unpause_restores_functionality() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);

    let contract_id = env.register(ChronoPayContract, ());
    let client = ChronoPayContractClient::new(&env, &contract_id);

    client.initialize(admin.clone());

    client.pause();
    client.unpause();

    let slot_id = client.create_time_slot(
        &String::from_str(&env, "pro"),
        &1000u64,
        &2000u64,
    );
    assert_eq!(slot_id, 1);
}
