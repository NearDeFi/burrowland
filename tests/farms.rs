mod setup;

use crate::setup::*;
use common::ONE_YOCTO;
use contract::FarmId;
use near_sdk::json_types::U128;

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

#[test]
fn test_farm_net_tvl() {
    let (e, tokens, users) = basic_setup();

    let reward_per_day = d(100, 18);
    let total_reward = d(3000, 18);

    let farm_id = FarmId::NetTvl;
    e.add_farm(
        farm_id.clone(),
        &e.booster_token,
        reward_per_day,
        d(100, 18),
        total_reward,
    );

    let asset_farm = e.get_asset_farm(farm_id.clone());
    let booster_reward = asset_farm
        .rewards
        .get(&e.booster_token.account_id())
        .cloned()
        .unwrap();
    assert_eq!(booster_reward.remaining_rewards, total_reward);

    let amount = d(100, 18);
    e.supply_to_collateral(&users.alice, &tokens.ndai, amount)
        .assert_success();
    // Borrow 1 NEAR
    let borrow_amount = d(1, 24);
    e.borrow_and_withdraw(
        &users.alice,
        &tokens.wnear,
        price_data(&tokens, Some(100000), None),
        borrow_amount,
    )
    .assert_success();

    let asset = e.get_asset(&e.booster_token);
    assert_eq!(asset.supplied.balance, 0);

    let account = e.get_account(&users.alice);
    assert_eq!(account.farms.len(), 1);
    assert_eq!(account.farms[0].farm_id, farm_id);
    assert_eq!(
        account.farms[0].rewards[0].reward_token_id,
        e.booster_token.account_id()
    );
    // The account should have 90$ of Net TVL. $100 from dai and 10$ wNEAR borrowed.
    assert_eq!(account.farms[0].rewards[0].boosted_shares, d(90, 18));
    assert_eq!(account.farms[0].rewards[0].unclaimed_amount, 0);

    e.skip_time(ONE_DAY_SEC * 3);

    let farmed_amount = reward_per_day * 3;

    let asset = e.get_asset(&e.booster_token);
    assert_eq!(asset.supplied.balance, 0);

    let asset_farm = e.get_asset_farm(farm_id.clone());
    let booster_reward = asset_farm
        .rewards
        .get(&e.booster_token.account_id())
        .cloned()
        .unwrap();
    assert_eq!(
        booster_reward.remaining_rewards,
        total_reward - farmed_amount
    );

    let account = e.get_account(&users.alice);
    assert_eq!(account.farms[0].farm_id, farm_id);
    assert_eq!(account.farms[0].rewards[0].unclaimed_amount, farmed_amount);

    e.account_farm_claim_all(&users.alice).assert_success();

    let asset = e.get_asset(&e.booster_token);
    assert_eq!(asset.supplied.balance, farmed_amount);

    let asset_farm = e.get_asset_farm(farm_id.clone());
    let booster_reward = asset_farm
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
        &[av(e.booster_token.account_id(), farmed_amount)],
    );

    assert_eq!(account.farms[0].farm_id, farm_id);
    // Due to borrowing interest
    assert!(
        account.farms[0].rewards[0].boosted_shares >= d(89, 18)
            && account.farms[0].rewards[0].boosted_shares < d(90, 18)
    );
    assert_eq!(account.farms[0].rewards[0].unclaimed_amount, 0);
}

#[test]
fn test_farm_net_tvl_complex() {
    let (e, tokens, users) = basic_setup();

    let reward_per_day = d(100, 18);
    let total_reward = d(3000, 18);

    let farm_id = FarmId::NetTvl;
    e.add_farm(
        farm_id.clone(),
        &tokens.ndai,
        reward_per_day,
        d(100, 18),
        total_reward,
    );

    let asset_farm = e.get_asset_farm(farm_id.clone());
    let reward = asset_farm
        .rewards
        .get(&tokens.ndai.account_id())
        .cloned()
        .unwrap();
    assert_eq!(reward.remaining_rewards, total_reward);

    let amount = d(100, 18);
    e.supply_to_collateral(&users.alice, &tokens.ndai, amount)
        .assert_success();

    let bob_amount = d(30, 6);
    e.supply_to_collateral(&users.bob, &tokens.nusdc, bob_amount)
        .assert_success();

    let charlie_amount = d(40, 6);
    e.supply_to_collateral(&users.charlie, &tokens.nusdt, charlie_amount)
        .assert_success();

    let bob_borrow_amount = d(1, 24);
    e.borrow_and_withdraw(
        &users.alice,
        &tokens.wnear,
        price_data(&tokens, Some(100000), None),
        bob_borrow_amount,
    )
    .assert_success();

    let charlie_borrow_amount = d(10, 18);
    e.borrow_and_withdraw(
        &users.charlie,
        &tokens.nusdt,
        price_data(&tokens, Some(100000), None),
        charlie_borrow_amount,
    )
        .assert_success();

    let account = e.get_account(&users.alice);
    assert_eq!(account.farms.len(), 1);
    assert_eq!(account.farms[0].farm_id, farm_id);
    assert_eq!(
        account.farms[0].rewards[0].reward_token_id,
        tokens.ndai.account_id()
    );
    // The account should have 90$ of Net TVL. $100 from dai and 10$ wNEAR borrowed.
    assert_eq!(account.farms[0].rewards[0].boosted_shares, d(90, 18));
    assert_eq!(account.farms[0].rewards[0].unclaimed_amount, 0);

    // Bob doesn't have a farm, since there were no prices when bob made a deposit.
    let account = e.get_account(&users.bob);
    assert!(account.farms.is_empty());

    let asset_farm = e.get_asset_farm(farm_id.clone());
    let reward = asset_farm
        .rewards
        .get(&tokens.ndai.account_id())
        .cloned()
        .unwrap();
    assert_eq!(reward.boosted_shares, d(120, 18));

    e.account_farm_claim_all_on_behalf(&users.alice, &users.bob)
        .assert_success();

    let account = e.get_account(&users.bob);
    assert_eq!(account.farms.len(), 1);
    assert_eq!(account.farms[0].farm_id, farm_id);
    assert_eq!(
        account.farms[0].rewards[0].reward_token_id,
        tokens.ndai.account_id()
    );
    // The account should have 30$ of Net TVL. $30 from usdc.
    assert_eq!(account.farms[0].rewards[0].boosted_shares, d(30, 18));
    assert_eq!(account.farms[0].rewards[0].unclaimed_amount, 0);

    e.account_farm_claim_all_on_behalf(&users.alice, &users.charlie)
        .assert_success();

    let account = e.get_account(&users.charlie);
    assert_eq!(account.farms.len(), 1);
    assert_eq!(account.farms[0].farm_id, farm_id);
    assert_eq!(
        account.farms[0].rewards[0].reward_token_id,
        tokens.ndai.account_id()
    );
    // The account should have 30$ of Net TVL. $40 from usdt deposit - $10 from usdt borrow.
    assert_eq!(account.farms[0].rewards[0].boosted_shares, d(30, 18));
    assert_eq!(account.farms[0].rewards[0].unclaimed_amount, 0);

    let asset_farm = e.get_asset_farm(farm_id.clone());
    let reward = asset_farm
        .rewards
        .get(&tokens.ndai.account_id())
        .cloned()
        .unwrap();
    assert_eq!(reward.boosted_shares, d(150, 18));

    e.skip_time(ONE_DAY_SEC * 3);

    let farmed_amount = reward_per_day * 3;

    let asset_farm = e.get_asset_farm(farm_id.clone());
    let reward = asset_farm
        .rewards
        .get(&tokens.ndai.account_id())
        .cloned()
        .unwrap();
    assert_eq!(reward.remaining_rewards, total_reward - farmed_amount);

    let account = e.get_account(&users.alice);
    assert_eq!(account.farms[0].farm_id, farm_id);
    almost_eq(
        account.farms[0].rewards[0].unclaimed_amount,
        farmed_amount * 90 / 150,
        18
    );

    let bobs_farmed_amount = farmed_amount * 30 / 150;
    let account = e.get_account(&users.bob);
    assert_eq!(account.farms[0].farm_id, farm_id);
    assert_eq!(
        account.farms[0].rewards[0].unclaimed_amount,
        bobs_farmed_amount
    );

    e.account_farm_claim_all(&users.bob).assert_success();

    let account = e.get_account(&users.bob);
    assert_balances(
        &account.supplied,
        &[av(tokens.ndai.account_id(), bobs_farmed_amount)],
    );
    // 30$ usdc + 60$ ndai from farming rewards.
    almost_eq(account.farms[0].rewards[0].boosted_shares, d(30 + 60 , 18), 13);
    assert_eq!(account.farms[0].rewards[0].unclaimed_amount, 0);

    let asset_farm = e.get_asset_farm(farm_id.clone());
    let reward = asset_farm
        .rewards
        .get(&tokens.ndai.account_id())
        .cloned()
        .unwrap();
    almost_eq(reward.boosted_shares, d(120 + 90, 18), 13);

    let charlie_farmed_amount = farmed_amount * 30 / 150;
    let account = e.get_account(&users.charlie);
    assert_eq!(account.farms[0].farm_id, farm_id);
    assert_eq!(
        account.farms[0].rewards[0].unclaimed_amount,
        charlie_farmed_amount
    );

    e.account_farm_claim_all(&users.charlie).assert_success();

    let account = e.get_account(&users.charlie);
    assert_balances(
        &account.supplied,
        &[av(tokens.ndai.account_id(), charlie_farmed_amount)],
    );
    // 30$ usdt + 60$ ndai from farming rewards.
    almost_eq(account.farms[0].rewards[0].boosted_shares, d(30 + 60 , 18), 13);
    assert_eq!(account.farms[0].rewards[0].unclaimed_amount, 0);

    let asset_farm = e.get_asset_farm(farm_id.clone());
    let reward = asset_farm
        .rewards
        .get(&tokens.ndai.account_id())
        .cloned()
        .unwrap();
    almost_eq(reward.boosted_shares, d(120 + 150, 18), 13);
}

#[test]
fn test_farm_net_tvl_price_change() {
    let (e, tokens, users) = basic_setup();

    let reward_per_day = d(100, 18);
    let total_reward = d(3000, 18);

    let farm_id = FarmId::NetTvl;
    e.add_farm(
        farm_id.clone(),
        &tokens.ndai,
        reward_per_day,
        d(100, 18),
        total_reward,
    );

    let asset_farm = e.get_asset_farm(farm_id.clone());
    let reward = asset_farm
        .rewards
        .get(&tokens.ndai.account_id())
        .cloned()
        .unwrap();
    assert_eq!(reward.remaining_rewards, total_reward);

    let amount = d(100, 18);
    e.supply_to_collateral(&users.alice, &tokens.ndai, amount)
        .assert_success();

    let borrow_amount = d(2, 24);
    e.borrow_and_withdraw(
        &users.alice,
        &tokens.wnear,
        price_data(&tokens, Some(100000), None),
        borrow_amount,
    )
    .assert_success();

    let account = e.get_account(&users.alice);
    assert_eq!(account.farms.len(), 1);
    assert_eq!(account.farms[0].farm_id, farm_id);
    assert_eq!(
        account.farms[0].rewards[0].reward_token_id,
        tokens.ndai.account_id()
    );
    // The account should have 80$ of Net TVL. $100 from dai and 20$ wNEAR borrowed.
    assert_eq!(account.farms[0].rewards[0].boosted_shares, d(80, 18));
    assert_eq!(account.farms[0].rewards[0].unclaimed_amount, 0);

    let amount = d(60, 18);
    e.supply_to_collateral(&users.bob, &tokens.ndai, amount)
        .assert_success();

    let borrow_amount = d(1, 24);
    e.borrow_and_withdraw(
        &users.bob,
        &tokens.wnear,
        price_data(&tokens, Some(150000), None),
        borrow_amount,
    )
    .assert_success();

    let account = e.get_account(&users.bob);
    assert_eq!(account.farms.len(), 1);
    // The account should have 45$ of Net TVL. $60 from dai and 15$ wNEAR borrowed.
    assert_eq!(account.farms[0].rewards[0].boosted_shares, d(45, 18));

    let account = e.get_account(&users.alice);
    // The shares do not change until the account is affected.
    assert_eq!(account.farms[0].rewards[0].boosted_shares, d(80, 18));

    let asset_farm = e.get_asset_farm(farm_id.clone());
    let reward = asset_farm
        .rewards
        .get(&tokens.ndai.account_id())
        .cloned()
        .unwrap();
    assert_eq!(reward.boosted_shares, d(125, 18));

    e.account_farm_claim_all_on_behalf(&users.bob, &users.alice)
        .assert_success();

    let account = e.get_account(&users.alice);
    // The account should have 80$ of Net TVL. $100 from dai and 30$ (2 * 15$) wNEAR borrowed.
    assert_eq!(account.farms[0].rewards[0].boosted_shares, d(70, 18));

    let asset_farm = e.get_asset_farm(farm_id.clone());
    let reward = asset_farm
        .rewards
        .get(&tokens.ndai.account_id())
        .cloned()
        .unwrap();
    assert_eq!(reward.boosted_shares, d(115, 18));
}

#[test]
fn test_farm_net_tvl_bad_debt() {
    let (e, tokens, users) = basic_setup();

    let reward_per_day = d(100, 18);
    let total_reward = d(3000, 18);

    let farm_id = FarmId::NetTvl;
    e.add_farm(
        farm_id.clone(),
        &tokens.ndai,
        reward_per_day,
        d(100, 18),
        total_reward,
    );

    let asset_farm = e.get_asset_farm(farm_id.clone());
    let reward = asset_farm
        .rewards
        .get(&tokens.ndai.account_id())
        .cloned()
        .unwrap();
    assert_eq!(reward.remaining_rewards, total_reward);

    let amount = d(100, 18);
    e.supply_to_collateral(&users.alice, &tokens.ndai, amount)
        .assert_success();

    let borrow_amount = d(4, 24);
    e.borrow_and_withdraw(
        &users.alice,
        &tokens.wnear,
        price_data(&tokens, Some(100000), None),
        borrow_amount,
    )
    .assert_success();

    let account = e.get_account(&users.alice);
    // The account should have 60$ of Net TVL. $100 from dai and 40$ wNEAR borrowed.
    assert_eq!(account.farms[0].rewards[0].boosted_shares, d(60, 18));

    let amount = d(100, 18);
    e.supply_to_collateral(&users.bob, &tokens.ndai, amount)
        .assert_success();

    let borrow_amount = d(1, 24);
    e.borrow_and_withdraw(
        &users.bob,
        &tokens.wnear,
        price_data(&tokens, Some(300000), None),
        borrow_amount,
    )
    .assert_success();

    e.account_farm_claim_all_on_behalf(&users.bob, &users.alice)
        .assert_success();

    let account = e.get_account(&users.alice);
    // The account has bad debt (more borrowed than collateral), so no net-tvl farm.
    assert!(account.farms.is_empty());

    let asset_farm = e.get_asset_farm(farm_id.clone());
    let reward = asset_farm
        .rewards
        .get(&tokens.ndai.account_id())
        .cloned()
        .unwrap();
    // Bob only
    assert_eq!(reward.boosted_shares, d(70, 18));

    let borrow_amount = d(1, 24);
    e.borrow_and_withdraw(
        &users.bob,
        &tokens.wnear,
        price_data(&tokens, Some(120000), None),
        borrow_amount,
    )
    .assert_success();

    let account = e.get_account(&users.bob);
    // 100 - 12 * 2
    assert_eq!(account.farms[0].rewards[0].boosted_shares, d(76, 18));

    e.account_farm_claim_all_on_behalf(&users.bob, &users.alice)
        .assert_success();

    let account = e.get_account(&users.alice);
    // 100 - 12 * 4
    assert_eq!(account.farms[0].rewards[0].boosted_shares, d(52, 18));

    let asset_farm = e.get_asset_farm(farm_id.clone());
    let reward = asset_farm
        .rewards
        .get(&tokens.ndai.account_id())
        .cloned()
        .unwrap();
    // Bob and Alice
    assert_eq!(reward.boosted_shares, d(128, 18));
}

#[test]
fn test_farm_net_tvl_multipliers() {
    let (e, tokens, users) = basic_setup();

    let reward_per_day = d(100, 18);
    let total_reward = d(3000, 18);

    let farm_id = FarmId::NetTvl;
    e.add_farm(
        farm_id.clone(),
        &tokens.ndai,
        reward_per_day,
        d(100, 18),
        total_reward,
    );

    let asset_farm = e.get_asset_farm(farm_id.clone());
    let reward = asset_farm
        .rewards
        .get(&tokens.ndai.account_id())
        .cloned()
        .unwrap();
    assert_eq!(reward.remaining_rewards, total_reward);

    e.owner
        .function_call(
            e.contract.contract.update_asset(
                tokens.wnear.account_id(),
                AssetConfig {
                    reserve_ratio: 2500,
                    target_utilization: 8000,
                    target_utilization_rate: U128(1000000000003593629036885046),
                    max_utilization_rate: U128(1000000000039724853136740579),
                    volatility_ratio: 6000,
                    extra_decimals: 0,
                    can_deposit: true,
                    can_withdraw: true,
                    can_use_as_collateral: true,
                    can_borrow: true,
                    net_tvl_multiplier: 8000,
                },
            ),
            DEFAULT_GAS.0,
            ONE_YOCTO,
        )
        .assert_success();

    let amount = d(100, 18);
    e.supply_to_collateral(&users.alice, &tokens.ndai, amount)
        .assert_success();

    let borrow_amount = d(4, 24);
    e.borrow_and_withdraw(
        &users.alice,
        &tokens.wnear,
        price_data(&tokens, Some(100000), None),
        borrow_amount,
    )
    .assert_success();

    let account = e.get_account(&users.alice);
    // 100 - 4 * 10 * 0.8
    assert_eq!(account.farms[0].rewards[0].boosted_shares, d(68, 18));

    // Deposit 10 wNEAR.
    let amount = d(10, 24);
    e.contract_ft_transfer_call(&tokens.wnear, &users.alice, amount, "")
        .assert_success();

    let account = e.get_account(&users.alice);
    // 100 - 4 * 10 * 0.8 + 10 * 10 * 0.8
    assert_eq!(account.farms[0].rewards[0].boosted_shares, d(148, 18));
}
