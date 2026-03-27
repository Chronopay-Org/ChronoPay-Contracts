#![no_std]
//! ChronoPay time token contract — implementation for ownership transfer logic.

use soroban_sdk::{contract, contractimpl, contracttype, vec, Address, Env, String, Symbol, Vec};

/// Represents the possible states of a time token.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TimeTokenStatus {
    Available, // Initial state, can be bought.
    Sold,      // Has been purchased, can be resold or redeemed.
    Redeemed,  // Final state, token service has been consumed.
}

/// The core data structure for a time token slot.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Token {
    pub slot_id: u32,
    pub professional: Address, // The professional who created the slot.
    pub owner: Address,        // Current owner of the token.
    pub status: TimeTokenStatus,
    pub start_time: u64,
    pub end_time: u64,
}

/// Keys used for contract storage.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    SlotSeq,    // u32: auto-incrementing counter for slot IDs.
    Token(u32), // Token: mapping from slot_id to Token data.
}

#[contract]
pub struct ChronoPayContract;

#[contractimpl]
impl ChronoPayContract {
    /// Create a time slot with an auto-incrementing slot id.
    /// Returns the newly assigned slot id.
    /// The professional creating the slot must authorize this call.
    pub fn create_time_slot(
        env: Env,
        professional: Address,
        start_time: u64,
        end_time: u64,
    ) -> u32 {
        // Only the professional can create a slot for themselves.
        professional.require_auth();

        let current_seq: u32 = env
            .storage()
            .instance()
            .get(&DataKey::SlotSeq)
            .unwrap_or(0u32);

        let next_seq = current_seq.checked_add(1).expect("slot id overflow");

        // Initialize the token with the professional as the initial owner.
        let token = Token {
            slot_id: next_seq,
            professional: professional.clone(),
            owner: professional,
            status: TimeTokenStatus::Available,
            start_time,
            end_time,
        };

        // Persist token data and update the sequence.
        env.storage()
            .instance()
            .set(&DataKey::Token(next_seq), &token);
        env.storage().instance().set(&DataKey::SlotSeq, &next_seq);

        next_seq
    }

    /// Mint a time token for a slot (stub).
    /// In this implementation, create_time_slot already initializes the token data.
    /// This function remains for interface compatibility and could be expanded later.
    pub fn mint_time_token(env: Env, slot_id: u32) -> Symbol {
        let key = DataKey::Token(slot_id);
        if !env.storage().instance().has(&key) {
            panic!("token does not exist");
        }

        Symbol::new(&env, "TIME_TOKEN")
    }

    /// Buy / transfer time token.
    /// Logic:
    /// 1. Verify the token exists.
    /// 2. Verify the seller is the current owner.
    /// 3. Require seller's authorization to prevent unauthorized transfers.
    /// 4. Update the owner to the buyer and change status to 'Sold'.
    pub fn buy_time_token(env: Env, slot_id: u32, buyer: Address, seller: Address) -> bool {
        let key = DataKey::Token(slot_id);
        let mut token: Token = env
            .storage()
            .instance()
            .get(&key)
            .expect("token does not exist");

        // SECURITY: Ensure the person claiming to be the seller actually owns the token.
        if token.owner != seller {
            panic!("seller is not the owner");
        }

        // Only available or already sold tokens can be transferred.
        if token.status == TimeTokenStatus::Redeemed {
            panic!("token already redeemed and cannot be bought");
        }

        // SECURITY: Seller MUST authorize the transfer of their token.
        seller.require_auth();

        // Perform ownership transfer.
        token.owner = buyer;
        token.status = TimeTokenStatus::Sold;

        env.storage().instance().set(&key, &token);

        true
    }

    /// Redeem time token.
    /// Marks the token as redeemed, rendering it unusable for further transfers.
    /// Only the current owner can redeem the token.
    pub fn redeem_time_token(env: Env, slot_id: u32) -> bool {
        let key = DataKey::Token(slot_id);
        let mut token: Token = env
            .storage()
            .instance()
            .get(&key)
            .expect("token does not exist");

        // SECURITY: Only the current owner can redeem the token.
        token.owner.require_auth();

        if token.status == TimeTokenStatus::Redeemed {
            panic!("token already redeemed");
        }

        token.status = TimeTokenStatus::Redeemed;
        env.storage().instance().set(&key, &token);

        true
    }

    /// Hello-style entrypoint for CI and SDK sanity check.
    pub fn hello(env: Env, to: String) -> Vec<String> {
        vec![&env, String::from_str(&env, "ChronoPay"), to]
    }
}

mod test;
