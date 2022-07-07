mod setup;

use crate::setup::*;
use near_sdk::serde_json;

const PREVIOUS_VERSION: &'static str = "0.6.0";
const LATEST_VERSION: &'static str = "0.7.0";

#[test]
fn test_version() {
    let (e, _tokens, _users) = basic_setup();

    let version: String = e
        .near
        .view_method_call(e.contract.contract.get_version())
        .unwrap_json();

    assert_eq!(version, LATEST_VERSION);
}

#[test]
fn test_upgrade_with_private_key() {
    let (e, tokens, users) = basic_setup_with_contract(burrowland_0_3_0_wasm_bytes());

    let amount = d(100, 24);
    e.contract_ft_transfer_call(&tokens.wnear, &users.alice, amount, "")
        .assert_success();

    let asset: serde_json::value::Value = e
        .near
        .view_method_call(e.contract.contract.get_asset(tokens.wnear.account_id()))
        .unwrap_json();
    assert_eq!(
        asset
            .get("supplied")
            .unwrap()
            .get("balance")
            .unwrap()
            .as_str()
            .unwrap(),
        &amount.to_string()
    );

    // The version is not available
    assert!(e
        .near
        .view_method_call(e.contract.contract.get_version())
        .is_err());

    e.deploy_contract_by_key(burrowland_0_4_0_wasm_bytes())
        .assert_success();

    let asset: serde_json::value::Value = e
        .near
        .view_method_call(e.contract.contract.get_asset(tokens.wnear.account_id()))
        .unwrap_json();
    assert_eq!(
        asset
            .get("supplied")
            .unwrap()
            .get("balance")
            .unwrap()
            .as_str()
            .unwrap(),
        &amount.to_string()
    );

    let version: String = e
        .near
        .view_method_call(e.contract.contract.get_version())
        .unwrap_json();

    assert_eq!(version, "0.4.0");
}

/// Note, the following test has logic specific to verify upgrade to 0.7.0 that modifies internal
/// account storage, so the available storage should increase.
#[test]
fn test_upgrade_by_owner() {
    let (e, tokens, users) = basic_setup_with_contract(burrowland_previous_wasm_bytes());

    let amount = d(100, 24);
    e.contract_ft_transfer_call(&tokens.wnear, &users.alice, amount, "")
        .assert_success();

    let asset: serde_json::value::Value = e
        .near
        .view_method_call(e.contract.contract.get_asset(tokens.wnear.account_id()))
        .unwrap_json();
    assert_eq!(
        asset
            .get("supplied")
            .unwrap()
            .get("balance")
            .unwrap()
            .as_str()
            .unwrap(),
        &amount.to_string()
    );

    let account = e.get_account(&users.alice);
    assert_eq!(
        find_asset(&account.supplied, &tokens.wnear.account_id()).balance,
        amount
    );

    let version: String = e
        .near
        .view_method_call(e.contract.contract.get_version())
        .unwrap_json();

    assert_eq!(version, PREVIOUS_VERSION);

    e.deploy_contract_by_owner(burrowland_wasm_bytes())
        .assert_success();

    let version: String = e
        .near
        .view_method_call(e.contract.contract.get_version())
        .unwrap_json();

    assert_eq!(version, LATEST_VERSION);

    let asset = e.get_asset(&tokens.wnear);
    assert_eq!(asset.supplied.balance, amount);
    assert_eq!(asset.config.net_tvl_multiplier, 10000);

    let account = e.get_account(&users.alice);
    assert_eq!(
        find_asset(&account.supplied, &tokens.wnear.account_id()).balance,
        amount
    );

    let before_action_storage_balance = e.debug_storage_balance_of(&users.alice).unwrap();

    e.contract_ft_transfer_call(&tokens.wnear, &users.alice, amount, "")
        .assert_success();

    let account = e.get_account(&users.alice);
    assert_eq!(
        find_asset(&account.supplied, &tokens.wnear.account_id()).balance,
        amount * 2
    );

    let after_action_storage_balance = e.debug_storage_balance_of(&users.alice).unwrap();
    assert!(before_action_storage_balance.available.0 < after_action_storage_balance.available.0);

    e.contract_ft_transfer_call(&tokens.wnear, &users.alice, amount, "")
        .assert_success();

    let account = e.get_account(&users.alice);
    assert_eq!(
        find_asset(&account.supplied, &tokens.wnear.account_id()).balance,
        amount * 3
    );

    let after_two_actions_storage_balance = e.debug_storage_balance_of(&users.alice).unwrap();
    assert_eq!(
        after_action_storage_balance.available.0,
        after_two_actions_storage_balance.available.0
    );
}

#[test]
fn test_degrade_fails() {
    let (e, _tokens, _users) = basic_setup_with_contract(burrowland_0_4_0_wasm_bytes());

    assert!(!e
        .deploy_contract_by_owner(burrowland_0_3_0_wasm_bytes())
        .is_ok());

    let version: String = e
        .near
        .view_method_call(e.contract.contract.get_version())
        .unwrap_json();

    assert_eq!(version, "0.4.0");
}
