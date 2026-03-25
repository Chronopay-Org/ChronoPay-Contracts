#![no_std]
//! ChronoPay time token contract — stub for create_time_slot, mint_time_token, buy_time_token, redeem_time_token.

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

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    SlotSeq,
    Owner(Symbol),
    Status,
}

/// Error codes for ChronoPay contract operations
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum Error {
    AlreadyInitialized = 1,   // Contract has already been initialized
    TokenNotFound = 2,        // Token does not exist in storage
    NotTokenOwner = 3,        // Caller is not the token owner
    TokenAlreadyRedeemed = 4, // Token has already been redeemed
    StartTimeInPast = 5,      // Start time is in the past
    InvalidTimeRange = 6,     // End time is not greater than start time
}

#[contract]
pub struct ChronoPayContract;

#[contractimpl]
impl ChronoPayContract {
    /// Create a time slot with an auto-incrementing slot id.
    /// Returns the newly assigned slot id.
    pub fn create_time_slot(
        env: Env,
        professional: String,
        start_time: u64,
        end_time: u64,
    ) -> Result<u32, Error> {
        let current_time = env.ledger().timestamp();

        if start_time <= current_time {
            return Err(Error::StartTimeInPast);
        }

        if end_time <= start_time {
            return Err(Error::InvalidTimeRange);
        }

        let _ = professional;

        let current_seq: u32 = env
            .storage()
            .instance()
            .get(&DataKey::SlotSeq)
            .unwrap_or(0u32);

        let next_seq = current_seq.checked_add(1).expect("slot id overflow");

        env.storage().instance().set(&DataKey::SlotSeq, &next_seq);

        Ok(next_seq)
    }

    /// Mint a time token for a slot (stub).
    pub fn mint_time_token(env: Env, slot_id: u32) -> Symbol {
        let _ = slot_id;
        Symbol::new(&env, "TIME_TOKEN")
    }

    /// Buy / transfer time token (stub). In full implementation: token_id, buyer, seller, price.
    pub fn buy_time_token(
        env: Env,
        token_id: Symbol,
        buyer: Address,
        seller: Address,
    ) -> Result<bool, Error> {
        let _ = seller;

        env.storage()
            .instance()
            .set(&DataKey::Owner(token_id.clone()), &buyer);
        Ok(true)
    }

    /// Redeem time token. Verifies ownership via require_auth and marks as redeemed.
    pub fn redeem_time_token(env: Env, token_id: Symbol) -> Result<bool, Error> {
        let owner: Address = env
            .storage()
            .instance()
            .get(&DataKey::Owner(token_id.clone()))
            .ok_or(Error::TokenNotFound)?;

        owner.require_auth();

        env.storage()
            .instance()
            .set(&DataKey::Status, &TimeTokenStatus::Redeemed);
        Ok(true)
    }

    /// Hello-style entrypoint for CI and SDK sanity check.
    pub fn hello(env: Env, to: String) -> Vec<String> {
        vec![&env, String::from_str(&env, "ChronoPay"), to]
    }
}

mod test;
