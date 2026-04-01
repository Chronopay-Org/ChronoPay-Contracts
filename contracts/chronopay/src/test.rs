#![cfg(test)]

use super::*;
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{vec, Address, Env, String, Symbol};

fn setup() -> Env {
    Env::default()
}

#[test]
fn test_hello() {
    let env = setup();
    let contract_id = env.register(ChronoPayContract, ());
    let client = ChronoPayContractClient::new(&env, &contract_id);

// ── hello ─────────────────────────────────────────────────────────────────────

#[test]
fn test_hello() {
    let (env, client) = setup();
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

// ── create_time_slot: happy paths ─────────────────────────────────────────────

#[test]
fn test_initialize_and_metadata() {
    let env = setup();
    let contract_id = env.register(ChronoPayContract, ());
    let client = ChronoPayContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let name = String::from_str(&env, "ChronoPay Time Tokens");
    let symbol = String::from_str(&env, "TIME");

    client.initialize(&admin, &name, &symbol);

    let metadata = client
        .get_collection_metadata()
        .expect("metadata should exist");
    assert_eq!(metadata.name, name);
    assert_eq!(metadata.symbol, symbol);
}

#[test]
#[should_panic(expected = "already initialized")]
fn test_initialize_twice_panics() {
    let env = setup();
    let contract_id = env.register(ChronoPayContract, ());
    let client = ChronoPayContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let name = String::from_str(&env, "Name");
    let symbol = String::from_str(&env, "SYM");

    client.initialize(&admin, &name, &symbol);
    client.initialize(&admin, &name, &symbol);
}

#[test]
fn test_create_time_slot_persists() {
    let env = setup();
    env.mock_all_auths();
    let contract_id = env.register(ChronoPayContract, ());
    let client = ChronoPayContractClient::new(&env, &contract_id);

    let professional = Address::generate(&env);
    let slot_id = client.create_time_slot(&professional, &1_000u64, &2_000u64);
    assert_eq!(slot_id, 1);

    let slot = client.get_time_slot(&slot_id).expect("slot should exist");
    assert_eq!(slot.professional, professional);
    assert_eq!(slot.start_time, 1_000u64);
    assert_eq!(slot.end_time, 2_000u64);
    assert!(slot.token.is_none());
}

#[test]
fn test_create_time_slot_auto_increments() {
    let env = setup();
    env.mock_all_auths();
    let contract_id = env.register(ChronoPayContract, ());
    let client = ChronoPayContractClient::new(&env, &contract_id);

    let prof = Address::generate(&env);

    let slot_id_1 = client.create_time_slot(&prof, &1000u64, &2000u64);
    let slot_id_2 = client.create_time_slot(&prof, &3000u64, &4000u64);
    let slot_id_3 = client.create_time_slot(&prof, &5000u64, &6000u64);

    assert_eq!(slot_id_1, 1);
    assert_eq!(slot_id_2, 2);

    // Query existing slot (From feature/sc-038 logic)
    let slot = client.get_time_slot(&slot_id_1).expect("slot should exist");
    assert_eq!(slot.professional, professional);
    assert_eq!(slot.start_time, 1000u64);
    assert_eq!(slot.end_time, 2000u64);
    assert!(slot.token.is_none());

    // Query non-existent slot
    let non_existent = client.get_time_slot(&999u32);
    assert!(non_existent.is_none());
}

#[test]
#[should_panic(expected = "end_time must be after start_time")]
fn test_create_time_slot_rejects_invalid_times() {
    let env = setup();
    env.mock_all_auths();
    let contract_id = env.register(ChronoPayContract, ());
    let client = ChronoPayContractClient::new(&env, &contract_id);
    let professional = Address::generate(&env);
    let _ = client.create_time_slot(&professional, &10u64, &10u64);
}

#[test]
fn test_mint_buy_redeem_lifecycle() {
    let env = setup();
    env.mock_all_auths();
    let contract_id = env.register(ChronoPayContract, ());
    let client = ChronoPayContractClient::new(&env, &contract_id);

    let professional = Address::generate(&env);
    let slot_id = client.create_time_slot(&professional, &100u64, &200u64);

    let token_metadata = TokenMetadata {
        name: String::from_str(&env, "Consultation #1"),
        description: String::from_str(&env, "Expert consultation"),
        image_uri: String::from_str(&env, "ipfs://hash"),
    };

    let token = client.mint_time_token(&slot_id, &token_metadata);
    assert_eq!(token, Symbol::new(&env, "TIME_1"));

    // metadata after mint
    let metadata = client
        .get_token_metadata(&token)
        .expect("metadata should exist");
    assert_eq!(metadata.slot_id, slot_id);
    assert_eq!(metadata.status, TimeTokenStatus::Available);
    assert_eq!(metadata.current_owner, professional);
    assert_eq!(metadata.metadata.name, token_metadata.name);

    // buy / transfer
    let buyer = Address::generate(&env);
    let purchased = client.buy_time_token(&token, &buyer);
    assert!(purchased);

    let metadata_after_buy = client.get_token_metadata(&token).unwrap();
    assert_eq!(metadata_after_buy.status, TimeTokenStatus::Sold);
    assert_eq!(metadata_after_buy.current_owner, buyer);

    // redeem
    let redeemed = client.redeem_time_token(&token);
    assert!(redeemed);
    let metadata_after_redeem = client.get_token_metadata(&token).unwrap();
    assert_eq!(metadata_after_redeem.status, TimeTokenStatus::Redeemed);
}

#[test]
#[should_panic(expected = "token already minted for slot")]
fn test_mint_twice_panics() {
    let env = setup();
    env.mock_all_auths();
    let contract_id = env.register(ChronoPayContract, ());
    let client = ChronoPayContractClient::new(&env, &contract_id);

    let professional = Address::generate(&env);
    let slot_id = client.create_time_slot(&professional, &10u64, &20u64);

    let token_metadata = TokenMetadata {
        name: String::from_str(&env, "T"),
        description: String::from_str(&env, "D"),
        image_uri: String::from_str(&env, "I"),
    };

    let _ = client.mint_time_token(&slot_id, &token_metadata);
    let _ = client.mint_time_token(&slot_id, &token_metadata);
}

#[test]
#[should_panic(expected = "token already redeemed")]
fn test_buy_redeemed_panics() {
    let env = setup();
    env.mock_all_auths();
    let contract_id = env.register(ChronoPayContract, ());
    let client = ChronoPayContractClient::new(&env, &contract_id);

    let professional = Address::generate(&env);
    let slot_id = client.create_time_slot(&professional, &10u64, &20u64);

    let token_metadata = TokenMetadata {
        name: String::from_str(&env, "T"),
        description: String::from_str(&env, "D"),
        image_uri: String::from_str(&env, "I"),
    };

    let token = client.mint_time_token(&slot_id, &token_metadata);
    let buyer = Address::generate(&env);
    let _ = client.buy_time_token(&token, &buyer);
    let _ = client.redeem_time_token(&token);

    // Buying again after redemption should fail
    let buyer2 = Address::generate(&env);
    let _ = client.buy_time_token(&token, &buyer2);
}

#[test]
#[should_panic(expected = "token already redeemed")]
fn test_redeem_twice_panics() {
    let env = setup();
    env.mock_all_auths();
    let contract_id = env.register(ChronoPayContract, ());
    let client = ChronoPayContractClient::new(&env, &contract_id);

    let professional = Address::generate(&env);
    let slot_id = client.create_time_slot(&professional, &10u64, &20u64);

    let token_metadata = TokenMetadata {
        name: String::from_str(&env, "T"),
        description: String::from_str(&env, "D"),
        image_uri: String::from_str(&env, "I"),
    };

    let token = client.mint_time_token(&slot_id, &token_metadata);
    let _ = client.redeem_time_token(&token);
    let _ = client.redeem_time_token(&token);
}

#[test]
#[should_panic(expected = "buyer is already the owner")]
fn test_buy_requires_distinct_parties() {
    let env = setup();
    env.mock_all_auths();
    let contract_id = env.register(ChronoPayContract, ());
    let client = ChronoPayContractClient::new(&env, &contract_id);

    let professional = Address::generate(&env);
    let slot_id = client.create_time_slot(&professional, &10u64, &20u64);

    let token_metadata = TokenMetadata {
        name: String::from_str(&env, "T"),
        description: String::from_str(&env, "D"),
        image_uri: String::from_str(&env, "I"),
    };

    let token = client.mint_time_token(&slot_id, &token_metadata);
    let _ = client.buy_time_token(&token, &professional);
}
