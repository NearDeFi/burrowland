mod setup;

use crate::setup::*;
use common::DurationSec;
use near_sdk::json_types::U128;
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

    let duration_sec: DurationSec = 31536000;

    users
        .alice
        .call(
            e.contract.account_id(),
            "account_stake_booster",
            &json!({
                "amount": U128::from(amount),
                "duration": duration_sec,
            })
            .to_string()
            .into_bytes(),
            MAX_GAS.0,
            1,
        )
        .assert_success();

    let account = e.get_account(&users.alice);
    assert!(account.supplied.is_empty());
    let booster_staking = account.booster_staking.unwrap();
    assert_eq!(booster_staking.staked_booster_amount, amount);
    assert_eq!(booster_staking.x_booster_amount, amount * 4);
    assert_eq!(
        booster_staking.unlock_timestamp,
        GENESIS_TIMESTAMP + sec_to_nano(duration_sec)
    );
}
