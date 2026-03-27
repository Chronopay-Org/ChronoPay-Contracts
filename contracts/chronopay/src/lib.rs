#![no_std]
//! ChronoPay time token contract — stub for create_time_slot, mint_time_token, buy_time_token, redeem_time_token.

mod domain;
#[cfg(test)]
mod test;

pub use domain::{DataKey, TimeTokenStatus};

use soroban_sdk::{contract, contractimpl, vec, Env, String, Symbol, Vec};

const CONTRACT_NAME: &str = "ChronoPay";
const TIME_TOKEN_SYMBOL: &str = "TIME_TOKEN";

#[contract]
pub struct ChronoPayContract;

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
        load_or_init_time_token_symbol(&env)
    }

    /// Buy / transfer time token (stub). In full implementation: token_id, buyer, seller, price.
    pub fn buy_time_token(env: Env, token_id: Symbol, buyer: String, seller: String) -> bool {
        let _ = (token_id, buyer, seller);
        env.storage()
            .instance()
            .set(&DataKey::Owner, &env.current_contract_address());
        true
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
        vec![&env, load_or_init_contract_name(&env), to]
    }
}

fn load_or_init_contract_name(env: &Env) -> String {
    let storage = env.storage().instance();
    if let Some(contract_name) = storage.get::<DataKey, String>(&DataKey::ContractName) {
        return contract_name;
    }

    // Cache immutable metadata so repeat calls do not rebuild the same host
    // string object from a Rust literal.
    let contract_name = String::from_str(env, CONTRACT_NAME);
    storage.set(&DataKey::ContractName, &contract_name);
    contract_name
}

fn load_or_init_time_token_symbol(env: &Env) -> Symbol {
    let storage = env.storage().instance();
    if let Some(token_symbol) = storage.get::<DataKey, Symbol>(&DataKey::TokenSymbol) {
        return token_symbol;
    }

    let token_symbol = Symbol::new(env, TIME_TOKEN_SYMBOL);
    storage.set(&DataKey::TokenSymbol, &token_symbol);
    token_symbol
}
