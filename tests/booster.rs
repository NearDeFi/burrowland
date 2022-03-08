mod setup;

use crate::setup::*;
use common::DurationSec;
use near_sdk::serde_json::json;

#[test]
fn test_booster_stake_unstake() {
    let (e, _tokens, users) = basic_setup();

    let amount = d(100, 18);
    e.contract_ft_transfer_call(&e.booster_token, &users.alice, amount, "")
        .assert_success();

    let asset = e.get_asset(&e.booster_token);
    assert_eq!(asset.supplied.balance, amount);

    let account = e.get_account(&users.alice);
    assert_eq!(account.supplied[0].balance, amount);
    assert_eq!(account.supplied[0].token_id, e.booster_token.account_id());
    assert!(account.booster_staking.is_none());

    let duration_sec: DurationSec = MAX_DURATION_SEC;

    e.account_stake_booster(&users.alice, amount, duration_sec)
        .assert_success();

    let asset = e.get_asset(&e.booster_token);
    assert_eq!(asset.supplied.balance, 0);

    let account = e.get_account(&users.alice);
    assert!(account.supplied.is_empty());
    let booster_staking = account.booster_staking.unwrap();
    assert_eq!(booster_staking.staked_booster_amount, amount);
    assert_eq!(booster_staking.x_booster_amount, amount * 4);
    assert_eq!(
        booster_staking.unlock_timestamp,
        GENESIS_TIMESTAMP + sec_to_nano(duration_sec)
    );

    e.skip_time(duration_sec / 2);

    let account = e.get_account(&users.alice);
    assert!(account.supplied.is_empty());
    let booster_staking = account.booster_staking.unwrap();
    assert_eq!(booster_staking.x_booster_amount, amount * 4);

    assert!(!e.account_unstake_booster(&users.alice).is_ok());

    e.skip_time(duration_sec / 2);

    let account = e.get_account(&users.alice);
    assert!(account.supplied.is_empty());
    let booster_staking = account.booster_staking.unwrap();
    assert_eq!(booster_staking.x_booster_amount, amount * 4);

    e.account_unstake_booster(&users.alice).assert_success();
    assert!(!e.account_unstake_booster(&users.alice).is_ok());

    let asset = e.get_asset(&e.booster_token);
    assert_eq!(asset.supplied.balance, amount);

    let account = e.get_account(&users.alice);
    assert_eq!(account.supplied[0].balance, amount);
    assert_eq!(account.supplied[0].token_id, e.booster_token.account_id());
    assert!(account.booster_staking.is_none());
    assert!(!e.account_unstake_booster(&users.alice).is_ok());
}

#[test]
fn test_booster_add_stake() {
    let (e, _tokens, users) = basic_setup();

    let amount = d(100, 18);
    e.contract_ft_transfer_call(&e.booster_token, &users.alice, amount, "")
        .assert_success();

    let duration_sec: DurationSec = MAX_DURATION_SEC;

    e.account_stake_booster(&users.alice, amount / 2, duration_sec)
        .assert_success();

    assert!(!e
        .account_stake_booster(&users.alice, amount / 2, duration_sec - 1)
        .is_ok());

    let asset = e.get_asset(&e.booster_token);
    assert_eq!(asset.supplied.balance, amount / 2);

    let account = e.get_account(&users.alice);
    assert_eq!(account.supplied[0].balance, amount / 2);
    assert_eq!(account.supplied[0].token_id, e.booster_token.account_id());
    let booster_staking = account.booster_staking.unwrap();
    assert_eq!(booster_staking.staked_booster_amount, amount / 2);
    assert_eq!(booster_staking.x_booster_amount, amount / 2 * 4);
    assert_eq!(
        booster_staking.unlock_timestamp,
        GENESIS_TIMESTAMP + sec_to_nano(duration_sec)
    );

    e.skip_time(duration_sec / 2);

    e.account_stake_booster(&users.alice, amount / 2, duration_sec / 2)
        .assert_success();

    let asset = e.get_asset(&e.booster_token);
    assert_eq!(asset.supplied.balance, 0);

    let account = e.get_account(&users.alice);
    assert!(account.supplied.is_empty());
    let booster_staking = account.booster_staking.unwrap();
    assert_eq!(booster_staking.staked_booster_amount, amount);
    assert_eq!(
        booster_staking.x_booster_amount,
        amount / 2 * 4
            + amount / 2
                * u128::from(
                    MAX_DURATION_SEC - MIN_DURATION_SEC + (duration_sec / 2 - MIN_DURATION_SEC) * 3
                )
                / u128::from(MAX_DURATION_SEC - MIN_DURATION_SEC)
    );
    assert_eq!(
        booster_staking.unlock_timestamp,
        GENESIS_TIMESTAMP + sec_to_nano(duration_sec)
    );

    e.skip_time(duration_sec / 2);
    e.account_unstake_booster(&users.alice).assert_success();

    let asset = e.get_asset(&e.booster_token);
    assert_eq!(asset.supplied.balance, amount);

    let account = e.get_account(&users.alice);
    assert_eq!(account.supplied[0].balance, amount);
    assert_eq!(account.supplied[0].token_id, e.booster_token.account_id());
    assert!(account.booster_staking.is_none());
}

#[test]
fn test_booster_add_stake_extend_duration() {
    let (e, _tokens, users) = basic_setup();

    let amount = d(100, 18);
    e.contract_ft_transfer_call(&e.booster_token, &users.alice, amount, "")
        .assert_success();

    let duration_sec: DurationSec = MAX_DURATION_SEC;

    e.account_stake_booster(&users.alice, amount / 2, duration_sec)
        .assert_success();

    e.skip_time(duration_sec / 2);

    e.account_stake_booster(&users.alice, amount / 2, duration_sec)
        .assert_success();

    let asset = e.get_asset(&e.booster_token);
    assert_eq!(asset.supplied.balance, 0);

    let account = e.get_account(&users.alice);
    assert!(account.supplied.is_empty());
    let booster_staking = account.booster_staking.unwrap();
    assert_eq!(booster_staking.staked_booster_amount, amount);
    assert_eq!(booster_staking.x_booster_amount, amount * 4);
    assert_eq!(
        booster_staking.unlock_timestamp,
        GENESIS_TIMESTAMP + sec_to_nano(duration_sec * 3 / 2)
    );

    e.skip_time(duration_sec / 2);
    assert!(!e.account_unstake_booster(&users.alice).is_ok());

    e.skip_time(duration_sec / 2);
    e.account_unstake_booster(&users.alice).assert_success();

    let asset = e.get_asset(&e.booster_token);
    assert_eq!(asset.supplied.balance, amount);

    let account = e.get_account(&users.alice);
    assert_eq!(account.supplied[0].balance, amount);
    assert_eq!(account.supplied[0].token_id, e.booster_token.account_id());
    assert!(account.booster_staking.is_none());
}

#[test]
fn test_booster_stake_bad_args() {
    let (e, _tokens, users) = basic_setup();

    let amount = d(100, 18);
    e.contract_ft_transfer_call(&e.booster_token, &users.alice, amount, "")
        .assert_success();

    // Amount can't be 0
    assert!(!e
        .account_stake_booster(&users.alice, 0, MAX_DURATION_SEC)
        .is_ok());

    // Not enough balance.
    assert!(!e
        .account_stake_booster(&users.alice, amount + 1, MAX_DURATION_SEC)
        .is_ok());

    // Less than min duration.
    assert!(!e
        .account_stake_booster(&users.alice, amount, MIN_DURATION_SEC - 1)
        .is_ok());

    // More than max duration.
    assert!(!e
        .account_stake_booster(&users.alice, amount, MAX_DURATION_SEC + 1)
        .is_ok());
}

#[test]
fn test_booster_stake_all() {
    let (e, _tokens, users) = basic_setup();

    let amount = d(100, 18);
    e.contract_ft_transfer_call(&e.booster_token, &users.alice, amount, "")
        .assert_success();

    users
        .alice
        .call(
            e.contract.account_id(),
            "account_stake_booster",
            &json!({
                "duration": MAX_DURATION_SEC,
            })
            .to_string()
            .into_bytes(),
            MAX_GAS.0,
            1,
        )
        .assert_success();

    let asset = e.get_asset(&e.booster_token);
    assert_eq!(asset.supplied.balance, 0);

    let account = e.get_account(&users.alice);
    assert!(account.supplied.is_empty());
    let booster_staking = account.booster_staking.unwrap();
    assert_eq!(booster_staking.staked_booster_amount, amount);
    assert_eq!(booster_staking.x_booster_amount, amount * 4);
}
