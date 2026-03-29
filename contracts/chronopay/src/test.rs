#![cfg(test)]

use super::*;
use soroban_sdk::{vec, Env, String};

// ---------------------------------------------------------------------------
// Existing / regression tests
// ---------------------------------------------------------------------------

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

    let id1 = client.create_time_slot(&String::from_str(&env, "alice"), &1000u64, &2000u64);
    let id2 = client.create_time_slot(&String::from_str(&env, "alice"), &3000u64, &4000u64);
    let id3 = client.create_time_slot(&String::from_str(&env, "alice"), &5000u64, &6000u64);

    assert_eq!(id1, 1);
    assert_eq!(id2, 2);
    assert_eq!(id3, 3);
}

#[test]
fn test_mint_time_token() {
    let env = Env::default();
    let contract_id = env.register(ChronoPayContract, ());
    let client = ChronoPayContractClient::new(&env, &contract_id);

    let id = client.create_time_slot(&String::from_str(&env, "pro"), &1000u64, &2000u64);
    let token = client.mint_time_token(&id);
    assert_eq!(token, soroban_sdk::Symbol::new(&env, "TIME_TOKEN"));
}

// ---------------------------------------------------------------------------
// State machine — happy path
// ---------------------------------------------------------------------------

#[test]
fn test_initial_status_is_available() {
    let env = Env::default();
    let contract_id = env.register(ChronoPayContract, ());
    let client = ChronoPayContractClient::new(&env, &contract_id);

    let id = client.create_time_slot(&String::from_str(&env, "alice"), &0u64, &100u64);
    assert_eq!(client.get_slot_status(&id), SlotStatus::Available);
}

#[test]
fn test_full_transition_chain() {
    // Available → Sold → Redeemed
    let env = Env::default();
    let contract_id = env.register(ChronoPayContract, ());
    let client = ChronoPayContractClient::new(&env, &contract_id);

    let id = client.create_time_slot(&String::from_str(&env, "bob"), &0u64, &100u64);

    assert_eq!(client.get_slot_status(&id), SlotStatus::Available);

    client.sell_slot(&id);
    assert_eq!(client.get_slot_status(&id), SlotStatus::Sold);

    client.redeem_slot(&id);
    assert_eq!(client.get_slot_status(&id), SlotStatus::Redeemed);
}

#[test]
fn test_independent_slots_do_not_interfere() {
    // Each slot carries its own independent state.
    let env = Env::default();
    let contract_id = env.register(ChronoPayContract, ());
    let client = ChronoPayContractClient::new(&env, &contract_id);

    let id1 = client.create_time_slot(&String::from_str(&env, "alice"), &0u64, &100u64);
    let id2 = client.create_time_slot(&String::from_str(&env, "bob"), &200u64, &300u64);

    // Advance only id1.
    client.sell_slot(&id1);
    client.redeem_slot(&id1);

    // id2 must still be Available.
    assert_eq!(client.get_slot_status(&id2), SlotStatus::Available);
}

// ---------------------------------------------------------------------------
// State machine — illegal transitions
// ---------------------------------------------------------------------------

#[test]
#[should_panic(expected = "HostError: Error(Contract, #2)")]
fn test_sell_already_sold_slot() {
    let env = Env::default();
    let contract_id = env.register(ChronoPayContract, ());
    let client = ChronoPayContractClient::new(&env, &contract_id);

    let id = client.create_time_slot(&String::from_str(&env, "alice"), &0u64, &100u64);
    client.sell_slot(&id);
    // Second sell must panic with InvalidTransition (#2).
    client.sell_slot(&id);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #2)")]
fn test_redeem_available_slot() {
    // Cannot skip directly from Available to Redeemed.
    let env = Env::default();
    let contract_id = env.register(ChronoPayContract, ());
    let client = ChronoPayContractClient::new(&env, &contract_id);

    let id = client.create_time_slot(&String::from_str(&env, "alice"), &0u64, &100u64);
    client.redeem_slot(&id);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #2)")]
fn test_sell_redeemed_slot() {
    // Terminal state: cannot sell after redemption.
    let env = Env::default();
    let contract_id = env.register(ChronoPayContract, ());
    let client = ChronoPayContractClient::new(&env, &contract_id);

    let id = client.create_time_slot(&String::from_str(&env, "alice"), &0u64, &100u64);
    client.sell_slot(&id);
    client.redeem_slot(&id);
    client.sell_slot(&id);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #2)")]
fn test_redeem_redeemed_slot() {
    // Terminal state: cannot redeem twice.
    let env = Env::default();
    let contract_id = env.register(ChronoPayContract, ());
    let client = ChronoPayContractClient::new(&env, &contract_id);

    let id = client.create_time_slot(&String::from_str(&env, "alice"), &0u64, &100u64);
    client.sell_slot(&id);
    client.redeem_slot(&id);
    client.redeem_slot(&id);
}

// ---------------------------------------------------------------------------
// State machine — missing slot (SlotNotFound)
// ---------------------------------------------------------------------------

#[test]
#[should_panic(expected = "HostError: Error(Contract, #1)")]
fn test_get_status_unknown_slot() {
    let env = Env::default();
    let contract_id = env.register(ChronoPayContract, ());
    let client = ChronoPayContractClient::new(&env, &contract_id);
    client.get_slot_status(&999);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #1)")]
fn test_sell_unknown_slot() {
    let env = Env::default();
    let contract_id = env.register(ChronoPayContract, ());
    let client = ChronoPayContractClient::new(&env, &contract_id);
    client.sell_slot(&999);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #1)")]
fn test_redeem_unknown_slot() {
    let env = Env::default();
    let contract_id = env.register(ChronoPayContract, ());
    let client = ChronoPayContractClient::new(&env, &contract_id);
    client.redeem_slot(&999);
}
