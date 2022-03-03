mod setup;

use crate::setup::*;
use contract::FarmId;

#[test]
fn test_farm_supplied() {
    let (e, tokens, users) = basic_setup();

    let reward_per_day = d(100, 18);
    let total_reward = d(3000, 18);

    let farm_id = FarmId::Supplied(tokens.ndai.account_id());
    e.add_farm(
        farm_id.clone(),
        &e.booster_token,
        reward_per_day,
        d(100, 18),
        total_reward,
    );

    let asset = e.get_asset(&tokens.ndai);
    assert_eq!(asset.farms.len(), 1);
    assert_eq!(asset.farms[0].farm_id, farm_id);
    let booster_reward = asset.farms[0]
        .rewards
        .get(&e.booster_token.account_id())
        .cloned()
        .unwrap();
    assert_eq!(booster_reward.remaining_rewards, total_reward);

    let amount = d(100, 18);
    e.contract_ft_transfer_call(&tokens.ndai, &users.alice, amount, "")
        .assert_success();

    let asset = e.get_asset(&e.booster_token);
    assert_eq!(asset.supplied.balance, 0);

    let asset = e.get_asset(&tokens.ndai);
    assert_eq!(asset.supplied.balance, amount);
    let booster_reward = asset.farms[0]
        .rewards
        .get(&e.booster_token.account_id())
        .cloned()
        .unwrap();
    assert_eq!(booster_reward.remaining_rewards, total_reward);

    let account = e.get_account(&users.alice);
    assert_eq!(account.supplied.len(), 1);
    assert_eq!(account.supplied[0].balance, amount);
    assert_eq!(account.supplied[0].token_id, tokens.ndai.account_id());

    assert_eq!(account.farms[0].farm_id, farm_id);
    assert_eq!(
        account.farms[0].rewards[0].reward_token_id,
        e.booster_token.account_id()
    );
    assert_eq!(
        account.farms[0].rewards[0].boosted_shares,
        account.supplied[0].shares.0,
    );
    assert_eq!(account.farms[0].rewards[0].unclaimed_amount, 0);

    e.skip_time(ONE_DAY_SEC * 3);

    let farmed_amount = reward_per_day * 3;

    let asset = e.get_asset(&e.booster_token);
    assert_eq!(asset.supplied.balance, 0);

    let asset = e.get_asset(&tokens.ndai);
    assert_eq!(asset.supplied.balance, amount);
    let booster_reward = asset.farms[0]
        .rewards
        .get(&e.booster_token.account_id())
        .cloned()
        .unwrap();
    assert_eq!(
        booster_reward.remaining_rewards,
        total_reward - farmed_amount
    );

    let account = e.get_account(&users.alice);
    assert_eq!(account.supplied.len(), 1);
    assert_eq!(account.supplied[0].balance, amount);
    assert_eq!(account.supplied[0].token_id, tokens.ndai.account_id());

    assert_eq!(account.farms[0].farm_id, farm_id);
    assert_eq!(
        account.farms[0].rewards[0].boosted_shares,
        account.supplied[0].shares.0,
    );
    assert_eq!(account.farms[0].rewards[0].unclaimed_amount, farmed_amount);

    e.account_farm_claim_all(&users.alice).assert_success();

    let asset = e.get_asset(&e.booster_token);
    assert_eq!(asset.supplied.balance, farmed_amount);

    let asset = e.get_asset(&tokens.ndai);
    assert_eq!(asset.supplied.balance, amount);
    let booster_reward = asset.farms[0]
        .rewards
        .get(&e.booster_token.account_id())
        .cloned()
        .unwrap();
    assert_eq!(
        booster_reward.remaining_rewards,
        total_reward - farmed_amount
    );

    let account = e.get_account(&users.alice);
    assert_eq!(account.supplied.len(), 2);
    assert_eq!(account.supplied[0].balance, amount);
    assert_eq!(account.supplied[0].token_id, tokens.ndai.account_id());
    assert_eq!(account.supplied[1].balance, farmed_amount);
    assert_eq!(account.supplied[1].token_id, e.booster_token.account_id());

    assert_eq!(account.farms[0].farm_id, farm_id);
    assert_eq!(
        account.farms[0].rewards[0].boosted_shares,
        account.supplied[0].shares.0,
    );
    assert_eq!(account.farms[0].rewards[0].unclaimed_amount, 0);

    e.skip_time(ONE_DAY_SEC * 2);

    let asset = e.get_asset(&e.booster_token);
    assert_eq!(asset.supplied.balance, farmed_amount);

    let asset = e.get_asset(&tokens.ndai);
    assert_eq!(asset.supplied.balance, amount);
    let booster_reward = asset.farms[0]
        .rewards
        .get(&e.booster_token.account_id())
        .cloned()
        .unwrap();
    assert_eq!(
        booster_reward.remaining_rewards,
        total_reward - reward_per_day * 5
    );

    let account = e.get_account(&users.alice);
    assert_eq!(account.supplied.len(), 2);
    assert_eq!(account.supplied[0].balance, amount);
    assert_eq!(account.supplied[0].token_id, tokens.ndai.account_id());
    assert_eq!(account.supplied[1].balance, farmed_amount);
    assert_eq!(account.supplied[1].token_id, e.booster_token.account_id());

    assert_eq!(account.farms[0].farm_id, farm_id);
    assert_eq!(
        account.farms[0].rewards[0].boosted_shares,
        account.supplied[0].shares.0,
    );
    assert_eq!(
        account.farms[0].rewards[0].unclaimed_amount,
        reward_per_day * 2
    );

    e.skip_time(ONE_DAY_SEC * 30);

    let asset = e.get_asset(&tokens.ndai);
    assert_eq!(asset.supplied.balance, amount);
    let booster_reward = asset.farms[0]
        .rewards
        .get(&e.booster_token.account_id())
        .cloned()
        .unwrap();
    assert_eq!(booster_reward.remaining_rewards, 0);

    let account = e.get_account(&users.alice);
    assert_eq!(account.supplied.len(), 2);
    assert_eq!(account.supplied[0].balance, amount);
    assert_eq!(account.supplied[0].token_id, tokens.ndai.account_id());
    assert_eq!(account.supplied[1].balance, farmed_amount);
    assert_eq!(account.supplied[1].token_id, e.booster_token.account_id());

    assert_eq!(account.farms[0].farm_id, farm_id);
    assert_eq!(
        account.farms[0].rewards[0].boosted_shares,
        account.supplied[0].shares.0,
    );
    assert_eq!(
        account.farms[0].rewards[0].unclaimed_amount,
        total_reward - farmed_amount
    );

    e.account_farm_claim_all(&users.alice).assert_success();

    let asset = e.get_asset(&e.booster_token);
    assert_eq!(asset.supplied.balance, total_reward);

    let asset = e.get_asset(&tokens.ndai);
    assert_eq!(asset.supplied.balance, amount);
    assert!(asset.farms[0]
        .rewards
        .get(&e.booster_token.account_id())
        .is_none());

    let account = e.get_account(&users.alice);
    assert_eq!(account.supplied.len(), 2);
    assert_eq!(account.supplied[0].balance, amount);
    assert_eq!(account.supplied[0].token_id, tokens.ndai.account_id());
    assert_eq!(account.supplied[1].balance, total_reward);
    assert_eq!(account.supplied[1].token_id, e.booster_token.account_id());

    assert_eq!(account.farms[0].farm_id, farm_id);
    assert!(account.farms[0].rewards.is_empty());
}
