#![cfg(test)]

use super::*;
use soroban_sdk::{vec, Env, String, Symbol};

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

    let status = env.as_contract(&contract_id, || {
        env.storage()
            .instance()
            .get::<DataKey, TimeTokenStatus>(&DataKey::Status)
    });
    assert_eq!(status, Some(TimeTokenStatus::Redeemed));
}

#[test]
fn test_buy_time_token_persists_contract_owner_key() {
    let env = Env::default();
    let contract_id = env.register(ChronoPayContract, ());
    let client = ChronoPayContractClient::new(&env, &contract_id);
    let token_id = Symbol::new(&env, "TIME_TOKEN");

    let purchased = client.buy_time_token(
        &token_id,
        &String::from_str(&env, "buyer_bob"),
        &String::from_str(&env, "seller_alice"),
    );

    assert!(purchased);

    let owner = env.as_contract(&contract_id, || {
        env.storage().instance().get(&DataKey::Owner)
    });
    assert_eq!(owner, Some(contract_id));
}

#[test]
#[should_panic(expected = "slot id overflow")]
fn test_create_time_slot_panics_on_slot_id_overflow() {
    let env = Env::default();
    let contract_id = env.register(ChronoPayContract, ());
    let client = ChronoPayContractClient::new(&env, &contract_id);

    env.as_contract(&contract_id, || {
        env.storage().instance().set(&DataKey::SlotSeq, &u32::MAX);
    });

    let _ = client.create_time_slot(
        &String::from_str(&env, "professional_alice"),
        &1000u64,
        &2000u64,
    );
}

#[test]
fn test_domain_types_are_equatable_and_distinct() {
    assert_eq!(TimeTokenStatus::Available, TimeTokenStatus::Available);
    assert_ne!(TimeTokenStatus::Available, TimeTokenStatus::Redeemed);
    assert_eq!(DataKey::SlotSeq, DataKey::SlotSeq);
    assert_ne!(DataKey::Owner, DataKey::Status);
}
