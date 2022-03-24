mod setup;

use crate::setup::*;
use near_sdk::serde_json;

#[test]
fn test_deposit_event() {
    let (e, tokens, users) = basic_setup();

    let amount = d(100, 24);

    let res = e.contract_ft_transfer_call(&tokens.wnear, &users.alice, amount, "");
    res.assert_success();
    let logs = get_logs(&e.near.borrow_runtime());
    let event = &logs[1];
    assert!(event.starts_with(EVENT_JSON));

    let value: serde_json::Value =
        serde_json::from_str(&event[EVENT_JSON.len()..]).expect("Failed to parse the event");
    assert_eq!(value["standard"].as_str().unwrap(), "burrow");
    assert_eq!(value["event"].as_str().unwrap(), "deposit");
    assert_eq!(
        value["data"][0]["account_id"].as_str().unwrap(),
        users.alice.account_id().as_str()
    );
    assert_eq!(
        value["data"][0]["amount"].as_str().unwrap(),
        amount.to_string()
    );
    assert_eq!(
        value["data"][0]["token_id"].as_str().unwrap(),
        tokens.wnear.account_id().as_str()
    );
}
