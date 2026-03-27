#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, vec, Address, Env, String};

fn setup(env: &Env) -> (Address, ChronoPayContractClient) {
    env.mock_all_auths();
    let contract_id = env.register(ChronoPayContract, ());
    let client = ChronoPayContractClient::new(env, &contract_id);
    let professional = Address::generate(env);
    (professional, client)
}

// ── Hello (CI sanity check) ───────────────────────────────────────────────

#[test]
fn test_hello() {
    let env = Env::default();
    let (_, client) = setup(&env);
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

// ── create_time_slot ──────────────────────────────────────────────────────

#[test]
fn test_create_time_slot_returns_id() {
    let env = Env::default();
    let (pro, client) = setup(&env);
    let id = client.create_time_slot(&pro, &1000, &2000);
    assert_eq!(id, 1);
}

#[test]
fn test_create_time_slot_auto_increments() {
    let env = Env::default();
    let (pro, client) = setup(&env);
    assert_eq!(client.create_time_slot(&pro, &1000, &2000), 1);
    assert_eq!(client.create_time_slot(&pro, &3000, &4000), 2);
    assert_eq!(client.create_time_slot(&pro, &5000, &6000), 3);
}

#[test]
fn test_create_time_slot_persists_data() {
    let env = Env::default();
    let (pro, client) = setup(&env);
    let id = client.create_time_slot(&pro, &1000, &2000);
    let slot = client.get_time_slot(&id);
    assert_eq!(slot.id, id);
    assert_eq!(slot.professional, pro);
    assert_eq!(slot.start_time, 1000);
    assert_eq!(slot.end_time, 2000);
    assert_eq!(slot.status, TimeSlotStatus::Available);
}

#[test]
fn test_create_time_slot_invalid_range_equal() {
    let env = Env::default();
    let (pro, client) = setup(&env);
    let result = client.try_create_time_slot(&pro, &1000, &1000);
    assert_eq!(result, Err(Ok(SlotError::InvalidTimeRange)));
}

#[test]
fn test_create_time_slot_invalid_range_reversed() {
    let env = Env::default();
    let (pro, client) = setup(&env);
    let result = client.try_create_time_slot(&pro, &2000, &1000);
    assert_eq!(result, Err(Ok(SlotError::InvalidTimeRange)));
}

#[test]
fn test_create_time_slot_minimum_duration() {
    let env = Env::default();
    let (pro, client) = setup(&env);
    let id = client.create_time_slot(&pro, &1000, &1001);
    let slot = client.get_time_slot(&id);
    assert_eq!(slot.end_time - slot.start_time, 1);
}

#[test]
fn test_create_time_slot_multiple_professionals() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(ChronoPayContract, ());
    let client = ChronoPayContractClient::new(&env, &contract_id);

    let alice = Address::generate(&env);
    let bob = Address::generate(&env);

    let id1 = client.create_time_slot(&alice, &1000, &2000);
    let id2 = client.create_time_slot(&bob, &3000, &4000);

    let slot1 = client.get_time_slot(&id1);
    let slot2 = client.get_time_slot(&id2);
    assert_eq!(slot1.professional, alice);
    assert_eq!(slot2.professional, bob);
}

// ── get_time_slot ─────────────────────────────────────────────────────────

#[test]
fn test_get_time_slot_not_found() {
    let env = Env::default();
    let (_, client) = setup(&env);
    let result = client.try_get_time_slot(&999);
    assert_eq!(result, Err(Ok(SlotError::SlotNotFound)));
}

// ── cancel_time_slot ──────────────────────────────────────────────────────

#[test]
fn test_cancel_time_slot_success() {
    let env = Env::default();
    let (pro, client) = setup(&env);
    let id = client.create_time_slot(&pro, &1000, &2000);
    client.cancel_time_slot(&pro, &id);
    let slot = client.get_time_slot(&id);
    assert_eq!(slot.status, TimeSlotStatus::Cancelled);
}

#[test]
fn test_cancel_time_slot_not_found() {
    let env = Env::default();
    let (pro, client) = setup(&env);
    let result = client.try_cancel_time_slot(&pro, &999);
    assert_eq!(result, Err(Ok(SlotError::SlotNotFound)));
}

#[test]
fn test_cancel_time_slot_wrong_owner() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(ChronoPayContract, ());
    let client = ChronoPayContractClient::new(&env, &contract_id);

    let alice = Address::generate(&env);
    let bob = Address::generate(&env);

    let id = client.create_time_slot(&alice, &1000, &2000);
    let result = client.try_cancel_time_slot(&bob, &id);
    assert_eq!(result, Err(Ok(SlotError::NotSlotOwner)));
}

#[test]
fn test_cancel_time_slot_already_cancelled() {
    let env = Env::default();
    let (pro, client) = setup(&env);
    let id = client.create_time_slot(&pro, &1000, &2000);
    client.cancel_time_slot(&pro, &id);
    let result = client.try_cancel_time_slot(&pro, &id);
    assert_eq!(result, Err(Ok(SlotError::AlreadyCancelled)));
}

#[test]
fn test_cancel_booked_slot_fails() {
    let env = Env::default();
    let (pro, client) = setup(&env);
    let id = client.create_time_slot(&pro, &1000, &2000);
    client.book_time_slot(&pro, &id);
    let result = client.try_cancel_time_slot(&pro, &id);
    assert_eq!(result, Err(Ok(SlotError::SlotNotAvailable)));
}

// ── book_time_slot ────────────────────────────────────────────────────────

#[test]
fn test_book_time_slot_success() {
    let env = Env::default();
    let (pro, client) = setup(&env);
    let id = client.create_time_slot(&pro, &1000, &2000);
    client.book_time_slot(&pro, &id);
    let slot = client.get_time_slot(&id);
    assert_eq!(slot.status, TimeSlotStatus::Booked);
}

#[test]
fn test_book_time_slot_not_found() {
    let env = Env::default();
    let (pro, client) = setup(&env);
    let result = client.try_book_time_slot(&pro, &999);
    assert_eq!(result, Err(Ok(SlotError::SlotNotFound)));
}

#[test]
fn test_book_time_slot_wrong_owner() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(ChronoPayContract, ());
    let client = ChronoPayContractClient::new(&env, &contract_id);

    let alice = Address::generate(&env);
    let bob = Address::generate(&env);

    let id = client.create_time_slot(&alice, &1000, &2000);
    let result = client.try_book_time_slot(&bob, &id);
    assert_eq!(result, Err(Ok(SlotError::NotSlotOwner)));
}

#[test]
fn test_book_cancelled_slot_fails() {
    let env = Env::default();
    let (pro, client) = setup(&env);
    let id = client.create_time_slot(&pro, &1000, &2000);
    client.cancel_time_slot(&pro, &id);
    let result = client.try_book_time_slot(&pro, &id);
    assert_eq!(result, Err(Ok(SlotError::SlotNotAvailable)));
}

#[test]
fn test_double_book_fails() {
    let env = Env::default();
    let (pro, client) = setup(&env);
    let id = client.create_time_slot(&pro, &1000, &2000);
    client.book_time_slot(&pro, &id);
    let result = client.try_book_time_slot(&pro, &id);
    assert_eq!(result, Err(Ok(SlotError::SlotNotAvailable)));
}

// ── get_slots_by_professional ─────────────────────────────────────────────

#[test]
fn test_get_slots_by_professional() {
    let env = Env::default();
    let (pro, client) = setup(&env);
    client.create_time_slot(&pro, &1000, &2000);
    client.create_time_slot(&pro, &3000, &4000);
    let ids = client.get_slots_by_professional(&pro);
    assert_eq!(ids.len(), 2);
    assert_eq!(ids.get(0).unwrap(), 1);
    assert_eq!(ids.get(1).unwrap(), 2);
}

#[test]
fn test_get_slots_by_professional_empty() {
    let env = Env::default();
    let (pro, client) = setup(&env);
    let ids = client.get_slots_by_professional(&pro);
    assert_eq!(ids.len(), 0);
}

#[test]
fn test_get_slots_by_professional_isolation() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(ChronoPayContract, ());
    let client = ChronoPayContractClient::new(&env, &contract_id);

    let alice = Address::generate(&env);
    let bob = Address::generate(&env);

    client.create_time_slot(&alice, &1000, &2000);
    client.create_time_slot(&bob, &3000, &4000);
    client.create_time_slot(&alice, &5000, &6000);

    assert_eq!(client.get_slots_by_professional(&alice).len(), 2);
    assert_eq!(client.get_slots_by_professional(&bob).len(), 1);
}

// ── get_slot_count ────────────────────────────────────────────────────────

#[test]
fn test_get_slot_count() {
    let env = Env::default();
    let (pro, client) = setup(&env);
    assert_eq!(client.get_slot_count(), 0);
    client.create_time_slot(&pro, &1000, &2000);
    assert_eq!(client.get_slot_count(), 1);
    client.create_time_slot(&pro, &3000, &4000);
    assert_eq!(client.get_slot_count(), 2);
}

// ── Legacy stubs ──────────────────────────────────────────────────────────

#[test]
fn test_mint_time_token_stub() {
    let env = Env::default();
    let (pro, client) = setup(&env);
    let id = client.create_time_slot(&pro, &1000, &2000);
    let token = client.mint_time_token(&id);
    assert_eq!(token, soroban_sdk::Symbol::new(&env, "TIME_TOKEN"));
}

#[test]
fn test_redeem_time_token_stub() {
    let env = Env::default();
    let (_, client) = setup(&env);
    let result = client.redeem_time_token(&soroban_sdk::Symbol::new(&env, "TIME_TOKEN"));
    assert!(result);
}
