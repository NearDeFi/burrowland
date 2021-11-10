mod setup;

use crate::setup::*;
use contract::{BigDecimal, MS_PER_YEAR};

const SEC_PER_YEAR: u32 = (MS_PER_YEAR / 1000) as u32;

#[macro_use]
extern crate approx;

fn basic_setup() -> (Env, Tokens, Users) {
    let e = Env::init();
    let tokens = Tokens::init(&e);
    e.setup_assets(&tokens);
    e.deposit_reserves(&tokens);

    let users = Users::init(&e);
    e.mint_tokens(&tokens, &users.alice);
    storage_deposit(
        &users.alice,
        &e.contract.account_id(),
        &users.alice.account_id(),
        d(1, 23),
    );
    e.mint_tokens(&tokens, &users.bob);
    storage_deposit(
        &users.bob,
        &e.contract.account_id(),
        &users.bob.account_id(),
        d(1, 23),
    );

    (e, tokens, users)
}

#[test]
fn test_init_env() {
    let e = Env::init();
    let _tokens = Tokens::init(&e);
    let _users = Users::init(&e);
}

#[test]
fn test_mint_tokens() {
    let e = Env::init();
    let tokens = Tokens::init(&e);
    let users = Users::init(&e);
    e.mint_tokens(&tokens, &users.alice);
}

#[test]
fn test_dev_setup() {
    let e = Env::init();
    let tokens = Tokens::init(&e);
    e.setup_assets(&tokens);
    e.deposit_reserves(&tokens);

    let asset = e.get_asset(&tokens.wnear);
    assert_eq!(asset.reserved, d(10000, 24));
}

#[test]
fn test_supply() {
    let (e, tokens, users) = basic_setup();

    let amount = d(100, 24);
    e.contract_ft_transfer_call(&tokens.wnear, &users.alice, amount, "")
        .assert_success();

    let asset = e.get_asset(&tokens.wnear);
    assert_eq!(asset.supplied.balance, amount);

    let account = e.get_account(&users.alice);
    assert_eq!(account.supplied[0].balance, amount);
    assert_eq!(account.supplied[0].token_id, tokens.wnear.account_id());
}

#[test]
fn test_supply_to_collateral() {
    let (e, tokens, users) = basic_setup();

    let amount = d(100, 24);
    e.supply_to_collateral(&users.alice, &tokens.wnear, amount)
        .assert_success();

    let asset = e.get_asset(&tokens.wnear);
    assert_eq!(asset.supplied.balance, amount);

    let account = e.get_account(&users.alice);
    assert!(account.supplied.is_empty());
    assert_eq!(account.collateral[0].balance, amount);
    assert_eq!(account.collateral[0].token_id, tokens.wnear.account_id());
}

#[test]
fn test_borrow() {
    let (e, tokens, users) = basic_setup();

    let supply_amount = d(100, 24);
    e.supply_to_collateral(&users.alice, &tokens.wnear, supply_amount)
        .assert_success();

    let borrow_amount = d(200, 18);
    e.borrow(
        &users.alice,
        &tokens.ndai,
        price_data(&tokens, Some(100000), None),
        borrow_amount,
    )
    .assert_success();

    let asset = e.get_asset(&tokens.ndai);
    assert_eq!(asset.borrowed.balance, borrow_amount);
    assert!(asset.borrow_apr > BigDecimal::zero());
    assert_eq!(asset.supplied.balance, borrow_amount);
    assert!(asset.supply_apr > BigDecimal::zero());

    let account = e.get_account(&users.alice);
    assert_eq!(account.supplied[0].balance, borrow_amount);
    assert_eq!(account.supplied[0].token_id, tokens.ndai.account_id());
    assert!(account.supplied[0].apr > BigDecimal::zero());
    assert_eq!(account.borrowed[0].balance, borrow_amount);
    assert_eq!(account.borrowed[0].token_id, tokens.ndai.account_id());
    assert!(account.borrowed[0].apr > BigDecimal::zero());
}

#[test]
fn test_borrow_and_withdraw() {
    let (e, tokens, users) = basic_setup();

    let supply_amount = d(100, 24);
    e.supply_to_collateral(&users.alice, &tokens.wnear, supply_amount)
        .assert_success();

    let borrow_amount = d(200, 18);
    e.borrow_and_withdraw(
        &users.alice,
        &tokens.ndai,
        price_data(&tokens, Some(100000), None),
        borrow_amount,
    )
    .assert_success();

    let asset = e.get_asset(&tokens.ndai);
    assert_eq!(asset.borrowed.balance, borrow_amount);
    assert!(asset.borrow_apr > BigDecimal::zero());
    assert_eq!(asset.supplied.balance, 0);
    assert_eq!(asset.supply_apr, BigDecimal::zero());

    let account = e.get_account(&users.alice);
    assert!(account.supplied.is_empty());
    assert_eq!(account.borrowed[0].balance, borrow_amount);
    assert_eq!(account.borrowed[0].token_id, tokens.ndai.account_id());
    assert!(account.borrowed[0].apr > BigDecimal::zero());
}

#[test]
fn test_interest() {
    let (e, tokens, users) = basic_setup();

    let supply_amount = d(10000, 24);
    e.supply_to_collateral(&users.alice, &tokens.wnear, supply_amount)
        .assert_success();

    let borrow_amount = d(8000, 18);
    e.borrow_and_withdraw(
        &users.alice,
        &tokens.ndai,
        price_data(&tokens, Some(100000), None),
        borrow_amount,
    )
    .assert_success();

    let asset = e.get_asset(&tokens.ndai);
    assert_eq!(asset.borrowed.balance, borrow_amount);
    assert_relative_eq!(asset.borrow_apr.f64(), 0.08f64);

    e.skip_time(SEC_PER_YEAR);

    let expected_borrow_amount = borrow_amount * 108 / 100;

    let asset = e.get_asset(&tokens.ndai);
    assert_relative_eq!(asset.borrowed.balance as f64, expected_borrow_amount as f64);

    let account = e.get_account(&users.alice);
    assert_relative_eq!(
        account.borrowed[0].balance as f64,
        expected_borrow_amount as f64
    );
    assert_eq!(account.borrowed[0].token_id, tokens.ndai.account_id());
}
