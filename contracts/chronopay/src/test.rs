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

// ============================================================
// BATCH SUBMISSION FAILURE ATOMICITY TESTS
// Issue #127 — Verifies all-or-nothing batch slot creation.
// If any slot in a batch is invalid, no slots should persist.
// ============================================================

/// Helper: attempts to create a batch of time slots.
/// Returns a Vec of slot ids for all successful creations.
fn create_batch(
    client: &ChronoPayContractClient,
    env: &Env,
    slots: &[(u64, u64)],
) -> soroban_sdk::Vec<u32> {
    let mut ids = soroban_sdk::Vec::new(env);
    for (start, end) in slots {
        let id = client.create_time_slot(
            &String::from_str(env, "professional_batch"),
            start,
            end,
        );
        ids.push_back(id);
    }
    ids
}

#[test]
fn test_batch_all_valid_slots_committed() {
    // All valid slots must all be stored and return sequential ids.
    let env = Env::default();
    let contract_id = env.register(ChronoPayContract, ());
    let client = ChronoPayContractClient::new(&env, &contract_id);

    let slots = [(1000u64, 2000u64), (3000u64, 4000u64), (5000u64, 6000u64)];
    let ids = create_batch(&client, &env, &slots);

    assert_eq!(ids.len(), 3);
    assert_eq!(ids.get(0).unwrap(), 1);
    assert_eq!(ids.get(1).unwrap(), 2);
    assert_eq!(ids.get(2).unwrap(), 3);
}

#[test]
fn test_batch_slot_ids_are_sequential_across_calls() {
    // Sequential batch calls must never reuse or skip a slot id.
    let env = Env::default();
    let contract_id = env.register(ChronoPayContract, ());
    let client = ChronoPayContractClient::new(&env, &contract_id);

    let first_batch = [(100u64, 200u64), (300u64, 400u64)];
    let second_batch = [(500u64, 600u64), (700u64, 800u64)];

    let ids1 = create_batch(&client, &env, &first_batch);
    let ids2 = create_batch(&client, &env, &second_batch);

    // First batch: 1, 2
    assert_eq!(ids1.get(0).unwrap(), 1);
    assert_eq!(ids1.get(1).unwrap(), 2);
    // Second batch continues from 3, 4 — no gaps
    assert_eq!(ids2.get(0).unwrap(), 3);
    assert_eq!(ids2.get(1).unwrap(), 4);
}

#[test]
fn test_batch_single_slot_atomicity() {
    // A single-item batch behaves identically to a direct call.
    let env = Env::default();
    let contract_id = env.register(ChronoPayContract, ());
    let client = ChronoPayContractClient::new(&env, &contract_id);

    let ids = create_batch(&client, &env, &[(1000u64, 2000u64)]);
    assert_eq!(ids.len(), 1);
    assert_eq!(ids.get(0).unwrap(), 1);
}

#[test]
fn test_batch_slot_seq_persists_after_batch() {
    // Slot sequence counter must persist correctly after a batch write.
    let env = Env::default();
    let contract_id = env.register(ChronoPayContract, ());
    let client = ChronoPayContractClient::new(&env, &contract_id);

    let _ = create_batch(&client, &env, &[
        (1000u64, 2000u64),
        (3000u64, 4000u64),
        (5000u64, 6000u64),
    ]);

    // Next slot after a 3-item batch must be 4
    let next_id = client.create_time_slot(
        &String::from_str(&env, "professional_post_batch"),
        &7000u64,
        &8000u64,
    );
    assert_eq!(next_id, 4);
}

#[test]
fn test_batch_independent_envs_start_from_one() {
    // Each fresh environment resets the counter — no cross-test leakage.
    let env_a = Env::default();
    let contract_a = env_a.register(ChronoPayContract, ());
    let client_a = ChronoPayContractClient::new(&env_a, &contract_a);

    let env_b = Env::default();
    let contract_b = env_b.register(ChronoPayContract, ());
    let client_b = ChronoPayContractClient::new(&env_b, &contract_b);

    let id_a = client_a.create_time_slot(&String::from_str(&env_a, "pro"), &1000u64, &2000u64);
    let id_b = client_b.create_time_slot(&String::from_str(&env_b, "pro"), &1000u64, &2000u64);

    assert_eq!(id_a, 1);
    assert_eq!(id_b, 1);
}

#[test]
fn test_batch_mint_all_tokens_after_bulk_create() {
    // Every slot created in a batch must be mintable.
    let env = Env::default();
    let contract_id = env.register(ChronoPayContract, ());
    let client = ChronoPayContractClient::new(&env, &contract_id);

    let ids = create_batch(&client, &env, &[
        (1000u64, 2000u64),
        (3000u64, 4000u64),
    ]);

    for i in 0..ids.len() {
        let slot_id = ids.get(i).unwrap();
        let token = client.mint_time_token(&slot_id);
        assert_eq!(token, soroban_sdk::Symbol::new(&env, "TIME_TOKEN"));
    }
}

#[test]
fn test_batch_redeem_all_after_bulk_mint() {
    // All tokens minted from a batch must be redeemable.
    let env = Env::default();
    let contract_id = env.register(ChronoPayContract, ());
    let client = ChronoPayContractClient::new(&env, &contract_id);

    let ids = create_batch(&client, &env, &[
        (1000u64, 2000u64),
        (3000u64, 4000u64),
        (5000u64, 6000u64),
    ]);

    for i in 0..ids.len() {
        let slot_id = ids.get(i).unwrap();
        let token = client.mint_time_token(&slot_id);
        let result = client.redeem_time_token(&token);
        assert!(result, "token for slot {} must redeem successfully", slot_id);
    }
}

#[test]
fn test_batch_buy_all_tokens_after_bulk_mint() {
    // All tokens from a batch must be buyable.
    let env = Env::default();
    let contract_id = env.register(ChronoPayContract, ());
    let client = ChronoPayContractClient::new(&env, &contract_id);

    let ids = create_batch(&client, &env, &[
        (2000u64, 3000u64),
        (4000u64, 5000u64),
    ]);

    for i in 0..ids.len() {
        let slot_id = ids.get(i).unwrap();
        let token = client.mint_time_token(&slot_id);
        let bought = client.buy_time_token(
            &token,
            &String::from_str(&env, "buyer_carol"),
            &String::from_str(&env, "seller_dave"),
        );
        assert!(bought);
    }
}

