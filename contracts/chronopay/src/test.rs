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

    // Default rate is 0%, thus no fees distributed
    let result = client.redeem_time_token(&token, &100_000_i128);
    assert_eq!(result.professional_amount, 100_000);
    assert_eq!(result.protocol_fee, 0);
}

#[test]
fn test_fee_distribution_splits() {
    let env = Env::default();
    let contract_id = env.register(ChronoPayContract, ());
    let client = ChronoPayContractClient::new(&env, &contract_id);

    // Context: Setting the protocol fee to 500 basis points (5%)
    client.set_fee_rate(&500);
    assert_eq!(client.get_fee_rate(), 500);

    let token = soroban_sdk::Symbol::new(&env, "TOKEN");

    // Settlement of 10,000 should split into 9,500 and 500
    let result = client.redeem_time_token(&token, &10_000_i128);
    assert_eq!(result.protocol_fee, 500);
    assert_eq!(result.professional_amount, 9_500);

    // Changing fee to 12.5%
    client.set_fee_rate(&1250);
    assert_eq!(client.get_fee_rate(), 1250);

    // Settlement of 1,000,000
    let result2 = client.redeem_time_token(&token, &1_000_000_i128);
    assert_eq!(result2.protocol_fee, 125_000);
    assert_eq!(result2.professional_amount, 875_000);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #1)")]
fn test_invalid_fee_rate_bounded() {
    let env = Env::default();
    let contract_id = env.register(ChronoPayContract, ());
    let client = ChronoPayContractClient::new(&env, &contract_id);

    // 10001 > 10000 limit, should be rejected cleanly
    client.set_fee_rate(&10001);
}

#[test]
#[should_panic(expected = "settlement amount cannot be negative")]
fn test_negative_settlement_blocked() {
    let env = Env::default();
    let contract_id = env.register(ChronoPayContract, ());
    let client = ChronoPayContractClient::new(&env, &contract_id);
    let token = soroban_sdk::Symbol::new(&env, "TOKEN");

    // Should panic due to negative amount
    client.redeem_time_token(&token, &-500_i128);
}
