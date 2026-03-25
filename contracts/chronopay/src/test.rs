#![cfg(test)]

use super::*;
use soroban_sdk::{vec, Address, Env, String};

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
fn test_create_time_slot_rejects_zero_length_range() {
    let env = Env::default();
    let contract_id = env.register(ChronoPayContract, ());
    let client = ChronoPayContractClient::new(&env, &contract_id);

    let result = client.try_create_time_slot(
        &String::from_str(&env, "professional_alice"),
        &1000u64,
        &1000u64,
    );

    assert_eq!(result, Err(Ok(ContractError::InvalidTimeRange)));
}

#[test]
fn test_create_time_slot_rejects_inverted_range() {
    let env = Env::default();
    let contract_id = env.register(ChronoPayContract, ());
    let client = ChronoPayContractClient::new(&env, &contract_id);

    let result = client.try_create_time_slot(
        &String::from_str(&env, "professional_alice"),
        &2000u64,
        &1000u64,
    );

    assert_eq!(result, Err(Ok(ContractError::InvalidTimeRange)));
}

#[test]
fn test_create_time_slot_rejects_slot_sequence_overflow() {
    let env = Env::default();
    let contract_id = env.register(ChronoPayContract, ());
    env.as_contract(&contract_id, || {
        env.storage().instance().set(&DataKey::SlotSeq, &u32::MAX);
    });

    let client = ChronoPayContractClient::new(&env, &contract_id);
    let result = client.try_create_time_slot(
        &String::from_str(&env, "professional_alice"),
        &1000u64,
        &2000u64,
    );

    assert_eq!(result, Err(Ok(ContractError::SlotIdOverflow)));
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
fn test_buy_time_token_sets_owner_to_contract_address() {
    let env = Env::default();
    let contract_id = env.register(ChronoPayContract, ());
    let client = ChronoPayContractClient::new(&env, &contract_id);

    let bought = client.buy_time_token(
        &soroban_sdk::Symbol::new(&env, "TIME_TOKEN"),
        &String::from_str(&env, "buyer"),
        &String::from_str(&env, "seller"),
    );

    assert!(bought);
    let stored_owner: Address = env
        .as_contract(&contract_id, || {
            env.storage().instance().get(&DataKey::Owner)
        })
        .unwrap();
    assert_eq!(stored_owner, contract_id);
}
