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
    assert_balances(&account.supplied, &[av(tokens.ndai.account_id(), amount)]);

    assert_eq!(account.farms[0].farm_id, farm_id);
    assert_eq!(
        account.farms[0].rewards[0].reward_token_id,
        e.booster_token.account_id()
    );
    assert_eq!(
        account.farms[0].rewards[0].boosted_shares,
        find_asset(&account.supplied, &tokens.ndai.account_id())
            .shares
            .0,
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
    assert_balances(&account.supplied, &[av(tokens.ndai.account_id(), amount)]);

    assert_eq!(account.farms[0].farm_id, farm_id);
    assert_eq!(
        account.farms[0].rewards[0].boosted_shares,
        find_asset(&account.supplied, &tokens.ndai.account_id())
            .shares
            .0,
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
    assert_balances(
        &account.supplied,
        &[
            av(tokens.ndai.account_id(), amount),
            av(e.booster_token.account_id(), farmed_amount),
        ],
    );

    assert_eq!(account.farms[0].farm_id, farm_id);
    assert_eq!(
        account.farms[0].rewards[0].boosted_shares,
        find_asset(&account.supplied, &tokens.ndai.account_id())
            .shares
            .0,
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
    assert_balances(
        &account.supplied,
        &[
            av(tokens.ndai.account_id(), amount),
            av(e.booster_token.account_id(), farmed_amount),
        ],
    );

    assert_eq!(account.farms[0].farm_id, farm_id);
    assert_eq!(
        account.farms[0].rewards[0].boosted_shares,
        find_asset(&account.supplied, &tokens.ndai.account_id())
            .shares
            .0,
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
    assert_balances(
        &account.supplied,
        &[
            av(tokens.ndai.account_id(), amount),
            av(e.booster_token.account_id(), farmed_amount),
        ],
    );

    assert_eq!(account.farms[0].farm_id, farm_id);
    assert_eq!(
        account.farms[0].rewards[0].boosted_shares,
        find_asset(&account.supplied, &tokens.ndai.account_id())
            .shares
            .0,
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
    assert_balances(
        &account.supplied,
        &[
            av(tokens.ndai.account_id(), amount),
            av(e.booster_token.account_id(), total_reward),
        ],
    );

    assert_eq!(account.farms[0].farm_id, farm_id);
    assert!(account.farms[0].rewards.is_empty());
}

#[test]
fn test_has_potential_farms() {
    let (e, tokens, users) = basic_setup();

    let amount = d(100, 18);
    e.contract_ft_transfer_call(&tokens.ndai, &users.alice, amount, "")
        .assert_success();

    let account = e.get_account(&users.alice);
    assert!(!account.has_non_farmed_assets);

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

    let account = e.get_account(&users.alice);
    assert_eq!(account.farms.len(), 0);
    assert!(account.has_non_farmed_assets);

    e.account_farm_claim_all(&users.alice).assert_success();

    let account = e.get_account(&users.alice);
    assert_eq!(account.farms.len(), 1);
    assert!(!account.has_non_farmed_assets);
}

#[test]
fn test_farm_supplied_xbooster() {
    let (e, tokens, users) = basic_setup();

    let reward_per_day = d(100, 18);
    let total_reward = d(3000, 18);
    let booster_base = d(20, 18);

    let farm_id = FarmId::Supplied(tokens.ndai.account_id());
    e.add_farm(
        farm_id.clone(),
        &tokens.nusdc,
        reward_per_day,
        booster_base,
        total_reward,
    );

    let booster_amount = d(5, 18);
    e.contract_ft_transfer_call(&e.booster_token, &users.alice, booster_amount, "")
        .assert_success();

    e.account_stake_booster(&users.alice, booster_amount, MAX_DURATION_SEC)
        .assert_success();

    let amount = d(100, 18);
    e.contract_ft_transfer_call(&tokens.ndai, &users.alice, amount, "")
        .assert_success();

    let asset = e.get_asset(&tokens.nusdc);
    assert_eq!(asset.supplied.balance, 0);

    let asset = e.get_asset(&tokens.ndai);
    assert_eq!(asset.supplied.balance, amount);
    let booster_reward = asset.farms[0]
        .rewards
        .get(&tokens.nusdc.account_id())
        .cloned()
        .unwrap();
    assert_eq!(booster_reward.remaining_rewards, total_reward);
    assert_eq!(booster_reward.boosted_shares, asset.supplied.shares.0 * 2);

    let account = e.get_account(&users.alice);
    assert_balances(&account.supplied, &[av(tokens.ndai.account_id(), amount)]);

    let booster_staking = account.booster_staking.unwrap();
    assert_eq!(booster_staking.staked_booster_amount, booster_amount);
    assert_eq!(booster_staking.x_booster_amount, booster_amount * 4);

    // The amount of boosted shares should be 2X due to the log base.
    assert_eq!(
        account.farms[0].rewards[0].boosted_shares,
        find_asset(&account.supplied, &tokens.ndai.account_id())
            .shares
            .0
            * 2,
    );
    assert_eq!(account.farms[0].rewards[0].unclaimed_amount, 0);

    e.skip_time(ONE_DAY_SEC * 3);

    let farmed_amount = reward_per_day * 3;
    let asset = e.get_asset(&tokens.ndai);
    let booster_reward = asset.farms[0]
        .rewards
        .get(&tokens.nusdc.account_id())
        .cloned()
        .unwrap();
    assert_eq!(
        booster_reward.remaining_rewards,
        total_reward - farmed_amount
    );

    let account = e.get_account(&users.alice);
    assert_eq!(account.farms[0].rewards[0].unclaimed_amount, farmed_amount);

    let booster_amount = d(95, 18);
    e.contract_ft_transfer_call(&e.booster_token, &users.alice, booster_amount, "")
        .assert_success();

    // Increasing booster stake updates all farms.
    e.account_stake_booster(&users.alice, booster_amount, MAX_DURATION_SEC)
        .assert_success();

    let asset = e.get_asset(&tokens.nusdc);
    assert_eq!(asset.supplied.balance, farmed_amount);

    let asset = e.get_asset(&tokens.ndai);
    let booster_reward = asset.farms[0]
        .rewards
        .get(&tokens.nusdc.account_id())
        .cloned()
        .unwrap();
    assert_eq!(
        booster_reward.remaining_rewards,
        total_reward - farmed_amount
    );
    assert_eq!(booster_reward.boosted_shares, asset.supplied.shares.0 * 3);

    let account = e.get_account(&users.alice);
    assert_balances(
        &account.supplied,
        &[
            av(tokens.ndai.account_id(), amount),
            av(tokens.nusdc.account_id(), farmed_amount),
        ],
    );

    // The boosted amount should 3X because the xBooster is 400.
    assert_eq!(
        account.farms[0].rewards[0].boosted_shares,
        find_asset(&account.supplied, &tokens.ndai.account_id())
            .shares
            .0
            * 3,
    );
    assert_eq!(account.farms[0].rewards[0].unclaimed_amount, 0);
    let booster_staking = account.booster_staking.unwrap();
    assert_eq!(booster_staking.staked_booster_amount, d(100, 18));
    assert_eq!(booster_staking.x_booster_amount, d(400, 18));
}

#[test]
fn test_farm_supplied_xbooster_unstake() {
    let (e, tokens, users) = basic_setup();

    let booster_amount = d(5, 18);
    e.contract_ft_transfer_call(&e.booster_token, &users.alice, booster_amount, "")
        .assert_success();

    e.account_stake_booster(&users.alice, booster_amount, MAX_DURATION_SEC)
        .assert_success();

    e.skip_time(MAX_DURATION_SEC);

    let reward_per_day = d(100, 18);
    let total_reward = d(3000, 18);
    let booster_base = d(20, 18);

    let farm_id = FarmId::Supplied(tokens.ndai.account_id());
    e.add_farm(
        farm_id.clone(),
        &tokens.nusdc,
        reward_per_day,
        booster_base,
        total_reward,
    );

    let amount = d(100, 18);
    e.contract_ft_transfer_call(&tokens.ndai, &users.alice, amount, "")
        .assert_success();

    let asset = e.get_asset(&tokens.nusdc);
    assert_eq!(asset.supplied.balance, 0);

    let asset = e.get_asset(&tokens.ndai);
    assert_eq!(asset.supplied.balance, amount);
    let booster_reward = asset.farms[0]
        .rewards
        .get(&tokens.nusdc.account_id())
        .cloned()
        .unwrap();
    assert_eq!(booster_reward.remaining_rewards, total_reward);
    assert_eq!(booster_reward.boosted_shares, asset.supplied.shares.0 * 2);

    let account = e.get_account(&users.alice);

    // The amount of boosted shares should be 2X due to the log base.
    assert_eq!(
        account.farms[0].rewards[0].boosted_shares,
        find_asset(&account.supplied, &tokens.ndai.account_id())
            .shares
            .0
            * 2,
    );
    assert_eq!(account.farms[0].rewards[0].unclaimed_amount, 0);

    e.skip_time(ONE_DAY_SEC * 3);

    let farmed_amount = reward_per_day * 3;
    let asset = e.get_asset(&tokens.ndai);
    let booster_reward = asset.farms[0]
        .rewards
        .get(&tokens.nusdc.account_id())
        .cloned()
        .unwrap();
    assert_eq!(
        booster_reward.remaining_rewards,
        total_reward - farmed_amount
    );

    let account = e.get_account(&users.alice);
    assert_eq!(account.farms[0].rewards[0].unclaimed_amount, farmed_amount);

    // Unstaking booster updates all farms.
    e.account_unstake_booster(&users.alice).assert_success();

    let asset = e.get_asset(&tokens.nusdc);
    assert_eq!(asset.supplied.balance, farmed_amount);

    let asset = e.get_asset(&tokens.ndai);
    let booster_reward = asset.farms[0]
        .rewards
        .get(&tokens.nusdc.account_id())
        .cloned()
        .unwrap();
    assert_eq!(
        booster_reward.remaining_rewards,
        total_reward - farmed_amount
    );
    // The boosted amount should 1X because of xBooster unstaking.
    assert_eq!(booster_reward.boosted_shares, asset.supplied.shares.0);

    let account = e.get_account(&users.alice);
    assert_balances(
        &account.supplied,
        &[
            av(tokens.ndai.account_id(), amount),
            av(e.booster_token.account_id(), booster_amount),
            av(tokens.nusdc.account_id(), farmed_amount),
        ],
    );

    assert_eq!(
        account.farms[0].rewards[0].boosted_shares,
        find_asset(&account.supplied, &tokens.ndai.account_id())
            .shares
            .0,
    );
    assert_eq!(account.farms[0].rewards[0].unclaimed_amount, 0);
    assert!(account.booster_staking.is_none());
}
#[test]
fn test_farm_supplied_two_users() {
    let (e, tokens, users) = basic_setup();

    let booster_amount_alice = d(5, 18);
    e.contract_ft_transfer_call(&e.booster_token, &users.alice, booster_amount_alice, "")
        .assert_success();

    e.account_stake_booster(&users.alice, booster_amount_alice, MAX_DURATION_SEC)
        .assert_success();

    let booster_amount_bob = d(100, 18);
    e.contract_ft_transfer_call(&e.booster_token, &users.bob, booster_amount_bob, "")
        .assert_success();

    e.account_stake_booster(&users.bob, booster_amount_bob, MAX_DURATION_SEC)
        .assert_success();

    let reward_per_day = d(100, 18);
    let total_reward = d(3000, 18);
    let booster_base = d(20, 18);

    let farm_id = FarmId::Supplied(tokens.ndai.account_id());
    e.add_farm(
        farm_id.clone(),
        &tokens.nusdc,
        reward_per_day,
        booster_base,
        total_reward,
    );

    let amount = d(100, 18);
    e.contract_ft_transfer_call(&tokens.ndai, &users.alice, amount, "")
        .assert_success();

    e.contract_ft_transfer_call(&tokens.ndai, &users.bob, amount, "")
        .assert_success();

    let asset = e.get_asset(&tokens.nusdc);
    assert_eq!(asset.supplied.balance, 0);

    let asset = e.get_asset(&tokens.ndai);
    assert_eq!(asset.supplied.balance, amount * 2);
    let booster_reward = asset.farms[0]
        .rewards
        .get(&tokens.nusdc.account_id())
        .cloned()
        .unwrap();
    assert_eq!(booster_reward.remaining_rewards, total_reward);
    // 2.5X (Alice 2X, Bob 3X)
    assert_eq!(
        booster_reward.boosted_shares,
        asset.supplied.shares.0 * 5 / 2
    );

    let account = e.get_account(&users.alice);

    assert_eq!(
        account.farms[0].rewards[0].boosted_shares,
        find_asset(&account.supplied, &tokens.ndai.account_id())
            .shares
            .0
            * 2,
    );
    assert_eq!(account.farms[0].rewards[0].unclaimed_amount, 0);

    let account = e.get_account(&users.bob);

    assert_eq!(
        account.farms[0].rewards[0].boosted_shares,
        find_asset(&account.supplied, &tokens.ndai.account_id())
            .shares
            .0
            * 3,
    );
    assert_eq!(account.farms[0].rewards[0].unclaimed_amount, 0);

    e.skip_time(ONE_DAY_SEC * 3);

    let farmed_amount = reward_per_day * 3;
    let asset = e.get_asset(&tokens.ndai);
    let booster_reward = asset.farms[0]
        .rewards
        .get(&tokens.nusdc.account_id())
        .cloned()
        .unwrap();
    assert_eq!(
        booster_reward.remaining_rewards,
        total_reward - farmed_amount
    );

    let account = e.get_account(&users.alice);
    assert_eq!(
        account.farms[0].rewards[0].unclaimed_amount,
        farmed_amount * 2 / 5
    );

    let account = e.get_account(&users.bob);
    assert_eq!(
        account.farms[0].rewards[0].unclaimed_amount,
        farmed_amount * 3 / 5
    );

    let extra_booster_amount = d(95, 18);
    e.contract_ft_transfer_call(&e.booster_token, &users.alice, extra_booster_amount, "")
        .assert_success();

    // Increasing booster stake updates all farms.
    e.account_stake_booster(&users.alice, extra_booster_amount, MAX_DURATION_SEC)
        .assert_success();

    let asset = e.get_asset(&tokens.nusdc);
    // The amount of only for Alice, but Bob still unclaimed
    assert_eq!(asset.supplied.balance, farmed_amount * 2 / 5);

    let asset = e.get_asset(&tokens.ndai);
    let booster_reward = asset.farms[0]
        .rewards
        .get(&tokens.nusdc.account_id())
        .cloned()
        .unwrap();
    assert_eq!(
        booster_reward.remaining_rewards,
        total_reward - farmed_amount
    );

    // Both Alice and Bob now have 3X booster
    assert_eq!(booster_reward.boosted_shares, asset.supplied.shares.0 * 3);

    let account = e.get_account(&users.alice);
    assert_balances(
        &account.supplied,
        &[
            av(tokens.ndai.account_id(), amount),
            av(tokens.nusdc.account_id(), farmed_amount * 2 / 5),
        ],
    );

    assert_eq!(
        account.farms[0].rewards[0].boosted_shares,
        find_asset(&account.supplied, &tokens.ndai.account_id())
            .shares
            .0
            * 3,
    );
    assert_eq!(account.farms[0].rewards[0].unclaimed_amount, 0);

    let account = e.get_account(&users.bob);
    assert_eq!(
        account.farms[0].rewards[0].unclaimed_amount,
        farmed_amount * 3 / 5
    );

    e.skip_time(ONE_DAY_SEC * 2);

    let asset = e.get_asset(&tokens.nusdc);
    assert_eq!(asset.supplied.balance, farmed_amount * 2 / 5);

    let asset = e.get_asset(&tokens.ndai);
    let booster_reward = asset.farms[0]
        .rewards
        .get(&tokens.nusdc.account_id())
        .cloned()
        .unwrap();
    assert_eq!(
        booster_reward.remaining_rewards,
        total_reward - reward_per_day * 5
    );

    let account = e.get_account(&users.alice);
    // Unclaimed half of the rewards for 2 days
    assert_eq!(
        account.farms[0].rewards[0].unclaimed_amount,
        reward_per_day * 2 / 2
    );

    let account = e.get_account(&users.bob);
    assert_eq!(
        account.farms[0].rewards[0].unclaimed_amount,
        farmed_amount * 3 / 5 + reward_per_day * 2 / 2
    );
}
