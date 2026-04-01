use soroban_sdk::{contractevent, Env, String, Symbol};

/// Emitted when a new time slot is created.
#[contractevent(topics = ["slot_created"])]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SlotCreated {
    #[topic]
    pub professional: String,
    pub slot_id: u32,
    pub start_time: u64,
    pub end_time: u64,
}

/// Emitted when a time token is minted for a slot.
#[contractevent(topics = ["token_minted"])]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TokenMinted {
    #[topic]
    pub slot_id: u32,
    pub token_id: Symbol,
}

/// Emitted when a time token is purchased.
#[contractevent(topics = ["token_purchased"])]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TokenPurchased {
    #[topic]
    pub token_id: Symbol,
    pub buyer: String,
    pub seller: String,
}

/// Emitted when a time token is redeemed.
#[contractevent(topics = ["token_redeemed"])]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TokenRedeemed {
    #[topic]
    pub token_id: Symbol,
}

pub fn emit_slot_created(
    env: &Env,
    professional: &String,
    slot_id: u32,
    start_time: u64,
    end_time: u64,
) {
    SlotCreated {
        professional: professional.clone(),
        slot_id,
        start_time,
        end_time,
    }
    .publish(env);
}

pub fn emit_token_minted(env: &Env, slot_id: u32, token_id: &Symbol) {
    TokenMinted {
        slot_id,
        token_id: token_id.clone(),
    }
    .publish(env);
}

pub fn emit_token_purchased(env: &Env, token_id: &Symbol, buyer: &String, seller: &String) {
    TokenPurchased {
        token_id: token_id.clone(),
        buyer: buyer.clone(),
        seller: seller.clone(),
    }
    .publish(env);
}

pub fn emit_token_redeemed(env: &Env, token_id: &Symbol) {
    TokenRedeemed {
        token_id: token_id.clone(),
    }
    .publish(env);
}
