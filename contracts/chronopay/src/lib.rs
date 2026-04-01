#![no_std]
//! ChronoPay time token contract.
//! Adds production-ready token metadata for time NFTs with validation, storage, and retrieval helpers.

extern crate alloc;

use alloc::format;
use soroban_sdk::{contract, contractimpl, contracttype, vec, Address, Env, String, Symbol, Vec};

/// Errors that can occur during contract execution.
#[contracterror]
#[derive(Clone, Debug, Eq, PartialEq, Copy)]
#[repr(u32)]
pub enum ChronoPayError {
    /// The caller is not authorized to perform this operation.
    Unauthorized = 1,
    /// The professional is not authorized to create time slots.
    ProfessionalNotAuthorized = 2,
    /// The professional is already authorized.
    AlreadyAuthorized = 3,
    /// The professional is not found in the registry.
    ProfessionalNotFound = 4,
    /// Invalid input parameters provided.
    InvalidInput = 5,
    /// Admin operation failed.
    AdminError = 6,
}

/// Status of a time token.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TimeTokenStatus {
    Available,
    Sold,
    Redeemed,
}

/// Persistent and instance storage keys used by the contract.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TimeSlot {
    pub professional: Address,
    pub start_time: u64,
    pub end_time: u64,
    pub minted: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    Admin,
    CollectionMetadata,
    SlotSeq,
    TokenSeq,
    Slot(u32),
    Token(Symbol),
    Owner,
    Status,
    /// Stores the admin Address for pause/unpause authorization.
    Admin,
    /// Stores the paused state as a bool.
    Paused,
}

/// Contract-level metadata for the NFT collection.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CollectionMetadata {
    pub name: String,
    pub symbol: String,
}

/// Detailed metadata for an individual token following production standards.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TokenMetadata {
    pub name: String,
    pub description: String,
    pub image_uri: String,
}

/// Data representing a scheduled time slot.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TimeSlot {
    pub professional: Address,
    pub start_time: u64,
    pub end_time: u64,
    pub token: Option<Symbol>,
}

/// Metadata stored for every minted time NFT.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TimeTokenMetadata {
    pub token_id: Symbol,
    pub slot_id: u32,
    pub professional: Address,
    pub start_time: u64,
    pub end_time: u64,
    pub status: TimeTokenStatus,
    pub current_owner: Address,
    pub metadata: TokenMetadata,
}

// ---------------------------------------------------------------------------
// Contract
// ---------------------------------------------------------------------------

#[contract]
pub struct ChronoPayContract;

#[contractimpl]
impl ChronoPayContract {
    /// Initialize the contract with admin and collection metadata.
    pub fn initialize(env: Env, admin: Address, name: String, symbol: String) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("already initialized");
        }
        env.storage().instance().set(&DataKey::Admin, &admin);

        let metadata = CollectionMetadata { name, symbol };
        env.storage()
            .instance()
            .set(&DataKey::CollectionMetadata, &metadata);
    }

    /// Create a time slot and persist it using persistent storage.
    pub fn create_time_slot(
        env: Env,
        professional: Address,
        start_time: u64,
        end_time: u64,
    ) -> u32 {
        professional.require_auth();

        if start_time >= end_time {
            panic!("end_time must be after start_time");
        }

        let slot_id = next_sequence(&env, DataKey::SlotSeq);
        let slot = TimeSlot {
            professional: professional.clone(),
            start_time,
            end_time,
            token: None,
        };

        // Use persistent storage for individual slots to handle scaling
        env.storage()
            .persistent()
            .set(&DataKey::Slot(slot_id), &slot);

        env.events()
            .publish((Symbol::new(&env, "slot_created"), professional), slot_id);

        slot_id
    }

    /// Mint a time token for a slot with detailed metadata.
    pub fn mint_time_token(env: Env, slot_id: u32, metadata: TokenMetadata) -> Symbol {
        let mut slot: TimeSlot = env
            .storage()
            .persistent()
            .get(&DataKey::Slot(slot_id))
            .expect("slot does not exist");

        slot.professional.require_auth();

        if slot.token.is_some() {
            panic!("token already minted for slot");
        }

        let token_id = next_sequence(&env, DataKey::TokenSeq);
        let token_symbol = build_token_symbol(&env, token_id);

        let time_token_metadata = TimeTokenMetadata {
            token_id: token_symbol.clone(),
            slot_id,
            professional: slot.professional.clone(),
            start_time: slot.start_time,
            end_time: slot.end_time,
            status: TimeTokenStatus::Available,
            current_owner: slot.professional.clone(),
            metadata,
        };

        env.storage()
            .persistent()
            .set(&DataKey::Token(token_symbol.clone()), &time_token_metadata);

        slot.token = Some(token_symbol.clone());
        env.storage()
            .persistent()
            .set(&DataKey::Slot(slot_id), &slot);

        env.events().publish(
            (Symbol::new(&env, "token_minted"), slot.professional),
            (token_symbol.clone(), slot_id),
        );

        token_symbol
    }

    /// Buy / transfer a time token from seller to buyer.
    pub fn buy_time_token(env: Env, token_id: Symbol, buyer: Address) -> bool {
        buyer.require_auth();

        let mut metadata: TimeTokenMetadata = env
            .storage()
            .persistent()
            .get(&DataKey::Token(token_id.clone()))
            .expect("unknown token");

        if metadata.status == TimeTokenStatus::Redeemed {
            panic!("token already redeemed");
        }

        if metadata.current_owner == buyer {
            panic!("buyer is already the owner");
        }

        let old_owner = metadata.current_owner.clone();
        metadata.current_owner = buyer.clone();
        metadata.status = TimeTokenStatus::Sold;

        env.storage()
            .persistent()
            .set(&DataKey::Token(token_id.clone()), &metadata);

        env.events().publish(
            (Symbol::new(&env, "token_bought"), token_id),
            (old_owner, buyer),
        );

        // Stub logic for backward compatibility from main
        env.storage()
            .instance()
            .set(&DataKey::Owner, &env.current_contract_address());

        true
    }

    /// Redeem a time token, marking it as consumed.
    pub fn redeem_time_token(env: Env, token_id: Symbol) -> bool {
        let mut metadata: TimeTokenMetadata = env
            .storage()
            .persistent()
            .get(&DataKey::Token(token_id.clone()))
            .expect("unknown token");

        metadata.current_owner.require_auth();

        if metadata.status == TimeTokenStatus::Redeemed {
            panic!("token already redeemed");
        }

        metadata.status = TimeTokenStatus::Redeemed;

        env.storage()
            .persistent()
            .set(&DataKey::Token(token_id.clone()), &metadata);

        env.events().publish(
            (Symbol::new(&env, "token_redeemed"), token_id),
            metadata.current_owner,
        );

        // Stub logic for backward compatibility from main
        env.storage()
            .instance()
            .set(&DataKey::Status, &TimeTokenStatus::Redeemed);

        true
    }

    pub fn get_token_metadata(env: Env, token_id: Symbol) -> Option<TimeTokenMetadata> {
        env.storage().persistent().get(&DataKey::Token(token_id))
    }

    pub fn get_time_slot(env: Env, slot_id: u32) -> Option<TimeSlot> {
        env.storage().persistent().get(&DataKey::Slot(slot_id))
    }

    pub fn get_collection_metadata(env: Env) -> Option<CollectionMetadata> {
        env.storage().instance().get(&DataKey::CollectionMetadata)
    }

    pub fn hello(env: Env, to: String) -> Vec<String> {
        vec![&env, String::from_str(&env, "ChronoPay"), to]
    }

    /// Panics if the contract is paused.
    fn require_not_paused(env: &Env) {
        let paused: bool = env
            .storage()
            .instance()
            .get(&DataKey::Paused)
            .unwrap_or(false);
        if paused {
            panic!("contract is paused");
        }
    }

    /// Panics if the provided address is not the stored admin.
    fn require_admin(env: &Env, caller: &Address) {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .expect("not initialized");
        if admin != *caller {
            panic!("unauthorized: caller is not admin");
        }
    }
}

fn next_sequence(env: &Env, key: DataKey) -> u32 {
    let next = env
        .storage()
        .instance()
        .get(&key)
        .unwrap_or(0u32)
        .saturating_add(1);
    env.storage().instance().set(&key, &next);
    next
}

fn build_token_symbol(env: &Env, token_id: u32) -> Symbol {
    let token_label = format!("TIME_{}", token_id);
    Symbol::new(env, &token_label)
}

#[cfg(test)]
mod test;
