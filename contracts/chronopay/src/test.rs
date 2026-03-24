#![cfg(test)]

use super::*;
use soroban_sdk::{Env, String, Symbol};

// ── helpers ──────────────────────────────────────────────────────────────────

fn setup() -> (Env, ChronoPayContractClient<'static>) {
    let env = Env::default();
    let contract_id = env.register(ChronoPayContract, ());
    let client = ChronoPayContractClient::new(&env, &contract_id);
    (env, client)
}

fn alice(env: &Env) -> String { String::from_str(env, "alice") }
fn bob(env: &Env)   -> String { String::from_str(env, "bob")   }
fn pro(env: &Env)   -> String { String::from_str(env, "professional_alice") }
fn token(env: &Env) -> Symbol { Symbol::new(env, "TIME_TOKEN") }

// ── hello ─────────────────────────────────────────────────────────────────────

#[test]
fn test_hello() {
    let (env, client) = setup();
    let words = client.hello(&String::from_str(&env, "Dev"));
    assert_eq!(words, soroban_sdk::vec![
        &env,
        String::from_str(&env, "ChronoPay"),
        String::from_str(&env, "Dev"),
    ]);
}

// ── create_time_slot ──────────────────────────────────────────────────────────

#[test]
fn test_create_time_slot_auto_increments() {
    let (env, client) = setup();
    assert_eq!(client.create_time_slot(&pro(&env), &1000u64, &2000u64), 1);
    assert_eq!(client.create_time_slot(&pro(&env), &3000u64, &4000u64), 2);
    assert_eq!(client.create_time_slot(&pro(&env), &5000u64, &6000u64), 3);
}

#[test]
fn test_create_time_slot_starts_at_one() {
    let (env, client) = setup();
    // First slot on a fresh contract must be 1, never 0
    let id = client.create_time_slot(&pro(&env), &0u64, &100u64);
    assert_eq!(id, 1);
}

// ── mint_time_token ───────────────────────────────────────────────────────────

#[test]
#[test]
fn test_mint_returns_time_token_symbol() {
    let (env, client) = setup();
    let slot_id = client.create_time_slot(&pro(&env), &1000u64, &2000u64);
    let tok = client.mint_time_token(&slot_id);
    assert_eq!(tok, Symbol::new(&env, "T_1"));
}

#[test]
fn test_mint_is_idempotent() {
    // Minting the same slot twice should not panic or corrupt status
    let (env, client) = setup();
    let slot_id = client.create_time_slot(&pro(&env), &1000u64, &2000u64);
    let t1 = client.mint_time_token(&slot_id);
    let t2 = client.mint_time_token(&slot_id);
    assert_eq!(t1, t2);
}

// ── buy_time_token — happy path ───────────────────────────────────────────────

#[test]
fn test_buy_available_token_succeeds() {
    let (env, client) = setup();
    let slot_id = client.create_time_slot(&pro(&env), &1000u64, &2000u64);
    let tok = client.mint_time_token(&slot_id);
    let result = client.buy_time_token(&tok, &alice(&env), &pro(&env));
    assert!(result);
}

// ── buy_time_token — idempotency ──────────────────────────────────────────────

#[test]
fn test_buy_same_buyer_twice_is_idempotent() {
    // Core requirement: repeated call from the same buyer must return true
    // without side effects (no panic, no state corruption).
    let (env, client) = setup();
    let slot_id = client.create_time_slot(&pro(&env), &1000u64, &2000u64);
    let tok = client.mint_time_token(&slot_id);

    let first  = client.buy_time_token(&tok, &alice(&env), &pro(&env));
    let second = client.buy_time_token(&tok, &alice(&env), &pro(&env));
    let third  = client.buy_time_token(&tok, &alice(&env), &pro(&env));

    assert!(first,  "first purchase must succeed");
    assert!(second, "idempotent repeat must return true");
    assert!(third,  "idempotent repeat (3rd) must return true");
}

#[test]
fn test_buy_different_buyer_after_sold_returns_false() {
    // A different buyer cannot purchase an already-sold token
    let (env, client) = setup();
    let slot_id = client.create_time_slot(&pro(&env), &1000u64, &2000u64);
    let tok = client.mint_time_token(&slot_id);

    let first  = client.buy_time_token(&tok, &alice(&env), &pro(&env));
    let second = client.buy_time_token(&tok, &bob(&env),   &pro(&env));

    assert!(first,   "alice's purchase must succeed");
    assert!(!second, "bob cannot buy alice's token");
}

#[test]
fn test_buy_does_not_overwrite_owner_on_repeat() {
    // After idempotent repeat, the owner must still be the original buyer —
    // not overwritten to a new value.
    let (env, client) = setup();
    let slot_id = client.create_time_slot(&pro(&env), &1000u64, &2000u64);
    let tok = client.mint_time_token(&slot_id);

    client.buy_time_token(&tok, &alice(&env), &pro(&env));
    // Attempt a second buy from alice (idempotent), then bob must still be rejected
    client.buy_time_token(&tok, &alice(&env), &pro(&env));
    let bob_attempt = client.buy_time_token(&tok, &bob(&env), &pro(&env));

    assert!(!bob_attempt, "owner must not be overwritten after idempotent repeat");
}

// ── buy_time_token — redeemed token ───────────────────────────────────────────

#[test]
fn test_buy_redeemed_token_returns_false() {
    let (env, client) = setup();
    let slot_id = client.create_time_slot(&pro(&env), &1000u64, &2000u64);
    let tok = client.mint_time_token(&slot_id);

    client.buy_time_token(&tok, &alice(&env), &pro(&env));
    client.redeem_time_token(&tok);

    let late_buy = client.buy_time_token(&tok, &bob(&env), &pro(&env));
    assert!(!late_buy, "cannot buy a redeemed token");
}

// ── redeem_time_token ─────────────────────────────────────────────────────────

#[test]
fn test_redeem_after_purchase_succeeds() {
    let (env, client) = setup();
    let slot_id = client.create_time_slot(&pro(&env), &1000u64, &2000u64);
    let tok = client.mint_time_token(&slot_id);
    client.buy_time_token(&tok, &alice(&env), &pro(&env));
    assert!(client.redeem_time_token(&tok));
}

#[test]
fn test_redeem_twice_returns_false() {
    // Double-redeem must be rejected — token is already spent
    let (env, client) = setup();
    let slot_id = client.create_time_slot(&pro(&env), &1000u64, &2000u64);
    let tok = client.mint_time_token(&slot_id);
    client.buy_time_token(&tok, &alice(&env), &pro(&env));

    let first  = client.redeem_time_token(&tok);
    let second = client.redeem_time_token(&tok);

    assert!(first,   "first redeem must succeed");
    assert!(!second, "double-redeem must return false");
}

#[test]
fn test_redeem_without_purchase_still_marks_redeemed() {
    // Edge case: redeeming an Available token (no prior buy)
    // should succeed once and fail on repeat
    let (env, client) = setup();
    let slot_id = client.create_time_slot(&pro(&env), &1000u64, &2000u64);
    let tok = client.mint_time_token(&slot_id);

    let first  = client.redeem_time_token(&tok);
    let second = client.redeem_time_token(&tok);

    assert!(first);
    assert!(!second);
}

// ── full lifecycle ─────────────────────────────────────────────────────────────

#[test]
fn test_full_lifecycle() {
    // create → mint → buy → idempotent repeat → redeem
    let (env, client) = setup();

    let slot_id = client.create_time_slot(&pro(&env), &1000u64, &9000u64);
    assert_eq!(slot_id, 1);

   let tok = client.mint_time_token(&slot_id);
assert_eq!(tok, Symbol::new(&env, "T_1"));

    assert!(client.buy_time_token(&tok, &alice(&env), &pro(&env)));  // first buy
    assert!(client.buy_time_token(&tok, &alice(&env), &pro(&env)));  // idempotent
    assert!(!client.buy_time_token(&tok, &bob(&env), &pro(&env)));   // bob rejected

    assert!(client.redeem_time_token(&tok));                          // redeem
    assert!(!client.redeem_time_token(&tok));                         // double-redeem blocked
    assert!(!client.buy_time_token(&tok, &bob(&env), &pro(&env)));   // post-redeem buy blocked
}

#[test]
fn test_multiple_independent_tokens() {
    // Two slots/tokens must not share state
    let (env, client) = setup();

    let slot_1 = client.create_time_slot(&pro(&env), &1000u64, &2000u64);
    let slot_2 = client.create_time_slot(&pro(&env), &3000u64, &4000u64);
    let tok_1 = client.mint_time_token(&slot_1);
    let tok_2 = client.mint_time_token(&slot_2);

    // Buy only token 1
    assert!(client.buy_time_token(&tok_1, &alice(&env), &pro(&env)));

    // Token 2 should still be purchasable by bob
    assert!(client.buy_time_token(&tok_2, &bob(&env), &pro(&env)));

    // Token 1 alice idempotent, bob blocked
    assert!(client.buy_time_token(&tok_1, &alice(&env), &pro(&env)));
    assert!(!client.buy_time_token(&tok_1, &bob(&env),  &pro(&env)));
}