#![cfg(test)]

use super::*;
use soroban_sdk::{vec, Address, Env, String, Symbol};

struct IntegrationHarness {
    env: Env,
    contract_id: Address,
}

impl IntegrationHarness {
    fn new() -> Self {
        let env = Env::default();
        let contract_id = env.register(ChronoPayContract, ());

        Self { env, contract_id }
    }

    fn client(&self) -> ChronoPayContractClient<'_> {
        ChronoPayContractClient::new(&self.env, &self.contract_id)
    }

    fn hello(&self, to: &str) -> soroban_sdk::Vec<String> {
        self.client().hello(&String::from_str(&self.env, to))
    }

    fn create_slot(&self, professional: &str, start: u64, end: u64) -> u32 {
        self.client()
            .create_time_slot(&String::from_str(&self.env, professional), &start, &end)
    }

    fn mint(&self, slot_id: u32) -> Symbol {
        self.client().mint_time_token(&slot_id)
    }

    fn buy(&self, token: &Symbol, buyer: &str, seller: &str) -> bool {
        self.client().buy_time_token(
            token,
            &String::from_str(&self.env, buyer),
            &String::from_str(&self.env, seller),
        )
    }

    fn redeem(&self, token: &Symbol) -> bool {
        self.client().redeem_time_token(token)
    }

    fn owner(&self) -> Option<String> {
        self.client().current_owner()
    }

    fn status(&self) -> Option<TimeTokenStatus> {
        self.client().current_status()
    }
}

#[test]
fn test_hello() {
    let h = IntegrationHarness::new();

    let words = h.hello("Dev");
    assert_eq!(
        words,
        vec![
            &h.env,
            String::from_str(&h.env, "ChronoPay"),
            String::from_str(&h.env, "Dev"),
        ]
    );
}

#[test]
fn test_create_time_slot_auto_increments() {
    let h = IntegrationHarness::new();

    let slot_id_1 = h.create_slot("professional_alice", 1000, 2000);
    let slot_id_2 = h.create_slot("professional_alice", 3000, 4000);
    let slot_id_3 = h.create_slot("professional_alice", 5000, 6000);

    assert_eq!(slot_id_1, 1);
    assert_eq!(slot_id_2, 2);
    assert_eq!(slot_id_3, 3);
}

#[test]
fn test_mint_and_redeem() {
    let h = IntegrationHarness::new();

    let slot_id = h.create_slot("pro", 1000, 2000);
    let token = h.mint(slot_id);
    assert_eq!(token, Symbol::new(&h.env, "TIME_TOKEN"));

    assert!(h.buy(&token, "buyer_bob", "seller_alice"));
    assert!(h.redeem(&token));
    assert_eq!(h.status(), Some(TimeTokenStatus::Redeemed));
}

#[test]
fn test_buy_updates_owner_and_status() {
    let h = IntegrationHarness::new();
    let token = Symbol::new(&h.env, "TIME_TOKEN");

    let result = h.buy(&token, "buyer_bob", "seller_alice");
    assert!(result);
    assert_eq!(h.owner(), Some(String::from_str(&h.env, "buyer_bob")));
    assert_eq!(h.status(), Some(TimeTokenStatus::Sold));
}

#[test]
#[should_panic(expected = "professional cannot be empty")]
fn test_create_slot_rejects_empty_professional() {
    let h = IntegrationHarness::new();
    let _ = h.create_slot("", 1000, 2000);
}

#[test]
#[should_panic(expected = "start_time must be before end_time")]
fn test_create_slot_rejects_invalid_time_range() {
    let h = IntegrationHarness::new();
    let _ = h.create_slot("professional_alice", 2000, 2000);
}

#[test]
#[should_panic(expected = "slot id must be non-zero")]
fn test_mint_rejects_zero_slot_id() {
    let h = IntegrationHarness::new();
    let _ = h.mint(0);
}

#[test]
#[should_panic(expected = "unsupported token id")]
fn test_buy_rejects_unknown_token() {
    let h = IntegrationHarness::new();
    let unknown = Symbol::new(&h.env, "UNKNOWN");
    let _ = h.buy(&unknown, "buyer_bob", "seller_alice");
}

#[test]
#[should_panic(expected = "buyer and seller must differ")]
fn test_buy_rejects_self_trade() {
    let h = IntegrationHarness::new();
    let token = Symbol::new(&h.env, "TIME_TOKEN");
    let _ = h.buy(&token, "same_user", "same_user");
}

#[test]
#[should_panic(expected = "token must be sold before redemption")]
fn test_redeem_requires_sold_state() {
    let h = IntegrationHarness::new();
    let token = Symbol::new(&h.env, "TIME_TOKEN");
    let _ = h.redeem(&token);
}

#[test]
#[should_panic(expected = "token must be sold before redemption")]
fn test_redeem_rejects_replay() {
    let h = IntegrationHarness::new();
    let slot_id = h.create_slot("pro", 1000, 2000);
    let token = h.mint(slot_id);
    assert!(h.buy(&token, "buyer_bob", "seller_alice"));
    assert!(h.redeem(&token));

    let _ = h.redeem(&token);
}
