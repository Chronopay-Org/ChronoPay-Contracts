#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, testutils::Ledger, vec, Address, Env, String};

fn setup(env: &Env) -> (Address, Address, ChronoPayContractClient<'_>) {
    env.mock_all_auths();
    let contract_id = env.register(ChronoPayContract, ());
    let client = ChronoPayContractClient::new(env, &contract_id);
    let admin = Address::generate(env);
    let professional = Address::generate(env);
    client.init(&admin);
    (admin, professional, client)
}

fn advance_time(env: &Env, secs: u64) {
    let current = env.ledger().timestamp();
    env.ledger().with_mut(|li| {
        li.timestamp = current + secs;
    });
}

// ── Hello (CI sanity) ─────────────────────────────────────────────────────

#[test]
fn test_hello() {
    let env = Env::default();
    let (_, _, client) = setup(&env);
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

// ── init ──────────────────────────────────────────────────────────────────

#[test]
fn test_init_sets_default_timeout() {
    let env = Env::default();
    let (_, _, client) = setup(&env);
    assert_eq!(client.get_timeout(), 86_400);
}

#[test]
fn test_init_double_init_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(ChronoPayContract, ());
    let client = ChronoPayContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    client.init(&admin);
    let result = client.try_init(&admin);
    assert_eq!(result, Err(Ok(SlotError::AlreadyInitialized)));
}

// ── set_timeout ───────────────────────────────────────────────────────────

#[test]
fn test_set_timeout_success() {
    let env = Env::default();
    let (admin, _, client) = setup(&env);
    client.set_timeout(&admin, &3600);
    assert_eq!(client.get_timeout(), 3600);
}

#[test]
fn test_set_timeout_wrong_admin() {
    let env = Env::default();
    let (_, _, client) = setup(&env);
    let imposter = Address::generate(&env);
    let result = client.try_set_timeout(&imposter, &3600);
    assert_eq!(result, Err(Ok(SlotError::NotAdmin)));
}

#[test]
fn test_set_timeout_zero_fails() {
    let env = Env::default();
    let (admin, _, client) = setup(&env);
    let result = client.try_set_timeout(&admin, &0);
    assert_eq!(result, Err(Ok(SlotError::ZeroTimeout)));
}

// ── create_time_slot ──────────────────────────────────────────────────────

#[test]
fn test_create_time_slot_success() {
    let env = Env::default();
    let (_, pro, client) = setup(&env);
    let id = client.create_time_slot(&pro, &1000, &2000);
    assert_eq!(id, 1);
    let slot = client.get_time_slot(&id);
    assert_eq!(slot.professional, pro);
    assert_eq!(slot.status, SlotStatus::Available);
}

#[test]
fn test_create_time_slot_auto_increments() {
    let env = Env::default();
    let (_, pro, client) = setup(&env);
    assert_eq!(client.create_time_slot(&pro, &1000, &2000), 1);
    assert_eq!(client.create_time_slot(&pro, &3000, &4000), 2);
    assert_eq!(client.create_time_slot(&pro, &5000, &6000), 3);
    assert_eq!(client.get_slot_count(), 3);
}

#[test]
fn test_create_time_slot_invalid_range() {
    let env = Env::default();
    let (_, pro, client) = setup(&env);
    assert_eq!(
        client.try_create_time_slot(&pro, &2000, &1000),
        Err(Ok(SlotError::InvalidTimeRange))
    );
    assert_eq!(
        client.try_create_time_slot(&pro, &1000, &1000),
        Err(Ok(SlotError::InvalidTimeRange))
    );
}

#[test]
fn test_get_time_slot_not_found() {
    let env = Env::default();
    let (_, _, client) = setup(&env);
    assert_eq!(
        client.try_get_time_slot(&999),
        Err(Ok(SlotError::SlotNotFound))
    );
}

// ── book_slot ─────────────────────────────────────────────────────────────

#[test]
fn test_book_slot_creates_settlement() {
    let env = Env::default();
    let (_, pro, client) = setup(&env);
    let buyer = Address::generate(&env);
    let id = client.create_time_slot(&pro, &1000, &2000);

    let settlement = client.book_slot(&buyer, &id);
    assert_eq!(settlement.slot_id, id);
    assert_eq!(settlement.buyer, buyer);
    assert_eq!(settlement.deadline, settlement.booked_at + 86_400);

    let slot = client.get_time_slot(&id);
    assert_eq!(slot.status, SlotStatus::Booked);
}

#[test]
fn test_book_slot_uses_custom_timeout() {
    let env = Env::default();
    let (admin, pro, client) = setup(&env);
    client.set_timeout(&admin, &3600);
    let buyer = Address::generate(&env);
    let id = client.create_time_slot(&pro, &1000, &2000);

    let settlement = client.book_slot(&buyer, &id);
    assert_eq!(settlement.deadline, settlement.booked_at + 3600);
}

#[test]
fn test_book_slot_not_available() {
    let env = Env::default();
    let (_, pro, client) = setup(&env);
    let buyer = Address::generate(&env);
    let id = client.create_time_slot(&pro, &1000, &2000);
    client.book_slot(&buyer, &id);

    let buyer2 = Address::generate(&env);
    assert_eq!(
        client.try_book_slot(&buyer2, &id),
        Err(Ok(SlotError::SlotNotAvailable))
    );
}

#[test]
fn test_book_cancelled_slot_fails() {
    let env = Env::default();
    let (_, pro, client) = setup(&env);
    let buyer = Address::generate(&env);
    let id = client.create_time_slot(&pro, &1000, &2000);
    client.cancel_slot(&pro, &id);

    assert_eq!(
        client.try_book_slot(&buyer, &id),
        Err(Ok(SlotError::SlotNotAvailable))
    );
}

// ── settle_slot ───────────────────────────────────────────────────────────

#[test]
fn test_settle_slot_before_deadline() {
    let env = Env::default();
    let (_, pro, client) = setup(&env);
    let buyer = Address::generate(&env);
    let id = client.create_time_slot(&pro, &1000, &2000);
    client.book_slot(&buyer, &id);

    client.settle_slot(&buyer, &id);
    let slot = client.get_time_slot(&id);
    assert_eq!(slot.status, SlotStatus::Settled);
}

#[test]
fn test_settle_slot_at_exact_deadline() {
    let env = Env::default();
    let (admin, pro, client) = setup(&env);
    client.set_timeout(&admin, &100);
    let buyer = Address::generate(&env);
    let id = client.create_time_slot(&pro, &1000, &2000);
    let settlement = client.book_slot(&buyer, &id);

    env.ledger().with_mut(|li| {
        li.timestamp = settlement.deadline;
    });

    client.settle_slot(&buyer, &id);
    assert_eq!(client.get_time_slot(&id).status, SlotStatus::Settled);
}

#[test]
fn test_settle_slot_after_deadline_fails() {
    let env = Env::default();
    let (admin, pro, client) = setup(&env);
    client.set_timeout(&admin, &100);
    let buyer = Address::generate(&env);
    let id = client.create_time_slot(&pro, &1000, &2000);
    let settlement = client.book_slot(&buyer, &id);

    env.ledger().with_mut(|li| {
        li.timestamp = settlement.deadline + 1;
    });

    assert_eq!(
        client.try_settle_slot(&buyer, &id),
        Err(Ok(SlotError::DeadlineExpired))
    );
}

#[test]
fn test_settle_slot_wrong_buyer() {
    let env = Env::default();
    let (_, pro, client) = setup(&env);
    let buyer = Address::generate(&env);
    let imposter = Address::generate(&env);
    let id = client.create_time_slot(&pro, &1000, &2000);
    client.book_slot(&buyer, &id);

    assert_eq!(
        client.try_settle_slot(&imposter, &id),
        Err(Ok(SlotError::NotBuyer))
    );
}

#[test]
fn test_settle_already_settled_fails() {
    let env = Env::default();
    let (_, pro, client) = setup(&env);
    let buyer = Address::generate(&env);
    let id = client.create_time_slot(&pro, &1000, &2000);
    client.book_slot(&buyer, &id);
    client.settle_slot(&buyer, &id);

    assert_eq!(
        client.try_settle_slot(&buyer, &id),
        Err(Ok(SlotError::AlreadySettled))
    );
}

#[test]
fn test_settle_available_slot_fails() {
    let env = Env::default();
    let (_, pro, client) = setup(&env);
    let buyer = Address::generate(&env);
    let id = client.create_time_slot(&pro, &1000, &2000);

    assert_eq!(
        client.try_settle_slot(&buyer, &id),
        Err(Ok(SlotError::AlreadySettled))
    );
}

// ── timeout_slot ──────────────────────────────────────────────────────────

#[test]
fn test_timeout_slot_after_deadline() {
    let env = Env::default();
    let (admin, pro, client) = setup(&env);
    client.set_timeout(&admin, &100);
    let buyer = Address::generate(&env);
    let id = client.create_time_slot(&pro, &1000, &2000);
    let settlement = client.book_slot(&buyer, &id);

    env.ledger().with_mut(|li| {
        li.timestamp = settlement.deadline + 1;
    });

    client.timeout_slot(&id);
    assert_eq!(client.get_time_slot(&id).status, SlotStatus::TimedOut);
}

#[test]
fn test_timeout_slot_before_deadline_fails() {
    let env = Env::default();
    let (_, pro, client) = setup(&env);
    let buyer = Address::generate(&env);
    let id = client.create_time_slot(&pro, &1000, &2000);
    client.book_slot(&buyer, &id);

    assert_eq!(
        client.try_timeout_slot(&id),
        Err(Ok(SlotError::DeadlineNotReached))
    );
}

#[test]
fn test_timeout_slot_at_exact_deadline_fails() {
    let env = Env::default();
    let (admin, pro, client) = setup(&env);
    client.set_timeout(&admin, &100);
    let buyer = Address::generate(&env);
    let id = client.create_time_slot(&pro, &1000, &2000);
    let settlement = client.book_slot(&buyer, &id);

    env.ledger().with_mut(|li| {
        li.timestamp = settlement.deadline;
    });

    assert_eq!(
        client.try_timeout_slot(&id),
        Err(Ok(SlotError::DeadlineNotReached))
    );
}

#[test]
fn test_timeout_already_settled_fails() {
    let env = Env::default();
    let (_, pro, client) = setup(&env);
    let buyer = Address::generate(&env);
    let id = client.create_time_slot(&pro, &1000, &2000);
    client.book_slot(&buyer, &id);
    client.settle_slot(&buyer, &id);

    assert_eq!(
        client.try_timeout_slot(&id),
        Err(Ok(SlotError::AlreadySettled))
    );
}

#[test]
fn test_timeout_available_slot_fails() {
    let env = Env::default();
    let (_, pro, client) = setup(&env);
    let id = client.create_time_slot(&pro, &1000, &2000);

    assert_eq!(
        client.try_timeout_slot(&id),
        Err(Ok(SlotError::AlreadySettled))
    );
}

// ── cancel_slot ───────────────────────────────────────────────────────────

#[test]
fn test_cancel_slot_success() {
    let env = Env::default();
    let (_, pro, client) = setup(&env);
    let id = client.create_time_slot(&pro, &1000, &2000);
    client.cancel_slot(&pro, &id);
    assert_eq!(client.get_time_slot(&id).status, SlotStatus::Cancelled);
}

#[test]
fn test_cancel_slot_wrong_owner() {
    let env = Env::default();
    let (_, pro, client) = setup(&env);
    let other = Address::generate(&env);
    let id = client.create_time_slot(&pro, &1000, &2000);
    assert_eq!(
        client.try_cancel_slot(&other, &id),
        Err(Ok(SlotError::NotSlotOwner))
    );
}

#[test]
fn test_cancel_booked_slot_fails() {
    let env = Env::default();
    let (_, pro, client) = setup(&env);
    let buyer = Address::generate(&env);
    let id = client.create_time_slot(&pro, &1000, &2000);
    client.book_slot(&buyer, &id);
    assert_eq!(
        client.try_cancel_slot(&pro, &id),
        Err(Ok(SlotError::SlotNotAvailable))
    );
}

#[test]
fn test_cancel_already_cancelled() {
    let env = Env::default();
    let (_, pro, client) = setup(&env);
    let id = client.create_time_slot(&pro, &1000, &2000);
    client.cancel_slot(&pro, &id);
    assert_eq!(
        client.try_cancel_slot(&pro, &id),
        Err(Ok(SlotError::AlreadyCancelled))
    );
}

// ── Full lifecycle simulation ─────────────────────────────────────────────

#[test]
fn test_full_lifecycle_book_settle() {
    let env = Env::default();
    let (admin, pro, client) = setup(&env);
    client.set_timeout(&admin, &3600);

    let buyer = Address::generate(&env);
    let id = client.create_time_slot(&pro, &1000, &2000);

    assert_eq!(client.get_time_slot(&id).status, SlotStatus::Available);

    let settlement = client.book_slot(&buyer, &id);
    assert_eq!(client.get_time_slot(&id).status, SlotStatus::Booked);
    assert_eq!(settlement.deadline, settlement.booked_at + 3600);

    advance_time(&env, 1800);
    client.settle_slot(&buyer, &id);
    assert_eq!(client.get_time_slot(&id).status, SlotStatus::Settled);
}

#[test]
fn test_full_lifecycle_book_timeout() {
    let env = Env::default();
    let (admin, pro, client) = setup(&env);
    client.set_timeout(&admin, &3600);

    let buyer = Address::generate(&env);
    let id = client.create_time_slot(&pro, &1000, &2000);

    client.book_slot(&buyer, &id);
    assert_eq!(client.get_time_slot(&id).status, SlotStatus::Booked);

    advance_time(&env, 3601);
    client.timeout_slot(&id);
    assert_eq!(client.get_time_slot(&id).status, SlotStatus::TimedOut);
}

// ── Legacy stubs ──────────────────────────────────────────────────────────

#[test]
fn test_mint_time_token_stub() {
    let env = Env::default();
    let (_, pro, client) = setup(&env);
    let id = client.create_time_slot(&pro, &1000, &2000);
    assert_eq!(
        client.mint_time_token(&id),
        soroban_sdk::Symbol::new(&env, "TIME_TOKEN")
    );
}

#[test]
fn test_redeem_time_token_stub() {
    let env = Env::default();
    let (_, _, client) = setup(&env);
    assert!(client.redeem_time_token(&soroban_sdk::Symbol::new(&env, "TIME_TOKEN")));
}
