mod setup;

use crate::setup::*;
use contract::BigDecimal;
use near_sdk_sim::transaction::ExecutionStatus;

/// Alice puts 1000 USDC and borrows 50 NEAR at 10$. Prices go up. REKT
/// Bob liquidates Alice and makes nice profit.
#[test]
fn test_liquidation_alice_by_bob() {
    let (e, tokens, users) = basic_setup();

    let extra_decimals_mult = d(1, 12);

    let supply_amount = d(1000, 18);
    e.supply_to_collateral(
        &users.alice,
        &tokens.nusdc,
        supply_amount / extra_decimals_mult,
    )
    .assert_success();

    let borrow_amount = d(50, 24);
    e.borrow_and_withdraw(
        &users.alice,
        &tokens.wnear,
        price_data(&tokens, Some(100000), None),
        borrow_amount,
    )
    .assert_success();

    let account = e.get_account(&users.alice);
    assert!(account.supplied.is_empty());
    assert_eq!(account.collateral.len(), 1);
    assert_eq!(account.collateral[0].token_id, tokens.nusdc.account_id());
    assert_eq!(account.collateral[0].balance, supply_amount);
    assert_eq!(account.borrowed.len(), 1);
    assert_eq!(account.borrowed[0].token_id, tokens.wnear.account_id());
    assert_eq!(account.borrowed[0].balance, borrow_amount);
    assert!(account.borrowed[0].apr > BigDecimal::zero());

    let bobs_amount = d(100, 24);
    e.contract_ft_transfer_call(&tokens.wnear, &users.bob, bobs_amount, "")
        .assert_success();

    let account = e.get_account(&users.bob);
    assert_eq!(account.supplied.len(), 1);
    assert_eq!(account.supplied[0].token_id, tokens.wnear.account_id());
    assert_eq!(account.supplied[0].balance, bobs_amount);
    assert!(account.supplied[0].apr > BigDecimal::zero());

    // Assuming 2% discount for 5 NEAR at 12$.
    let wnear_amount_in = d(49, 23);
    let usdc_amount_out = d(60, 18);
    let res = e.liquidate(
        &users.bob,
        &users.alice,
        price_data(&tokens, Some(120000), None),
        vec![asset_amount(&tokens.wnear, wnear_amount_in)],
        vec![asset_amount(&tokens.nusdc, usdc_amount_out)],
    );
    res.assert_success();
    // println!("{:#?}", res.logs());

    let account = e.get_account(&users.alice);
    assert!(account.supplied.is_empty());
    assert_eq!(account.collateral.len(), 1);
    assert_eq!(account.collateral[0].token_id, tokens.nusdc.account_id());
    assert_eq!(
        account.collateral[0].balance,
        supply_amount - usdc_amount_out
    );
    assert_eq!(account.borrowed.len(), 1);
    assert_eq!(account.borrowed[0].token_id, tokens.wnear.account_id());
    assert_eq!(account.borrowed[0].balance, borrow_amount - wnear_amount_in);
    assert!(account.borrowed[0].apr > BigDecimal::zero());

    let account = e.get_account(&users.bob);
    assert_eq!(account.supplied.len(), 2);
    assert_eq!(account.supplied[0].token_id, tokens.wnear.account_id());
    assert_eq!(account.supplied[0].balance, bobs_amount - wnear_amount_in);
    assert!(account.supplied[0].apr > BigDecimal::zero());
    assert_eq!(account.supplied[1].token_id, tokens.nusdc.account_id());
    assert_eq!(account.supplied[1].balance, usdc_amount_out);
    assert_eq!(account.supplied[1].apr, BigDecimal::zero());
}

/// Bob attemps to liquidate Alice which decreases health factor.
#[test]
fn test_liquidation_decrease_health_factor() {
    let (e, tokens, users) = basic_setup();

    let extra_decimals_mult = d(1, 12);

    let supply_amount = d(1000, 18);
    e.supply_to_collateral(
        &users.alice,
        &tokens.nusdc,
        supply_amount / extra_decimals_mult,
    )
    .assert_success();

    let wnear_borrow_amount = d(50, 24);
    e.borrow_and_withdraw(
        &users.alice,
        &tokens.wnear,
        price_data(&tokens, Some(100000), None),
        wnear_borrow_amount,
    )
    .assert_success();

    let usdt_borrow_amount = d(50, 18);
    e.borrow_and_withdraw(
        &users.alice,
        &tokens.nusdt,
        price_data(&tokens, Some(100000), None),
        usdt_borrow_amount,
    )
    .assert_success();

    let account = e.get_account(&users.alice);
    assert!(account.supplied.is_empty());
    assert_eq!(account.collateral.len(), 1);
    assert_eq!(account.collateral[0].token_id, tokens.nusdc.account_id());
    assert_eq!(account.collateral[0].balance, supply_amount);
    assert_eq!(account.borrowed.len(), 2);
    assert_eq!(account.borrowed[0].token_id, tokens.wnear.account_id());
    assert_eq!(account.borrowed[0].balance, wnear_borrow_amount);
    assert!(account.borrowed[0].apr > BigDecimal::zero());
    assert_eq!(account.borrowed[1].token_id, tokens.nusdt.account_id());
    assert_eq!(account.borrowed[1].balance, usdt_borrow_amount);
    assert!(account.borrowed[1].apr > BigDecimal::zero());

    let wnear_bobs_amount = d(100, 24);
    e.contract_ft_transfer_call(&tokens.wnear, &users.bob, wnear_bobs_amount, "")
        .assert_success();

    let usdt_bobs_amount = d(100, 18);
    e.contract_ft_transfer_call(
        &tokens.nusdt,
        &users.bob,
        usdt_bobs_amount / extra_decimals_mult,
        "",
    )
    .assert_success();

    let account = e.get_account(&users.bob);
    assert_eq!(account.supplied.len(), 2);
    assert_eq!(account.supplied[0].token_id, tokens.wnear.account_id());
    assert_eq!(account.supplied[0].balance, wnear_bobs_amount);
    assert!(account.supplied[0].apr > BigDecimal::zero());
    assert_eq!(account.supplied[1].token_id, tokens.nusdt.account_id());
    assert_eq!(account.supplied[1].balance, usdt_bobs_amount);
    assert!(account.supplied[1].apr > BigDecimal::zero());

    // Assuming 2% discount for NEAR at 12$. Paying 49 USDT for 50 USDC.
    let usdt_amount_in = d(49, 18);
    let usdc_amount_out = d(50, 18);
    let res = e.liquidate(
        &users.bob,
        &users.alice,
        price_data(&tokens, Some(120000), None),
        vec![asset_amount(&tokens.nusdt, usdt_amount_in)],
        vec![asset_amount(&tokens.nusdc, usdc_amount_out)],
    );
    let err = match res.status() {
        ExecutionStatus::Failure(e) => e.to_string(),
        _ => panic!("Should fail with liquidation health error"),
    };
    assert!(err.contains("The health factor of liquidation account can't decrease."));

    // Assuming ~2% discount for 5 NEAR at 12$. 50 USDT -> ~51 USDC, 4.9 NEAR -> 60 USDC.
    let wnear_amount_in = d(49, 23);
    let usdt_amount_in = d(50, 18);
    let usdc_amount_out = d(111, 18);
    let res = e.liquidate(
        &users.bob,
        &users.alice,
        price_data(&tokens, Some(120000), None),
        vec![
            asset_amount(&tokens.wnear, wnear_amount_in),
            asset_amount(&tokens.nusdt, usdt_amount_in),
        ],
        vec![asset_amount(&tokens.nusdc, usdc_amount_out)],
    );
    res.assert_success();
    // println!("{:#?}", res.logs());

    let account = e.get_account(&users.alice);
    assert!(account.supplied.is_empty());
    assert_eq!(account.collateral.len(), 1);
    assert_eq!(account.collateral[0].token_id, tokens.nusdc.account_id());
    assert_eq!(
        account.collateral[0].balance,
        supply_amount - usdc_amount_out
    );
    assert_eq!(account.borrowed.len(), 1);
    assert_eq!(account.borrowed[0].token_id, tokens.wnear.account_id());
    assert_eq!(
        account.borrowed[0].balance,
        wnear_borrow_amount - wnear_amount_in
    );
    assert!(account.borrowed[0].apr > BigDecimal::zero());

    let account = e.get_account(&users.bob);
    assert_eq!(account.supplied.len(), 3);
    assert_eq!(account.supplied[0].token_id, tokens.wnear.account_id());
    assert_eq!(
        account.supplied[0].balance,
        wnear_bobs_amount - wnear_amount_in
    );
    assert!(account.supplied[0].apr > BigDecimal::zero());
    assert_eq!(account.supplied[1].token_id, tokens.nusdt.account_id());
    assert_eq!(
        account.supplied[1].balance,
        usdt_bobs_amount - usdt_amount_in
    );
    // Not APR should be 0, since Bob has liquidated the entire USDT amount
    assert_eq!(account.supplied[1].apr, BigDecimal::zero());
    assert_eq!(account.supplied[2].token_id, tokens.nusdc.account_id());
    assert_eq!(account.supplied[2].balance, usdc_amount_out);
    assert_eq!(account.supplied[2].apr, BigDecimal::zero());
}

/// Force closing the account with bad debt.
#[test]
fn test_force_close() {
    let (e, tokens, users) = basic_setup();

    let extra_decimals_mult = d(1, 12);

    let supply_amount = d(1000, 18);
    e.supply_to_collateral(
        &users.alice,
        &tokens.nusdc,
        supply_amount / extra_decimals_mult,
    )
    .assert_success();

    let borrow_amount = d(50, 24);
    e.borrow_and_withdraw(
        &users.alice,
        &tokens.wnear,
        price_data(&tokens, Some(100000), None),
        borrow_amount,
    )
    .assert_success();

    let asset = e.get_asset(&tokens.nusdc);
    let usdc_reserve = asset.reserved;

    let asset = e.get_asset(&tokens.wnear);
    let wnear_reserve = asset.reserved;

    // Attempt to force close the account with NEAR at 12$, the account debt is still not bad.
    let res = e.force_close(
        &users.bob,
        &users.alice,
        price_data(&tokens, Some(120000), None),
    );
    let err = match res.status() {
        ExecutionStatus::Failure(e) => e.to_string(),
        _ => panic!("Should fail"),
    };
    assert!(err.contains("is not greater than total collateral"));

    // Force closing account with NEAR at 25$.
    let res = e.force_close(
        &users.bob,
        &users.alice,
        price_data(&tokens, Some(250000), None),
    );
    res.assert_success();
    // println!("{:#?}", res.logs());

    let account = e.get_account(&users.alice);
    assert!(account.supplied.is_empty());
    assert!(account.collateral.is_empty());
    assert!(account.borrowed.is_empty());

    let asset = e.get_asset(&tokens.nusdc);
    assert_eq!(asset.reserved, usdc_reserve + supply_amount);

    let asset = e.get_asset(&tokens.wnear);
    assert_eq!(asset.reserved, wnear_reserve - borrow_amount);
}
