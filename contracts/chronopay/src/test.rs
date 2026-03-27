#![cfg(test)]

use super::*;
use soroban_sdk::{vec, Env, String};

fn setup(env: &Env) -> ChronoPayContractClient<'_> {
    let id = env.register(ChronoPayContract, ());
    ChronoPayContractClient::new(env, &id)
}

#[test]
fn test_hello() {
    let env = Env::default();
    let client = setup(&env);

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
    let client = setup(&env);

    let id1 = client.create_time_slot(
        &String::from_str(&env, "professional_alice"),
        &1000u64,
        &2000u64,
    );
    let id2 = client.create_time_slot(
        &String::from_str(&env, "professional_alice"),
        &3000u64,
        &4000u64,
    );
    let id3 = client.create_time_slot(
        &String::from_str(&env, "professional_alice"),
        &5000u64,
        &6000u64,
    );

    assert_eq!(id1, 1);
    assert_eq!(id2, 2);
    assert_eq!(id3, 3);
}

#[test]
fn test_create_time_slot_stores_slot_info() {
    let env = Env::default();
    let client = setup(&env);

    let id = client.create_time_slot(&String::from_str(&env, "carol"), &100u64, &200u64);
    let info = client.get_slot_info(&id);

    assert_eq!(info.professional, String::from_str(&env, "carol"));
    assert_eq!(info.start_time, 100u64);
    assert_eq!(info.end_time, 200u64);
}

#[test]
fn test_create_time_slot_initial_status_is_available() {
    let env = Env::default();
    let client = setup(&env);

    let id = client.create_time_slot(&String::from_str(&env, "dave"), &1u64, &2u64);
    assert_eq!(client.get_slot_status(&id), TimeTokenStatus::Available);
}

#[test]
fn test_create_time_slot_multiple_slots_store_independently() {
    let env = Env::default();
    let client = setup(&env);

    let id1 = client.create_time_slot(&String::from_str(&env, "alice"), &100u64, &200u64);
    let id2 = client.create_time_slot(&String::from_str(&env, "bob"), &300u64, &400u64);

    let info1 = client.get_slot_info(&id1);
    let info2 = client.get_slot_info(&id2);

    assert_eq!(info1.professional, String::from_str(&env, "alice"));
    assert_eq!(info1.start_time, 100u64);
    assert_eq!(info2.professional, String::from_str(&env, "bob"));
    assert_eq!(info2.start_time, 300u64);
}

#[test]
#[should_panic]
fn test_create_time_slot_rejects_equal_times() {
    let env = Env::default();
    let client = setup(&env);
    client.create_time_slot(&String::from_str(&env, "eve"), &1000u64, &1000u64);
}

#[test]
#[should_panic]
fn test_create_time_slot_rejects_reversed_times() {
    let env = Env::default();
    let client = setup(&env);
    client.create_time_slot(&String::from_str(&env, "eve"), &2000u64, &1000u64);
}

#[test]
fn test_mint_time_token_returns_symbol() {
    let env = Env::default();
    let client = setup(&env);

    let id = client.create_time_slot(&String::from_str(&env, "frank"), &10u64, &20u64);
    let token = client.mint_time_token(&id);
    assert_eq!(token, soroban_sdk::Symbol::new(&env, "TIME_TOKEN"));
}

#[test]
#[should_panic]
fn test_mint_time_token_rejects_nonexistent_slot() {
    let env = Env::default();
    let client = setup(&env);
    client.mint_time_token(&999u32);
}

#[test]
fn test_buy_time_token_sets_owner_and_status() {
    let env = Env::default();
    let client = setup(&env);

    let id = client.create_time_slot(&String::from_str(&env, "grace"), &10u64, &20u64);
    let ok = client.buy_time_token(
        &id,
        &String::from_str(&env, "buyer_1"),
        &String::from_str(&env, "grace"),
    );

    assert!(ok);
    assert_eq!(client.get_slot_status(&id), TimeTokenStatus::Sold);
    assert_eq!(
        client.get_slot_owner(&id),
        String::from_str(&env, "buyer_1")
    );
}

#[test]
#[should_panic]
fn test_buy_time_token_rejects_nonexistent_slot() {
    let env = Env::default();
    let client = setup(&env);
    client.buy_time_token(
        &999u32,
        &String::from_str(&env, "buyer"),
        &String::from_str(&env, "seller"),
    );
}

#[test]
#[should_panic]
fn test_buy_time_token_rejects_double_purchase() {
    let env = Env::default();
    let client = setup(&env);

    let id = client.create_time_slot(&String::from_str(&env, "h"), &10u64, &20u64);
    client.buy_time_token(
        &id,
        &String::from_str(&env, "buyer1"),
        &String::from_str(&env, "h"),
    );
    client.buy_time_token(
        &id,
        &String::from_str(&env, "buyer2"),
        &String::from_str(&env, "buyer1"),
    );
}

#[test]
fn test_redeem_time_token_sets_redeemed_status() {
    let env = Env::default();
    let client = setup(&env);

    let id = client.create_time_slot(&String::from_str(&env, "ivan"), &10u64, &20u64);
    client.buy_time_token(
        &id,
        &String::from_str(&env, "buyer"),
        &String::from_str(&env, "ivan"),
    );

    let ok = client.redeem_time_token(&id);
    assert!(ok);
    assert_eq!(client.get_slot_status(&id), TimeTokenStatus::Redeemed);
}

#[test]
#[should_panic]
fn test_redeem_time_token_rejects_nonexistent_slot() {
    let env = Env::default();
    let client = setup(&env);
    client.redeem_time_token(&999u32);
}

#[test]
#[should_panic]
fn test_redeem_time_token_rejects_available_slot() {
    let env = Env::default();
    let client = setup(&env);
    let id = client.create_time_slot(&String::from_str(&env, "judy"), &10u64, &20u64);
    client.redeem_time_token(&id);
}

#[test]
#[should_panic]
fn test_redeem_time_token_rejects_double_redeem() {
    let env = Env::default();
    let client = setup(&env);
    let id = client.create_time_slot(&String::from_str(&env, "kate"), &10u64, &20u64);
    client.buy_time_token(
        &id,
        &String::from_str(&env, "buyer"),
        &String::from_str(&env, "kate"),
    );
    client.redeem_time_token(&id);
    client.redeem_time_token(&id);
}

#[test]
fn test_full_lifecycle() {
    let env = Env::default();
    let client = setup(&env);

    let id = client.create_time_slot(&String::from_str(&env, "lena"), &1000u64, &2000u64);
    assert_eq!(client.get_slot_status(&id), TimeTokenStatus::Available);

    let token = client.mint_time_token(&id);
    assert_eq!(token, soroban_sdk::Symbol::new(&env, "TIME_TOKEN"));

    client.buy_time_token(
        &id,
        &String::from_str(&env, "mike"),
        &String::from_str(&env, "lena"),
    );
    assert_eq!(client.get_slot_status(&id), TimeTokenStatus::Sold);
    assert_eq!(client.get_slot_owner(&id), String::from_str(&env, "mike"));

    client.redeem_time_token(&id);
    assert_eq!(client.get_slot_status(&id), TimeTokenStatus::Redeemed);
}

#[test]
fn test_get_slot_owner_returns_empty_before_purchase() {
    let env = Env::default();
    let client = setup(&env);
    let id = client.create_time_slot(&String::from_str(&env, "nora"), &10u64, &20u64);
    assert_eq!(client.get_slot_owner(&id), String::from_str(&env, ""));
}

#[test]
#[should_panic]
fn test_get_slot_info_rejects_nonexistent_slot() {
    let env = Env::default();
    let client = setup(&env);
    client.get_slot_info(&0u32);
}

#[test]
#[should_panic]
fn test_get_slot_status_rejects_nonexistent_slot() {
    let env = Env::default();
    let client = setup(&env);
    client.get_slot_status(&0u32);
}

#[test]
#[should_panic]
fn test_get_slot_owner_rejects_nonexistent_slot() {
    let env = Env::default();
    let client = setup(&env);
    client.get_slot_owner(&0u32);
}

#[test]
fn test_storage_keys_are_isolated_per_slot() {
    let env = Env::default();
    let client = setup(&env);

    let id1 = client.create_time_slot(&String::from_str(&env, "pro1"), &100u64, &200u64);
    let id2 = client.create_time_slot(&String::from_str(&env, "pro2"), &300u64, &400u64);

    client.buy_time_token(
        &id1,
        &String::from_str(&env, "buyer"),
        &String::from_str(&env, "pro1"),
    );

    assert_eq!(client.get_slot_status(&id1), TimeTokenStatus::Sold);
    assert_eq!(client.get_slot_status(&id2), TimeTokenStatus::Available);
    assert_eq!(client.get_slot_owner(&id2), String::from_str(&env, ""));
}

#[test]
fn test_mint_and_redeem() {
    let env = Env::default();
    let contract_id = env.register(ChronoPayContract, ());
    let client = ChronoPayContractClient::new(&env, &contract_id);

    let slot_id = client.create_time_slot(&String::from_str(&env, "pro"), &1000u64, &2000u64);
    let token = client.mint_time_token(&slot_id);
    assert_eq!(token, soroban_sdk::Symbol::new(&env, "TIME_TOKEN"));

    client.buy_time_token(
        &slot_id,
        &String::from_str(&env, "buyer"),
        &String::from_str(&env, "pro"),
    );
    let redeemed = client.redeem_time_token(&slot_id);
    assert!(redeemed);
}
