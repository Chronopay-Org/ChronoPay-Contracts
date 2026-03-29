#![no_std]
//! ChronoPay time-slot tokenization contract.
//!
//! SC-011 introduces production-grade payment guardrails around
//! `buy_time_token` and the minimum supporting state needed to enforce them.
//!
//! Acceptance criteria:
//! - Professionals must authorize slot creation and token minting.
//! - Buyers must authorize purchases.
//! - Time slots must have a valid time range and a strictly positive price.
//! - A time token can only be purchased once, and never after redemption.
//! - The buyer cannot purchase from themselves.
//! - The seller supplied to `buy_time_token` must match the seller recorded
//!   when the token was minted.
//! - The purchase amount must exactly match the listed price.
//!
//! Failure modes are surfaced as `ChronoPayError` values so callers and tests
//! can distinguish business-rule failures from authorization failures.

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, vec, Address, Env, String, Vec,
};

const INSTANCE_TTL_THRESHOLD: u32 = 100;
const INSTANCE_TTL_BUMP: u32 = 1_000;
const PERSISTENT_TTL_THRESHOLD: u32 = 100;
const PERSISTENT_TTL_BUMP: u32 = 1_000;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TimeTokenStatus {
    Available,
    Sold,
    Redeemed,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TimeSlot {
    pub professional: Address,
    pub start_time: u64,
    pub end_time: u64,
    pub price: i128,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TimeToken {
    pub slot_id: u32,
    pub seller: Address,
    pub owner: Address,
    pub price: i128,
    pub amount_paid: i128,
    pub status: TimeTokenStatus,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ChronoPayError {
    InvalidTimeRange = 1,
    InvalidPrice = 2,
    SlotNotFound = 3,
    SlotAlreadyTokenized = 4,
    TokenNotFound = 5,
    TokenAlreadySold = 6,
    TokenAlreadyRedeemed = 7,
    TokenNotSold = 8,
    BuyerIsSeller = 9,
    SellerMismatch = 10,
    PaymentAmountMismatch = 11,
    NotTokenOwner = 12,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    SlotSeq,
    TokenSeq,
    Slot(u32),
    Token(u32),
    TokenBySlot(u32),
}

#[contract]
pub struct ChronoPayContract;

#[contractimpl]
impl ChronoPayContract {
    /// Creates a priced time slot owned by the professional.
    pub fn create_time_slot(
        env: Env,
        professional: Address,
        start_time: u64,
        end_time: u64,
        price: i128,
    ) -> Result<u32, ChronoPayError> {
        professional.require_auth();

        if end_time <= start_time {
            return Err(ChronoPayError::InvalidTimeRange);
        }
        if price <= 0 {
            return Err(ChronoPayError::InvalidPrice);
        }

        let slot_id = next_sequence(&env, &DataKey::SlotSeq);
        let slot = TimeSlot {
            professional,
            start_time,
            end_time,
            price,
        };

        write_slot(&env, slot_id, &slot);
        Ok(slot_id)
    }

    /// Mints a single transferable token for a slot.
    pub fn mint_time_token(env: Env, slot_id: u32) -> Result<u32, ChronoPayError> {
        let slot = read_slot(&env, slot_id)?;
        slot.professional.require_auth();

        let token_by_slot_key = DataKey::TokenBySlot(slot_id);
        if env.storage().persistent().has(&token_by_slot_key) {
            return Err(ChronoPayError::SlotAlreadyTokenized);
        }

        let token_id = next_sequence(&env, &DataKey::TokenSeq);
        let token = TimeToken {
            slot_id,
            seller: slot.professional.clone(),
            owner: slot.professional,
            price: slot.price,
            amount_paid: 0,
            status: TimeTokenStatus::Available,
        };

        write_token(&env, token_id, &token);
        env.storage()
            .persistent()
            .set(&token_by_slot_key, &token_id);
        env.storage().persistent().extend_ttl(
            &token_by_slot_key,
            PERSISTENT_TTL_THRESHOLD,
            PERSISTENT_TTL_BUMP,
        );

        Ok(token_id)
    }

    /// Purchases an available time token.
    ///
    /// The `seller` argument is validated against the token listing to guard
    /// against clients accidentally or maliciously routing payment metadata to a
    /// different seller identity.
    pub fn buy_time_token(
        env: Env,
        token_id: u32,
        buyer: Address,
        seller: Address,
        payment_amount: i128,
    ) -> Result<(), ChronoPayError> {
        buyer.require_auth();

        let mut token = read_token(&env, token_id)?;
        match token.status {
            TimeTokenStatus::Available => {}
            TimeTokenStatus::Sold => return Err(ChronoPayError::TokenAlreadySold),
            TimeTokenStatus::Redeemed => return Err(ChronoPayError::TokenAlreadyRedeemed),
        }

        if buyer == seller {
            return Err(ChronoPayError::BuyerIsSeller);
        }
        if seller != token.seller {
            return Err(ChronoPayError::SellerMismatch);
        }
        if payment_amount != token.price {
            return Err(ChronoPayError::PaymentAmountMismatch);
        }

        // Validate the full purchase before mutating any token state so the
        // listing remains unchanged on every rejected payment path.
        token.owner = buyer;
        token.amount_paid = payment_amount;
        token.status = TimeTokenStatus::Sold;

        write_token(&env, token_id, &token);
        Ok(())
    }

    /// Redeems a previously sold token. Only the current owner may redeem.
    pub fn redeem_time_token(
        env: Env,
        token_id: u32,
        redeemer: Address,
    ) -> Result<(), ChronoPayError> {
        redeemer.require_auth();

        let mut token = read_token(&env, token_id)?;
        match token.status {
            TimeTokenStatus::Available => return Err(ChronoPayError::TokenNotSold),
            TimeTokenStatus::Sold => {}
            TimeTokenStatus::Redeemed => return Err(ChronoPayError::TokenAlreadyRedeemed),
        }

        if token.owner != redeemer {
            return Err(ChronoPayError::NotTokenOwner);
        }

        token.status = TimeTokenStatus::Redeemed;
        write_token(&env, token_id, &token);
        Ok(())
    }

    /// Returns a stored slot for auditability and tests.
    pub fn get_time_slot(env: Env, slot_id: u32) -> Result<TimeSlot, ChronoPayError> {
        read_slot(&env, slot_id)
    }

    /// Returns a stored token for auditability and tests.
    pub fn get_time_token(env: Env, token_id: u32) -> Result<TimeToken, ChronoPayError> {
        read_token(&env, token_id)
    }

    /// Hello-style entrypoint for CI and SDK sanity checks.
    pub fn hello(env: Env, to: String) -> Vec<String> {
        vec![&env, String::from_str(&env, "ChronoPay"), to]
    }
}

fn next_sequence(env: &Env, key: &DataKey) -> u32 {
    bump_instance_ttl(env);
    let current: u32 = env.storage().instance().get(key).unwrap_or(0u32);
    let next = current.checked_add(1).expect("sequence overflow");
    env.storage().instance().set(key, &next);
    next
}

fn read_slot(env: &Env, slot_id: u32) -> Result<TimeSlot, ChronoPayError> {
    let key = DataKey::Slot(slot_id);
    let slot = env
        .storage()
        .persistent()
        .get(&key)
        .ok_or(ChronoPayError::SlotNotFound)?;
    env.storage()
        .persistent()
        .extend_ttl(&key, PERSISTENT_TTL_THRESHOLD, PERSISTENT_TTL_BUMP);
    Ok(slot)
}

fn write_slot(env: &Env, slot_id: u32, slot: &TimeSlot) {
    let key = DataKey::Slot(slot_id);
    env.storage().persistent().set(&key, slot);
    env.storage()
        .persistent()
        .extend_ttl(&key, PERSISTENT_TTL_THRESHOLD, PERSISTENT_TTL_BUMP);
}

fn read_token(env: &Env, token_id: u32) -> Result<TimeToken, ChronoPayError> {
    let key = DataKey::Token(token_id);
    let token = env
        .storage()
        .persistent()
        .get(&key)
        .ok_or(ChronoPayError::TokenNotFound)?;
    env.storage()
        .persistent()
        .extend_ttl(&key, PERSISTENT_TTL_THRESHOLD, PERSISTENT_TTL_BUMP);
    Ok(token)
}

fn write_token(env: &Env, token_id: u32, token: &TimeToken) {
    let key = DataKey::Token(token_id);
    env.storage().persistent().set(&key, token);
    env.storage()
        .persistent()
        .extend_ttl(&key, PERSISTENT_TTL_THRESHOLD, PERSISTENT_TTL_BUMP);
}

fn bump_instance_ttl(env: &Env) {
    env.storage()
        .instance()
        .extend_ttl(INSTANCE_TTL_THRESHOLD, INSTANCE_TTL_BUMP);
}

mod test;
