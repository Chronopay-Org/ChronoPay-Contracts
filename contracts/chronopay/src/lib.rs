#![no_std]
//! ChronoPay time token contract.
//!
//! Access-control rules:
//! - `mint_time_token` requires the minted owner to authorize the operation.
//! - `buy_time_token` requires both buyer and seller authorization and transfers ownership.
//! - `redeem_time_token` requires the current owner to authorize the operation.
//!
//! Failure modes:
//! - minting for an unknown slot fails
//! - minting the same token twice fails
//! - buying or redeeming a missing token fails
//! - redeeming from a non-owner fails
//! - redeeming an already redeemed token fails

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
    TokenOwner(Symbol),
    TokenStatus(Symbol),
}

#[contracterror]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum ContractError {
    SlotNotFound = 1,
    TokenAlreadyMinted = 2,
    TokenNotFound = 3,
    UnauthorizedRedeemer = 4,
    TokenAlreadyRedeemed = 5,
    SellerDoesNotOwnToken = 6,
}

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

    /// Mint a time token for an existing slot.
    pub fn mint_time_token(
        env: Env,
        slot_id: u32,
        owner: Address,
    ) -> Result<Symbol, ContractError> {
        owner.require_auth();
        Self::require_existing_slot(&env, slot_id)?;

        let token_id = Self::time_token_symbol(&env);
        if env
            .storage()
            .instance()
            .has(&DataKey::TokenOwner(token_id.clone()))
        {
            return Err(ContractError::TokenAlreadyMinted);
        }

        env.storage()
            .instance()
            .set(&DataKey::TokenOwner(token_id.clone()), &owner);
        env.storage().instance().set(
            &DataKey::TokenStatus(token_id.clone()),
            &TimeTokenStatus::Available,
        );

        Ok(token_id)
    }

    /// Transfer a time token from the current owner to a buyer.
    pub fn buy_time_token(
        env: Env,
        token_id: Symbol,
        buyer: Address,
        seller: Address,
    ) -> Result<bool, ContractError> {
        buyer.require_auth();
        seller.require_auth();

        let current_owner = Self::read_owner(&env, &token_id)?;
        if current_owner != seller {
            return Err(ContractError::SellerDoesNotOwnToken);
        }

        let status = Self::read_status(&env, &token_id)?;
        if status == TimeTokenStatus::Redeemed {
            return Err(ContractError::TokenAlreadyRedeemed);
        }

        env.storage()
            .instance()
            .set(&DataKey::TokenOwner(token_id.clone()), &buyer);
        env.storage()
            .instance()
            .set(&DataKey::TokenStatus(token_id), &TimeTokenStatus::Sold);

        Ok(true)
    }

    /// Redeem a time token.
    ///
    /// Only the current token owner may redeem. A redeemed token cannot be redeemed again.
    pub fn redeem_time_token(
        env: Env,
        token_id: Symbol,
        redeemer: Address,
    ) -> Result<bool, ContractError> {
        redeemer.require_auth();

        let owner = Self::read_owner(&env, &token_id)?;
        if owner != redeemer {
            return Err(ContractError::UnauthorizedRedeemer);
        }

        let status = Self::read_status(&env, &token_id)?;
        if status == TimeTokenStatus::Redeemed {
            return Err(ContractError::TokenAlreadyRedeemed);
        }

        env.storage()
            .instance()
            .set(&DataKey::TokenStatus(token_id), &TimeTokenStatus::Redeemed);

        Ok(true)
    }

    /// Hello-style entrypoint for CI and SDK sanity check.
    pub fn hello(env: Env, to: String) -> Vec<String> {
        vec![&env, String::from_str(&env, "ChronoPay"), to]
    }

    fn time_token_symbol(env: &Env) -> Symbol {
        Symbol::new(env, "TIME_TOKEN")
    }

    fn require_existing_slot(env: &Env, slot_id: u32) -> Result<(), ContractError> {
        let current_seq: u32 = env
            .storage()
            .instance()
            .get(&DataKey::SlotSeq)
            .unwrap_or(0u32);
        if slot_id == 0 || slot_id > current_seq {
            return Err(ContractError::SlotNotFound);
        }

        Ok(())
    }

    fn read_owner(env: &Env, token_id: &Symbol) -> Result<Address, ContractError> {
        env.storage()
            .instance()
            .get(&DataKey::TokenOwner(token_id.clone()))
            .ok_or(ContractError::TokenNotFound)
    }

    fn read_status(env: &Env, token_id: &Symbol) -> Result<TimeTokenStatus, ContractError> {
        env.storage()
            .instance()
            .get(&DataKey::TokenStatus(token_id.clone()))
            .ok_or(ContractError::TokenNotFound)
    }
}

mod test;
