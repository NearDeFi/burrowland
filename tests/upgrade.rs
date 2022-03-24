mod setup;

use crate::setup::*;

const PREVIOUS_VERSION: &'static str = "0.5.1";
const LATEST_VERSION: &'static str = "0.6.0";

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

    let asset = e.get_asset(&tokens.wnear);
    assert_eq!(asset.supplied.balance, amount);

    // The version is not available
    assert!(e
        .near
        .view_method_call(e.contract.contract.get_version())
        .is_err());

    e.deploy_contract_by_key(burrowland_0_4_0_wasm_bytes())
        .assert_success();

    let asset = e.get_asset(&tokens.wnear);
    assert_eq!(asset.supplied.balance, amount);

    let version: String = e
        .near
        .view_method_call(e.contract.contract.get_version())
        .unwrap_json();

    assert_eq!(version, "0.4.0");
}

#[test]
fn test_upgrade_by_owner() {
    let (e, tokens, users) = basic_setup_with_contract(burrowland_previous_wasm_bytes());

    let amount = d(100, 24);
    e.contract_ft_transfer_call(&tokens.wnear, &users.alice, amount, "")
        .assert_success();

    let asset = e.get_asset(&tokens.wnear);
    assert_eq!(asset.supplied.balance, amount);

    let version: String = e
        .near
        .view_method_call(e.contract.contract.get_version())
        .unwrap_json();

    assert_eq!(version, PREVIOUS_VERSION);

    e.deploy_contract_by_owner(burrowland_wasm_bytes())
        .assert_success();

    let asset = e.get_asset(&tokens.wnear);
    assert_eq!(asset.supplied.balance, amount);

    let version: String = e
        .near
        .view_method_call(e.contract.contract.get_version())
        .unwrap_json();

    assert_eq!(version, LATEST_VERSION);
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
