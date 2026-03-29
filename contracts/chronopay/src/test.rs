#![cfg(test)]

extern crate std;

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env, InvokeError, String};

const START_TIME: u64 = 1_000;
const END_TIME: u64 = 2_000;
const PRICE: i128 = 250;

struct Fixture {
    contract_id: Address,
    seller: Address,
    buyer: Address,
    outsider: Address,
    token_id: u32,
    price: i128,
}

fn install_contract(env: &Env) -> Address {
    env.register(ChronoPayContract, ())
}

fn setup_minted_token(env: &Env) -> Fixture {
    let contract_id = install_contract(env);
    let client = ChronoPayContractClient::new(env, &contract_id);
    let seller = Address::generate(env);
    let buyer = Address::generate(env);
    let outsider = Address::generate(env);

    env.mock_all_auths();
    let slot_id = client.create_time_slot(&seller, &START_TIME, &END_TIME, &PRICE);
    let token_id = client.mint_time_token(&slot_id);
    env.set_auths(&[]);

    Fixture {
        contract_id,
        seller,
        buyer,
        outsider,
        token_id,
        price: PRICE,
    }
}

#[test]
fn test_hello() {
    let env = Env::default();
    let contract_id = install_contract(&env);
    let client = ChronoPayContractClient::new(&env, &contract_id);

    let words = client.hello(&String::from_str(&env, "Dev"));
    assert_eq!(
        words,
        soroban_sdk::vec![
            &env,
            String::from_str(&env, "ChronoPay"),
            String::from_str(&env, "Dev"),
        ]
    );
}

#[test]
fn test_create_time_slot_auto_increments_and_persists_listing_details() {
    let env = Env::default();
    let contract_id = install_contract(&env);
    let client = ChronoPayContractClient::new(&env, &contract_id);
    let seller = Address::generate(&env);

    env.mock_all_auths();
    let slot_id_1 = client.create_time_slot(&seller, &START_TIME, &END_TIME, &PRICE);
    let slot_id_2 =
        client.create_time_slot(&seller, &(START_TIME + 10), &(END_TIME + 10), &(PRICE + 10));

    assert_eq!(slot_id_1, 1);
    assert_eq!(slot_id_2, 2);

    let slot = client.get_time_slot(&slot_id_1);
    assert_eq!(slot.professional, seller);
    assert_eq!(slot.start_time, START_TIME);
    assert_eq!(slot.end_time, END_TIME);
    assert_eq!(slot.price, PRICE);
}

#[test]
fn test_create_time_slot_rejects_invalid_time_ranges_and_prices() {
    let env = Env::default();
    let contract_id = install_contract(&env);
    let client = ChronoPayContractClient::new(&env, &contract_id);
    let seller = Address::generate(&env);

    env.mock_all_auths();

    let invalid_range = client.try_create_time_slot(&seller, &END_TIME, &START_TIME, &PRICE);
    assert_eq!(invalid_range, Err(Ok(ChronoPayError::InvalidTimeRange)));

    let zero_price = client.try_create_time_slot(&seller, &START_TIME, &END_TIME, &0);
    assert_eq!(zero_price, Err(Ok(ChronoPayError::InvalidPrice)));

    let negative_price = client.try_create_time_slot(&seller, &START_TIME, &END_TIME, &-5);
    assert_eq!(negative_price, Err(Ok(ChronoPayError::InvalidPrice)));
}

#[test]
fn test_mint_time_token_copies_listing_price_and_blocks_duplicate_mints() {
    let env = Env::default();
    let contract_id = install_contract(&env);
    let client = ChronoPayContractClient::new(&env, &contract_id);
    let seller = Address::generate(&env);

    env.mock_all_auths();
    let slot_id = client.create_time_slot(&seller, &START_TIME, &END_TIME, &PRICE);
    let token_id = client.mint_time_token(&slot_id);
    let token = client.get_time_token(&token_id);

    assert_eq!(token.slot_id, slot_id);
    assert_eq!(token.seller, seller);
    assert_eq!(token.owner, seller);
    assert_eq!(token.price, PRICE);
    assert_eq!(token.amount_paid, 0);
    assert_eq!(token.status, TimeTokenStatus::Available);

    let duplicate_mint = client.try_mint_time_token(&slot_id);
    assert_eq!(
        duplicate_mint,
        Err(Ok(ChronoPayError::SlotAlreadyTokenized))
    );
}

#[test]
fn test_buy_time_token_requires_buyer_authorization() {
    let env = Env::default();
    let fixture = setup_minted_token(&env);
    let client = ChronoPayContractClient::new(&env, &fixture.contract_id);

    let result = client.try_buy_time_token(
        &fixture.token_id,
        &fixture.buyer,
        &fixture.seller,
        &fixture.price,
    );

    assert_eq!(result, Err(Err(InvokeError::Abort)));
}

#[test]
fn test_buy_time_token_transfers_ownership_after_exact_payment() {
    let env = Env::default();
    let fixture = setup_minted_token(&env);
    let client = ChronoPayContractClient::new(&env, &fixture.contract_id);

    env.mock_all_auths();
    client.buy_time_token(
        &fixture.token_id,
        &fixture.buyer,
        &fixture.seller,
        &fixture.price,
    );

    let auths = env.auths();
    assert_eq!(auths.len(), 1);
    assert_eq!(auths[0].0, fixture.buyer);

    let token = client.get_time_token(&fixture.token_id);
    assert_eq!(token.owner, fixture.buyer);
    assert_eq!(token.amount_paid, fixture.price);
    assert_eq!(token.status, TimeTokenStatus::Sold);
}

#[test]
fn test_buy_time_token_rejects_self_purchase() {
    let env = Env::default();
    let fixture = setup_minted_token(&env);
    let client = ChronoPayContractClient::new(&env, &fixture.contract_id);

    env.mock_all_auths();
    let result = client.try_buy_time_token(
        &fixture.token_id,
        &fixture.seller,
        &fixture.seller,
        &fixture.price,
    );

    assert_eq!(result, Err(Ok(ChronoPayError::BuyerIsSeller)));
}

#[test]
fn test_buy_time_token_rejects_seller_mismatches() {
    let env = Env::default();
    let fixture = setup_minted_token(&env);
    let client = ChronoPayContractClient::new(&env, &fixture.contract_id);

    env.mock_all_auths();
    let result = client.try_buy_time_token(
        &fixture.token_id,
        &fixture.buyer,
        &fixture.outsider,
        &fixture.price,
    );

    assert_eq!(result, Err(Ok(ChronoPayError::SellerMismatch)));
}

#[test]
fn test_buy_time_token_requires_exact_payment() {
    let env = Env::default();
    let fixture = setup_minted_token(&env);
    let client = ChronoPayContractClient::new(&env, &fixture.contract_id);

    env.mock_all_auths();

    let underpayment = client.try_buy_time_token(
        &fixture.token_id,
        &fixture.buyer,
        &fixture.seller,
        &(fixture.price - 1),
    );
    assert_eq!(underpayment, Err(Ok(ChronoPayError::PaymentAmountMismatch)));

    let overpayment = client.try_buy_time_token(
        &fixture.token_id,
        &fixture.buyer,
        &fixture.seller,
        &(fixture.price + 1),
    );
    assert_eq!(overpayment, Err(Ok(ChronoPayError::PaymentAmountMismatch)));

    let token = client.get_time_token(&fixture.token_id);
    assert_eq!(token.owner, fixture.seller);
    assert_eq!(token.amount_paid, 0);
    assert_eq!(token.status, TimeTokenStatus::Available);
}

#[test]
fn test_buy_time_token_blocks_repeat_purchases() {
    let env = Env::default();
    let fixture = setup_minted_token(&env);
    let client = ChronoPayContractClient::new(&env, &fixture.contract_id);

    env.mock_all_auths();
    client.buy_time_token(
        &fixture.token_id,
        &fixture.buyer,
        &fixture.seller,
        &fixture.price,
    );

    let second_buyer = Address::generate(&env);
    let second_purchase = client.try_buy_time_token(
        &fixture.token_id,
        &second_buyer,
        &fixture.seller,
        &fixture.price,
    );
    assert_eq!(second_purchase, Err(Ok(ChronoPayError::TokenAlreadySold)));
}

#[test]
fn test_redeem_time_token_requires_current_owner_after_sale() {
    let env = Env::default();
    let fixture = setup_minted_token(&env);
    let client = ChronoPayContractClient::new(&env, &fixture.contract_id);

    env.mock_all_auths();

    let redeem_before_sale = client.try_redeem_time_token(&fixture.token_id, &fixture.buyer);
    assert_eq!(redeem_before_sale, Err(Ok(ChronoPayError::TokenNotSold)));

    client.buy_time_token(
        &fixture.token_id,
        &fixture.buyer,
        &fixture.seller,
        &fixture.price,
    );

    let outsider_redeem = client.try_redeem_time_token(&fixture.token_id, &fixture.outsider);
    assert_eq!(outsider_redeem, Err(Ok(ChronoPayError::NotTokenOwner)));

    client.redeem_time_token(&fixture.token_id, &fixture.buyer);

    let token = client.get_time_token(&fixture.token_id);
    assert_eq!(token.status, TimeTokenStatus::Redeemed);

    let second_redeem = client.try_redeem_time_token(&fixture.token_id, &fixture.buyer);
    assert_eq!(second_redeem, Err(Ok(ChronoPayError::TokenAlreadyRedeemed)));

    let buy_after_redeem = client.try_buy_time_token(
        &fixture.token_id,
        &Address::generate(&env),
        &fixture.seller,
        &fixture.price,
    );
    assert_eq!(
        buy_after_redeem,
        Err(Ok(ChronoPayError::TokenAlreadyRedeemed))
    );
}
