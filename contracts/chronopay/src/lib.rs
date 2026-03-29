#![no_std]
//! ChronoPay time token contract — stub for create_time_slot, mint_time_token, buy_time_token, redeem_time_token.
//! Includes Fee distribution logic for settlement.

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, vec, Env, String, Symbol, Vec,
};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    InvalidFeeRate = 1,
    CalculationOverflow = 2,
}

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
    Owner,
    Status,
    FeeRateBps,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SettlementResult {
    pub professional_amount: i128,
    pub protocol_fee: i128,
}

#[contract]
pub struct ChronoPayContract;

#[contractimpl]
impl ChronoPayContract {
    /// Configure the global protocol fee rate in basis points (1 = 0.01%).
    /// Max limit is 10000 (100%).
    pub fn set_fee_rate(env: Env, rate_bps: u32) -> Result<(), Error> {
        if rate_bps > 10000 {
            return Err(Error::InvalidFeeRate);
        }
        env.storage()
            .instance()
            .set(&DataKey::FeeRateBps, &rate_bps);
        Ok(())
    }

    /// Retrieve the current global protocol fee rate in basis points.
    /// Defaults to 0 if not configured.
    pub fn get_fee_rate(env: Env) -> u32 {
        env.storage()
            .instance()
            .get(&DataKey::FeeRateBps)
            .unwrap_or(0)
    }

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

    /// Buy / transfer time token (stub).
    pub fn buy_time_token(env: Env, token_id: Symbol, buyer: String, seller: String) -> bool {
        let _ = (token_id, buyer, seller);
        env.storage()
            .instance()
            .set(&DataKey::Owner, &env.current_contract_address());
        true
    }

    /// Redeem time token (stub) and distribute settlement fees.
    /// Calculates the protocol fee from the `settlement_amount` based on the active `fee_rate`.
    pub fn redeem_time_token(
        env: Env,
        token_id: Symbol,
        settlement_amount: i128,
    ) -> SettlementResult {
        let _ = token_id;

        // Ensure atomic state transaction bounds locally
        if settlement_amount < 0 {
            panic!("settlement amount cannot be negative");
        }

        env.storage()
            .instance()
            .set(&DataKey::Status, &TimeTokenStatus::Redeemed);

        let fee_bps = Self::get_fee_rate(env.clone());
        let protocol_fee = (settlement_amount * fee_bps as i128) / 10000;
        let professional_amount = settlement_amount - protocol_fee;

        SettlementResult {
            professional_amount,
            protocol_fee,
        }
    }

    /// Hello-style entrypoint for CI and SDK sanity check.
    pub fn hello(env: Env, to: String) -> Vec<String> {
        vec![&env, String::from_str(&env, "ChronoPay"), to]
    }
}

mod test;
