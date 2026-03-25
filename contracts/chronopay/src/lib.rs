#![no_std]
//! ChronoPay time token contract (Soroban).
//!
//! This crate intentionally keeps the business logic small and auditable.
//!
//! ## Replay-proof purchase nonce
//!
//! `buy_time_token` includes a **per-buyer, strictly-increasing nonce** stored on-chain to
//! make purchases idempotent and replay-resistant (e.g., when a signed purchase intent is
//! relayed multiple times).
//!
//! Acceptance criteria:
//! - Each buyer has a `purchase_nonce` that starts at `0`.
//! - A purchase succeeds only when the supplied `nonce == purchase_nonce`.
//! - On success, the stored nonce is incremented by `1`.
//! - Re-using a nonce (replay) or skipping ahead fails with `Error::InvalidPurchaseNonce`.
//! - The buyer must authorize the purchase (`buyer.require_auth()`).

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, vec, Address, Env, String, Symbol, Vec,
};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TimeTokenStatus {
    Available,
    Sold,
    Redeemed,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum Error {
    /// The provided nonce does not match the next expected nonce for the buyer.
    InvalidPurchaseNonce = 1,
    /// The purchase nonce cannot be incremented further.
    PurchaseNonceOverflow = 2,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    SlotSeq,
    Owner,
    Status,
    PurchaseNonce(Address),
}

#[contract]
pub struct ChronoPayContract;

fn get_next_purchase_nonce(env: &Env, buyer: &Address) -> u64 {
    env.storage()
        .instance()
        .get(&DataKey::PurchaseNonce(buyer.clone()))
        .unwrap_or(0u64)
}

fn consume_purchase_nonce(env: &Env, buyer: &Address, nonce: u64) -> Result<(), Error> {
    let expected = get_next_purchase_nonce(env, buyer);
    if nonce != expected {
        return Err(Error::InvalidPurchaseNonce);
    }

    let next = expected
        .checked_add(1)
        .ok_or(Error::PurchaseNonceOverflow)?;
    env.storage()
        .instance()
        .set(&DataKey::PurchaseNonce(buyer.clone()), &next);

    Ok(())
}

#[contractimpl]
impl ChronoPayContract {
    /// Create a time slot with an auto-incrementing slot id.
    /// Returns the newly assigned slot id.
    pub fn create_time_slot(env: Env, professional: String, start_time: u64, end_time: u64) -> u32 {
        let _ = (professional, start_time, end_time);

        let current_seq: u32 = env
            .storage()
            .instance()
            .get(&DataKey::SlotSeq)
            .unwrap_or(0u32);

        let next_seq = current_seq.checked_add(1).expect("slot id overflow");

        env.storage().instance().set(&DataKey::SlotSeq, &next_seq);

        next_seq
    }

    /// Mint a time token for a slot (stub).
    pub fn mint_time_token(env: Env, slot_id: u32) -> Symbol {
        let _ = slot_id;
        Symbol::new(&env, "TIME_TOKEN")
    }

    /// Returns the **next expected** purchase nonce for `buyer`.
    ///
    /// Client guidance: call this before constructing a purchase to learn which nonce to supply.
    pub fn get_purchase_nonce(env: Env, buyer: Address) -> u64 {
        get_next_purchase_nonce(&env, &buyer)
    }

    /// Buy / transfer time token.
    ///
    /// Replay protection: the purchase is accepted only when `nonce` matches the next expected
    /// nonce for `buyer`, and the nonce is incremented on success.
    pub fn buy_time_token(
        env: Env,
        token_id: Symbol,
        buyer: Address,
        seller: Address,
        nonce: u64,
    ) -> Result<bool, Error> {
        let _ = (token_id, seller);

        // Prevent unauthorized purchases and make intent-based purchases safe to relay.
        buyer.require_auth();

        // Consume the nonce *before* mutating any additional state, so a failed validation
        // cannot be replayed into a partial update.
        consume_purchase_nonce(&env, &buyer, nonce)?;

        env.storage().instance().set(&DataKey::Owner, &buyer);
        env.storage()
            .instance()
            .set(&DataKey::Status, &TimeTokenStatus::Sold);

        Ok(true)
    }

    /// Redeem time token (stub). In full implementation: token_id, marks as redeemed.
    pub fn redeem_time_token(env: Env, token_id: Symbol) -> bool {
        let _ = token_id;
        env.storage()
            .instance()
            .set(&DataKey::Status, &TimeTokenStatus::Redeemed);
        true
    }

    /// Hello-style entrypoint for CI and SDK sanity check.
    pub fn hello(env: Env, to: String) -> Vec<String> {
        vec![&env, String::from_str(&env, "ChronoPay"), to]
    }
}

mod test;
