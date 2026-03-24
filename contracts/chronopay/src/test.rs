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

fn alice(env: &Env) -> String {
    String::from_str(env, "alice")
}
fn bob(env: &Env) -> String {
    String::from_str(env, "bob")
}
fn pro(env: &Env) -> String {
    String::from_str(env, "professional_alice")
}
fn token(env: &Env) -> Symbol {
    Symbol::new(env, "TIME_TOKEN")
}

// ── hello ─────────────────────────────────────────────────────────────────────

#[test]
fn test_hello() {
    let (env, client) = setup();
    let words = client.hello(&String::from_str(&env, "Dev"));
    assert_eq!(
        words,
        soroban_sdk::vec![
            &env,
            String::from_str(&env, "ChronoPay"),
            String::from_str(&env, "Dev"),
        ]
    );
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

    let first = client.buy_time_token(&tok, &alice(&env), &pro(&env));
    let second = client.buy_time_token(&tok, &alice(&env), &pro(&env));
    let third = client.buy_time_token(&tok, &alice(&env), &pro(&env));

    assert!(first, "first purchase must succeed");
    assert!(second, "idempotent repeat must return true");
    assert!(third, "idempotent repeat (3rd) must return true");
}

#[test]
fn test_buy_different_buyer_after_sold_returns_false() {
    // A different buyer cannot purchase an already-sold token
    let (env, client) = setup();
    let slot_id = client.create_time_slot(&pro(&env), &1000u64, &2000u64);
    let tok = client.mint_time_token(&slot_id);

    let first = client.buy_time_token(&tok, &alice(&env), &pro(&env));
    let second = client.buy_time_token(&tok, &bob(&env), &pro(&env));

    assert!(first, "alice's purchase must succeed");
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

    assert!(
        !bob_attempt,
        "owner must not be overwritten after idempotent repeat"
    );
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

    let first = client.redeem_time_token(&tok);
    let second = client.redeem_time_token(&tok);

    assert!(first, "first redeem must succeed");
    assert!(!second, "double-redeem must return false");
}

#[test]
fn test_redeem_without_purchase_still_marks_redeemed() {
    // Edge case: redeeming an Available token (no prior buy)
    // should succeed once and fail on repeat
    let (env, client) = setup();
    let slot_id = client.create_time_slot(&pro(&env), &1000u64, &2000u64);
    let tok = client.mint_time_token(&slot_id);

    let first = client.redeem_time_token(&tok);
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

    assert!(client.buy_time_token(&tok, &alice(&env), &pro(&env))); // first buy
    assert!(client.buy_time_token(&tok, &alice(&env), &pro(&env))); // idempotent
    assert!(!client.buy_time_token(&tok, &bob(&env), &pro(&env))); // bob rejected

    assert!(client.redeem_time_token(&tok)); // redeem
    assert!(!client.redeem_time_token(&tok)); // double-redeem blocked
    assert!(!client.buy_time_token(&tok, &bob(&env), &pro(&env))); // post-redeem buy blocked
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
    assert!(!client.buy_time_token(&tok_1, &bob(&env), &pro(&env)));
}

// ═════════════════════════════════════════════════════════════════════════════
// SC-053 — Integration tests: full slot lifecycle
//
// These tests exercise multi-step, realistic scenarios across the full
// create → mint → buy → redeem pipeline. Unlike the unit tests above,
// each test here represents a complete user journey or a realistic
// failure mode a production deployment would encounter.
//
// Security assumptions validated:
// - Slot state is never shared across professionals or buyers
// - A token cannot move backwards in status (Redeemed → Sold is impossible)
// - Sequence integrity holds across interleaved operations
// ═════════════════════════════════════════════════════════════════════════════

fn pro_bob(env: &Env) -> String {
    String::from_str(env, "professional_bob")
}
fn carol(env: &Env) -> String {
    String::from_str(env, "carol")
}

// ── scenario 1: canonical happy path ─────────────────────────────────────────

#[test]
fn integration_canonical_lifecycle() {
    // Full end-to-end: one professional, one buyer, one slot, clean redeem.
    // This is the baseline scenario every other test deviates from.
    let (env, client) = setup();

    // Stage 1: professional creates a slot
    let slot_id = client.create_time_slot(&pro(&env), &9000u64, &18000u64);
    assert_eq!(slot_id, 1, "first slot must be id 1");

    // Stage 2: slot is minted into a token
    let tok = client.mint_time_token(&slot_id);
    assert_eq!(tok, Symbol::new(&env, "T_1"));

    // Stage 3: buyer purchases the token
    assert!(
        client.buy_time_token(&tok, &alice(&env), &pro(&env)),
        "purchase of available token must succeed"
    );

    // Stage 4: token is redeemed (session delivered)
    assert!(
        client.redeem_time_token(&tok),
        "redeem after purchase must succeed"
    );

    // Stage 5: verify terminal state — no further actions possible
    assert!(
        !client.redeem_time_token(&tok),
        "token cannot be redeemed twice"
    );
    assert!(
        !client.buy_time_token(&tok, &bob(&env), &pro(&env)),
        "token cannot be purchased after redemption"
    );
}

// ── scenario 2: two professionals, independent slot sequences ─────────────────

#[test]
fn integration_two_professionals_independent_sequences() {
    // Each professional's slots are independent — purchasing or redeeming
    // one must never affect the other's tokens.
    let (env, client) = setup();

    let slot_a = client.create_time_slot(&pro(&env), &1000u64, &2000u64);
    let slot_b = client.create_time_slot(&pro_bob(&env), &3000u64, &4000u64);

    assert_eq!(slot_a, 1);
    assert_eq!(slot_b, 2);

    let tok_a = client.mint_time_token(&slot_a);
    let tok_b = client.mint_time_token(&slot_b);

    // Alice buys pro_alice's slot, carol buys pro_bob's slot
    assert!(client.buy_time_token(&tok_a, &alice(&env), &pro(&env)));
    assert!(client.buy_time_token(&tok_b, &carol(&env), &pro_bob(&env)));

    // Redeem pro_alice's token — must not affect pro_bob's
    assert!(client.redeem_time_token(&tok_a));
    assert!(
        !client.redeem_time_token(&tok_a),
        "pro_alice token already redeemed"
    );

    // pro_bob's token must still be redeemable independently
    assert!(
        client.redeem_time_token(&tok_b),
        "pro_bob token must redeem independently"
    );
}

// ── scenario 3: buyer attempts to jump the queue ─────────────────────────────

#[test]
fn integration_second_buyer_cannot_jump_queue() {
    // Once alice purchases a token, bob must be rejected at every stage —
    // including after alice's idempotent repeat calls.
    let (env, client) = setup();

    let slot_id = client.create_time_slot(&pro(&env), &5000u64, &6000u64);
    let tok = client.mint_time_token(&slot_id);

    // Alice purchases and repeats (idempotent)
    assert!(client.buy_time_token(&tok, &alice(&env), &pro(&env)));
    assert!(client.buy_time_token(&tok, &alice(&env), &pro(&env)));
    assert!(client.buy_time_token(&tok, &alice(&env), &pro(&env)));

    // Bob attempts at every stage — all must fail
    assert!(
        !client.buy_time_token(&tok, &bob(&env), &pro(&env)),
        "bob rejected after alice's first buy"
    );

    // Even after alice redeems, bob cannot buy
    assert!(client.redeem_time_token(&tok));
    assert!(
        !client.buy_time_token(&tok, &bob(&env), &pro(&env)),
        "bob rejected after redemption"
    );
}

// ── scenario 4: multiple slots interleaved ────────────────────────────────────

#[test]
fn integration_interleaved_slot_operations() {
    // Realistic session: three slots created upfront, purchased and redeemed
    // in a non-sequential order. State must never bleed between slots.
    let (env, client) = setup();

    let s1 = client.create_time_slot(&pro(&env), &1000u64, &2000u64);
    let s2 = client.create_time_slot(&pro(&env), &3000u64, &4000u64);
    let s3 = client.create_time_slot(&pro(&env), &5000u64, &6000u64);

    let t1 = client.mint_time_token(&s1);
    let t2 = client.mint_time_token(&s2);
    let t3 = client.mint_time_token(&s3);

    // Purchase out of order: s3 first, then s1, skip s2
    assert!(client.buy_time_token(&t3, &carol(&env), &pro(&env)));
    assert!(client.buy_time_token(&t1, &alice(&env), &pro(&env)));

    // s2 still available — bob can buy it
    assert!(client.buy_time_token(&t2, &bob(&env), &pro(&env)));

    // Redeem in yet another order: s2, s3, s1
    assert!(client.redeem_time_token(&t2));
    assert!(client.redeem_time_token(&t3));
    assert!(client.redeem_time_token(&t1));

    // All tokens now spent — nothing redeemable
    assert!(!client.redeem_time_token(&t1));
    assert!(!client.redeem_time_token(&t2));
    assert!(!client.redeem_time_token(&t3));
}

// ── scenario 5: slot sequence integrity ──────────────────────────────────────

#[test]
fn integration_slot_sequence_is_monotonic_and_gapless() {
    // Slot IDs must increment by exactly 1 each time with no gaps,
    // regardless of what operations happen between creations.
    let (env, client) = setup();

    let s1 = client.create_time_slot(&pro(&env), &0u64, &1u64);
    let t1 = client.mint_time_token(&s1);
    client.buy_time_token(&t1, &alice(&env), &pro(&env));

    let s2 = client.create_time_slot(&pro(&env), &2u64, &3u64);
    client.redeem_time_token(&t1); // redeem between creates

    let s3 = client.create_time_slot(&pro_bob(&env), &4u64, &5u64);

    assert_eq!(s1, 1, "slot 1 must be 1");
    assert_eq!(s2, 2, "slot 2 must be 2 — no gap after buy/redeem");
    assert_eq!(
        s3, 3,
        "slot 3 must be 3 — different professional, same sequence"
    );
}

// ── scenario 6: redeem-before-buy edge case ───────────────────────────────────

#[test]
fn integration_redeem_before_buy_blocks_subsequent_purchase() {
    // If a token is redeemed without being purchased first (operator error),
    // no buyer must be able to purchase it afterwards.
    let (env, client) = setup();

    let slot_id = client.create_time_slot(&pro(&env), &1000u64, &2000u64);
    let tok = client.mint_time_token(&slot_id);

    // Redeem directly — no buy
    assert!(client.redeem_time_token(&tok));

    // Neither alice nor bob can purchase a redeemed token
    assert!(
        !client.buy_time_token(&tok, &alice(&env), &pro(&env)),
        "alice cannot buy a pre-redeemed token"
    );
    assert!(
        !client.buy_time_token(&tok, &bob(&env), &pro(&env)),
        "bob cannot buy a pre-redeemed token"
    );
}

// ── scenario 7: high-volume slot creation ────────────────────────────────────
#[test]
fn integration_ten_slots_all_independent() {
    // Stress the slot/token isolation guarantee across 10 concurrent slots.
    // Each slot is bought by a unique buyer string and redeemed independently.
    let (env, client) = setup();

    // Create and mint 10 slots — verify sequential IDs
    let s1 = client.create_time_slot(&pro(&env), &0u64, &500u64);
    let s2 = client.create_time_slot(&pro(&env), &1000u64, &1500u64);
    let s3 = client.create_time_slot(&pro(&env), &2000u64, &2500u64);
    let s4 = client.create_time_slot(&pro(&env), &3000u64, &3500u64);
    let s5 = client.create_time_slot(&pro(&env), &4000u64, &4500u64);
    let s6 = client.create_time_slot(&pro(&env), &5000u64, &5500u64);
    let s7 = client.create_time_slot(&pro(&env), &6000u64, &6500u64);
    let s8 = client.create_time_slot(&pro(&env), &7000u64, &7500u64);
    let s9 = client.create_time_slot(&pro(&env), &8000u64, &8500u64);
    let s10 = client.create_time_slot(&pro(&env), &9000u64, &9500u64);

    assert_eq!(
        (s1, s2, s3, s4, s5, s6, s7, s8, s9, s10),
        (1, 2, 3, 4, 5, 6, 7, 8, 9, 10),
        "slots must be sequentially numbered 1-10"
    );

    // Mint all tokens
    let t1 = client.mint_time_token(&s1);
    let t2 = client.mint_time_token(&s2);
    let t3 = client.mint_time_token(&s3);
    let t4 = client.mint_time_token(&s4);
    let t5 = client.mint_time_token(&s5);
    let t6 = client.mint_time_token(&s6);
    let t7 = client.mint_time_token(&s7);
    let t8 = client.mint_time_token(&s8);
    let t9 = client.mint_time_token(&s9);
    let t10 = client.mint_time_token(&s10);

    // Buy every token with a distinct buyer — all must succeed
    assert!(client.buy_time_token(&t1, &String::from_str(&env, "u1"), &pro(&env)));
    assert!(client.buy_time_token(&t2, &String::from_str(&env, "u2"), &pro(&env)));
    assert!(client.buy_time_token(&t3, &String::from_str(&env, "u3"), &pro(&env)));
    assert!(client.buy_time_token(&t4, &String::from_str(&env, "u4"), &pro(&env)));
    assert!(client.buy_time_token(&t5, &String::from_str(&env, "u5"), &pro(&env)));
    assert!(client.buy_time_token(&t6, &String::from_str(&env, "u6"), &pro(&env)));
    assert!(client.buy_time_token(&t7, &String::from_str(&env, "u7"), &pro(&env)));
    assert!(client.buy_time_token(&t8, &String::from_str(&env, "u8"), &pro(&env)));
    assert!(client.buy_time_token(&t9, &String::from_str(&env, "u9"), &pro(&env)));
    assert!(client.buy_time_token(&t10, &String::from_str(&env, "u10"), &pro(&env)));

    // Redeem all — each must succeed exactly once
    assert!(client.redeem_time_token(&t1));
    assert!(!client.redeem_time_token(&t1));
    assert!(client.redeem_time_token(&t2));
    assert!(!client.redeem_time_token(&t2));
    assert!(client.redeem_time_token(&t3));
    assert!(!client.redeem_time_token(&t3));
    assert!(client.redeem_time_token(&t4));
    assert!(!client.redeem_time_token(&t4));
    assert!(client.redeem_time_token(&t5));
    assert!(!client.redeem_time_token(&t5));
    assert!(client.redeem_time_token(&t6));
    assert!(!client.redeem_time_token(&t6));
    assert!(client.redeem_time_token(&t7));
    assert!(!client.redeem_time_token(&t7));
    assert!(client.redeem_time_token(&t8));
    assert!(!client.redeem_time_token(&t8));
    assert!(client.redeem_time_token(&t9));
    assert!(!client.redeem_time_token(&t9));
    assert!(client.redeem_time_token(&t10));
    assert!(!client.redeem_time_token(&t10));
}
