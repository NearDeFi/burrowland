# Burrowland contact

## How it works

### Interest model

The contract uses a compounding interest model similar to Aave.

Each asset defines interest rate configuration with the following values:

* `target_utilization` - the ideal percent at for the asset utilization, e.g. 80% borrowed comparing to the total supplied.
* `target_utilization_r` - the constant to use as a base for computing compounding APR at the target utilization.
* `max_utilization_r` - the constant to use as a base for computing compounding APR at the 100% utilization.
* `reserve_ratio` - the percentage of the acquired interest reserved for the platform.

Based on these values we define 3 points of utilization: `0%`, target utilization and `100%`.
For each of these points we have the `r` constant: `1.0`, `target_utilization_r` and `max_utilization_r` respectively.

To compute the APR, we can use the following formula:

`1 + APR = r ** MS_PER_YEAR`, where MS_PER_YEAR is the number of milliseconds in a year equal to `31536000000`. 

Based on the current supplied, reserved and borrowed balances, the current utilization is defined using the following formula:

`utilization = (supplied + reserved) / borrowed`

To compute current APR, we need to find the current `r` constant based on the linear interpolation between utilization points:

* if `utilization <= target_utilization`, `r = target_utilization_r * (utilization / target_utilization)`
* if `utilization > target_utilization`, `r = target_utilization_r + (max_utilization_r - target_utilization_r) * (utilization - target_utilization) / (1 - target_utilization)` 

To calculate the amount of interest acquired for the duration of `t` milliseconds, we can use the following formula:

`interest = (r ** t) * borrowed`

The interest are distributed to `reserved` and `supplied`, based on `reserve_ratio`, so the new values are:

```
reserved_interest = interest * reserve_ratio
new_reserved = reserved + reserved_interest
new_supplied = supplied + (interest - reserved_interest)
new_borrowed = borrowed + interest 
```

### Health factor

The health factor is computed per account instead of per asset.

Each account may supply multiple collateral assets and may borrow multiple assets.

Each asset has a configuration value `volatility_ratio` which indicates the expected price stability factor.
The higher the ratio, the higher expectation of the stability of the price of the corresponding asset.

To compute the current health factor for the account, we need to know the current prices of all collateral and borrowed assets.
Firstly, we compute the adjusted for volatility sums of all collateral assets and borrowed assets.

```
adjusted_collateral_sum = sum(collateral_i * price_i * volatility_ratio_i)
adjusted_borrowed_sum = sum(borrowed_i * price_i / volatility_ratio_i)
```

Now we can compute the health factor:

`health_factor = adjusted_collateral_sum / adjusted_borrowed_sum`

If the health factor is higher than 100%, it means the account is in a good state and can't be liquidated.
If the health factor is less than 100%, it means the account can be partially liquidated and can't borrow more without
repaying some amount of the existing assets or providing more collateral assets.

### Liquidations

Contract liquidations are designed to make liquidators compete for the profit that they make during liquidations to
minimize the loss taken by the unhealthy accounts. Instead of the fixed profit that is used in the legacy products,
this contract introduces a variable discount with variable liquidation size.

Liquidations rules:
1. the initial health factor of the liquidated accounts has to be below 100%
2. the discounted sum of the taken collateral should be less than the sum of repaid assets  
3. the final health factor of the liquidated accounts has to stay below 100%
4. the final health factor of the liquidated account should be higher than the initial health factor

A liquidation action consists of the following:
- `account_id` - the account ID that is being liquidated
- `in_assets` - the assets and corresponding amounts to repay form borrowed assets
- `out_assets` - the assets and corresponding amounts to take from collateral assets

The discount is computed based on the initial health factor of the liquidated account:

`discount = (1 - health_factor) / 2`

Now we can compute the taken discounted collateral sum and the repaid borrowed sum:

```
taken_sum = sum(out_asset_i * price_i)
discounted_collateral_sum = taken_sum * (1 - discount)
repaid_sum = sum(in_asset_i * price_i)
```

Once we action is completed, we can compute the final values and verify the liquidation rules:

1. `initial_health_factor < 100%`
2. `discounted_collateral_sum <= repaid_sum`
3. `new_health_factor < 100%`
3. `new_health_factor > initial_health_factor`

The first rule only allows to liquidate accounts in the unhealthy state.
The second rule prevents from taking more collateral than the repaid sum (after discount).
The third rule prevents the liquidator from repaying too much of the borrowed assets, only enough to bring closer to the 100%.

#### Liquidation example

Account `alice.near` supplied to collateral `1000 wNEAR` and borrowed `4000 nDAI`.

Let's say:
- the price of `wNEAR` is `10`
- the price of the `nDAI` is `1`
- the `volatility_ratio` of `wNEAR` is `0.5`
- the `volatility_ratio` of `nDAI` is `1`

The health factor of `alice.near` is the following:

```
adjusted_collateral_sum = sum(1000 * 10 * 0.5) = 5000
adjusted_borrowed_sum = sum(4000 * 1 / 1) = 4000
health_factor = 5000 / 4000 = 125% 
```

Let's say the price of `wNEAR` drops to `8`

```
adjusted_collateral_sum = sum(1000 * 8 * 0.5) = 4000
adjusted_borrowed_sum = sum(4000 * 1 / 1) = 4000
health_factor = 4000 / 4000 = 100% 
```

The health factor is 100%, so the account still can't be liquidated.

Let's say the price of `wNEAR` drops to `7`

```
adjusted_collateral_sum = sum(1000 * 7 * 0.5) = 3500
adjusted_borrowed_sum = sum(4000 * 1 / 1) = 4000
health_factor = 3500 / 4000 = 0.875 = 87.5% 
```

The health factor is below 100%, so the account can be liquidated. The discount is the following:

```
discount = (1 - 0.875) / 2 = 0.0625 = 6.25%
```

It means anyone can repay some `nDAI` and take some `wNEAR` from `alice.near` with `6.25%` discount.  

Account `bob.near` decides to liquidate `alice.near`

`bob.near` wants to repay `1000 nDAI`, we can compute the maximum sum of the collateral to take:

```
repaid_sum = sum(1000 * 1) = 1000
max_taken_sum = repaid_sum / (1 - discount) = 1000 / (1 - 0.0625) = 1066.666
```

And based on the `wNEAR` price, we can compute the maximum amount:

```
max_wnear_amount = max_taken_sum / wnear_price = 1066.666 / 7 = 152.38
```

But to avoid risk, `bob.near` takes `152` `wNEAR` - a bit less to avoid price fluctuation for the duration of the transaction.

Let's compute the liquidation action:

```
taken_sum = sum(out_asset_i * price_i) = sum(152 * 7) = 1064
discounted_collateral_sum = taken_sum * (1 - discount) = 1064 * (1 - 0.0625) = 997.5
repaid_sum = sum(in_asset_i * price_i) = sum(1000 * 1) = 1000

new_adjusted_collateral_sum = sum((1000 - 152) * 7 * 0.5) = 2968
new_adjusted_borrowed_sum = sum((4000 - 1000) * 1 / 1) = 3000

new_health_factor = 2968 / 3000 = 0.9893 = 98.93%
```

Now checking the liquidation rules:

```
1. 87.5% < 100%
2. 997.5 <= 1000
3. 98.93% < 100%
4. 98.93% > 87.5%
```

All rules satisfied, so the liquidation was successful.

Now, let's compute the profit of `bob.near` (or the loss for `alice.near`) for this liquidation:
```
profit = taken_sum - repaid_sum = 1064 - 1000 = 64
```

Notes:
- If someone during the time when the price of `wNEAR` was falling from `8` to `7` liquidated `alice.near` they would have
  made less profit, by liquidating a smaller amount with a smaller collateral discount.
- To fully realize the profit, `bob.near` has to take another action on some exchange and swap received `152` `wNEAR` for `nDAI`, which 
  may involve extra fees and transactional risks. That's why liquidators may wait for higher discount.

### Booster token

One of the assets may be designated as the Booster token. This asset can be staked (locked) for some given duration to become xBooster token.
The amount of xBooster token depends on the amount of Booster token staked, the previous amount of xBooster token and the duration of the stake.
xBooster token is non-transferrable and can not be unstaked until the staking duration expires.

xBooster token may be used to increase farm multiplier. See Farms section for details.

The following parameters in the config are used for Booster logic:
- `booster_token_id` - the asset ID that is used as a booster token.
- `booster_decimals` - the number of digits after decimal point for the booster token.
- `minimum_staking_duration_sec` - the minimum duration in seconds that the booster token can be staked to get xBooster token.
- `maximum_staking_duration_sec` - the maximum duration in seconds that the booster token can be staked to get xBooster token.
- `x_booster_multiplier_at_maximum_staking_duration` - the multiplier of xBooster amount relative to Booster amount given at the maximum staking duration.

The account can only have one staking duration. It means if the account has staked some amount of Booster token before, the restaking can only 
be done for a longer duration from the current moment, than the expiration of the previous stake from the previous moment of staking.

For example if the account previously staked Booster for `6` months `1` month ago. The new staking duration should be at least `5` months (`6 - 1`).

To stake Booster token, an account should call `account_stake_booster` and pass `amount` (optional) and `duration` in seconds.
- If the `amount` is not given than all existing booster token from the supplied assets of this account will be staked.
- The duration should within range of `minimum_staking_duration_sec` and `maximum_staking_duration_sec` inclusive.
- The `current time + duration` should be not less than the previous `unlock time` (if the previous stake exists).

The xBooster multiplier for a given duration is computed using the following formula:

`xbooster_multiplier(duration) = 1 + (duration - minimum_staking_duration_sec) / (maximum_staking_duration_sec - minimum_staking_duration_sec) * (x_booster_multiplier_at_maximum_staking_duration / 10000 - 1)`

If the previous xBooster exists, then we have the following values:
- `staked_booster_amount` - the total amount of Booster token staked by the account.
- `x_booster_amount` - the total amount of xBooster token received for staking.
- `unlock_timestamp` - the timestamp when xBooster can be unstaked to get back the total Booster amount.

Compute the extra xBooster received for the new given `amount` and the `duration`:

`extra_x_booster_amount = amount * xbooster_multiplier(duration)`

If the previous xBooster stake exists then we compute the potential xBooster amount for the previous Booster token amount and the new duration, and 
then compute the actual total new xBooster amount from the maximum of the restaking or keeping the old xBooster amount:

```
potential_x_booster_amount = staked_booster_amount * xbooster_multiplier(duration)
total_xbooster_amount = max(x_booster_amount, potential_x_booster_amount) + extra_x_booster_amount
```

Once staked we remember the total Booster token amount staked, the new total xBooster amount and the unlock timestamp.
- `staked_booster_amount = staked_booster_amount + amount`
- `x_booster_amount = total_xbooster_amount`
- `unlock_timestamp = current time + sec_to_nano(duration)`

### Farms

A farm is identified by the type of the asset (supplied or borrowed) and the asset ID.
That's why there can be at most `N * 2` number of farms, where `N` is the number of different assets.

A farm can have multiple rewards: one reward per asset ID.

Each farm reward is identified by the asset ID it gives. The reward config includes:
- `reward_per_day` - the amount of tokens split across farms participants daily based on their number of boosted shares (until there are no more remaining rewards).
- `booster_log_base` - the log base for the xbooster amounts. It's used to compute boosted shares per account. The number includes decimals of the xBooster token. E.g. `100 * 1e18` is the log base of `100`, if xBooster has `18` decimals.
- `remaining_rewards` - the amount of the remaining tokens to be distributed.

For example to create a farm for `30` days and distribute `1000` tokens per day, the `reward_per_day` should be set to `1000` and the remaining rewards will be `30000`.

Once the `remaining_rewards` becomes equal to `0`, the farm stops distributing this reward.

#### Farm booster

The farming multiplier for each specific farm is calculated based on `booster_log_base` and `x_booster_amount` for the account.

`booster_decimals` is the number of decimals of the booster token. By default, it's `18`.

`booster_base = 10 ** token_decimals`

If the account has `x_booster_amount <= booster_base`, then the multiplier is `1`

If `x_booster_amount > booster_base`, then the farming multiplier is the following:

`farming_multiplier = 1 + log(x_booster_amount / booster_base) / log(booster_log_base / booster_base)`

##### Farming multiplier example

Let's say `booster_log_base` is `20 * 1e18`.

If an account has `x_booster_amount` equal to:
- `0`, then `farming_multiplier = 1`
- `1 * 1e18`, then `farming_multiplier = 1`
- `20 * 1e18`, then `farming_multiplier = 2`
- `400 * 1e18`, then `farming_multiplier = 3`
- `5 * 1e18`, then `farming_multiplier = 1.53724357368`

##### Farm example

Let's say there is a farm for supplying USDC that gives 200 wNEAR per day and there are 2 users Alice and Bob.

The farm has the following config:
- `reward_per_day = 200 * 1e24` - 200 wNEAR per day
- `booster_log_base = 10 * 1e18` - 10 log base for xBooster
- `remaining_rewards = 30 * 200 * 1e24` - 30 days

- Alice puts `300` USDC in to supplied and has `100` xBooster tokens.
- Bob puts `50` USDC in to supplied and has `10` xBooster tokens.

Boosted shares are computed the following way:
- Alice gets farming multiplier of `3X`, so the number of boosted shares becomes `900 (300 * 3)`.
- Bob gets farming multiplier of `2X`, so the number of boosted shares becomes `100 (50 * 2)`.
- The total is `1000 (900 + 100)` boosted shares.

So Alice gets `90%` of the rewards and Bob gets `10%` of the rewards.

So by the end of the day:
- Alice gets `180` wNEAR per day.
- Bob gets `20` wNEAR per day.

Let's Charlie supplies `1000` USDC without xBooster tokens.
- This adds `1000` boosted shares for Charlie. Now the total amount of boosted shares is `2000`.
- Alice gets `45%` of the farm rewards equal to `90` wNEAR per day.
- Bob gets `5%` of the farm rewards equal to `10` wNEAR per day.
- Charlie gets `50%` of the farm rewards equal to `100` wNEAR per day.

## Development

### Build (optional)

Requires Rust and wasm32 target.

```bash
./build.sh
```

### Deploy on the testnet

Requires NEAR CLI to be installed.

```bash
./scripts/dev_deploy.sh
```

This will provide a list of exports at the end. Execute them to get the CLI ready for further transactions.

Example exports:
```bash
export NEAR_ENV=testnet
export OWNER_ID=dev-1634411537975-18277461139961
export ORACLE_ID=dev-1634411553736-97024242878560
export CONTRACT_ID=dev-1634411561699-94876475207945
export BOOSTER_TOKEN_ID=ref.fakes.testnet
export WETH_TOKEN_ID=weth.fakes.testnet
export DAI_TOKEN_ID=dai.fakes.testnet
export USDT_TOKEN_ID=usdt.fakes.testnet
export WNEAR_TOKEN_ID=wrap.testnet
export ONE_YOCTO=0.000000000000000000000001
export GAS=200000000000000
```

### Create a test account

Requires exports from running `dev_deploy`. 

```bash
./scripts/create_account.sh
```

This will create a new test account with fake assets. Execute the export at the end to get the account ID.

Example export:
```bash
export ACCOUNT_ID=dev-1634680029152-10252684568108
```

## Actions

### Register account by paying for storage

This has to be done one per account.

```bash
near call $CONTRACT_ID --accountId=$ACCOUNT_ID --gas=$GAS --amount=0.1 storage_deposit '{}'
```

### Supply some token

Let's supply `5` USDT. USDT has `6` decimals, so amount should be `5000000`. For a simple deposit, the `msg` can be empty string.

```bash
near call $USDT_TOKEN_ID --accountId=$ACCOUNT_ID --gas=$GAS --amount=$ONE_YOCTO ft_transfer_call '{
  "receiver_id": "'$CONTRACT_ID'",
  "amount": "5000000",
  "msg": ""
}'
```

### View account information

```bash
near view $CONTRACT_ID get_account '{"account_id": "'$ACCOUNT_ID'"}'
```

Example result:
```javascript
{
  account_id: 'dev-1634682124572-99167526870966',
  supplied: [
    {
      token_id: 'usdt.fakes.testnet',
      balance: '5000000000000000000',
      shares: '5000000000000000000'
    }
  ],
  collateral: [],
  borrowed: [],
  farms: []
}
```

Note: Since USDT asset has extra `12` decimals, it brings the 5 USDT in the balance to `5000000000000000000`

### View a given asset

```bash
near view $CONTRACT_ID get_asset '{"token_id": "'$USDT_TOKEN_ID'"}'
```

Example result:
```javascript
{
  supplied: { shares: '5000000000000000000', balance: '5000000000000000000' },
  borrowed: { shares: '0', balance: '0' },
  reserved: '2000000000000000000000',
  last_update_timestamp: '1634682347763275349',
  config: {
    reserve_ratio: 2500,
    target_utilization: 8000,
    target_utilization_rate: '1000000000002440418605283556',
    max_utilization_rate: '1000000000039724853136740579',
    volatility_ratio: 9500,
    extra_decimals: 12,
    can_deposit: true,
    can_withdraw: true,
    can_use_as_collateral: true,
    can_borrow: true
  }
}
```

Note: You can also see `2000000000000000000000` reserved. That's `2000` USDT from the owner.

### Provide token as a collateral

Let's add all USDT to a collateral. If the `amount` for a given action is not specified, then all available amount will be used.

Increasing the collateral doesn't require prices from the oracle, because it can't decrease the existing collateral.

```bash
near call $CONTRACT_ID --accountId=$ACCOUNT_ID --gas=$GAS --amount=$ONE_YOCTO execute '{
  "actions": [
    {
      "IncreaseCollateral": {
        "token_id": "'$USDT_TOKEN_ID'"
      }
    }
  ]
}'
```

Let's view the account info again:

```bash
near view $CONTRACT_ID get_account '{"account_id": "'$ACCOUNT_ID'"}'
```

Example result:
```javascript
{
  account_id: 'dev-1634682124572-99167526870966',
  supplied: [],
  collateral: [
    {
      token_id: 'usdt.fakes.testnet',
      balance: '5000000000000000000',
      shares: '5000000000000000000'
    }
  ],
  borrowed: [],
  farms: []
}
```

Note, you can see the USDT asset was moved from `supplied` to `collateral`

### Borrow a token

Let's borrow `1` DAI. DAI has `18` decimals, so the amount should be `1000000000000000000`.

Since borrow action puts account into the debt, we have to call this action through the oracle.
The oracle should provide prices for all assets in the collateral as well as all existing borrowed assets and the new borrowed asset.

The `msg` passed to the oracle should be string. Since it's part of the JSON, it has to be double-encoded and can't have newlines.

FYI: The message that we pass to the contract from the oracle is the following:
```json
{
  "Execute": {
    "actions": [
      {
        "Borrow": {
          "token_id": "dai.fakes.testnet",
          "amount": "1000000000000000000"
        }
      }
    ]
  }
}
```

```bash
near call $ORACLE_ID --accountId=$ACCOUNT_ID --gas=$GAS --amount=$ONE_YOCTO oracle_call '{
  "receiver_id": "'$CONTRACT_ID'",
  "asset_ids": [
    "'$USDT_TOKEN_ID'",
    "'$DAI_TOKEN_ID'"
  ],
  "msg": "{\"Execute\": {\"actions\": [{\"Borrow\": {\"token_id\": \"'$DAI_TOKEN_ID'\", \"amount\": \"1000000000000000000\"}}]}}"
}'
```

You should see a log message like: `Account dev-1634682124572-99167526870966 borrows 1000000000000000000 of dai.fakes.testnet`

Let's view the account info again:

```bash
near view $CONTRACT_ID get_account '{"account_id": "'$ACCOUNT_ID'"}'
```

Example result:
```javascript
{
  account_id: 'dev-1634682124572-99167526870966', 
  supplied: [
    {
      token_id: 'dai.fakes.testnet',
      balance: '1000000000048216105',
      shares: '1000000000000000000'
    }
  ],
  collateral: [
    {
      token_id: 'usdt.fakes.testnet',
      balance: '5000000000000000000',
      shares: '5000000000000000000'
    }
  ],
  borrowed: [
    {
      token_id: 'dai.fakes.testnet',
      balance: '1000000000064288139',
      shares: '1000000000000000000'
    }
  ],
  farms: []
}
```

Note, without extra action the borrowed assets are not withdrawn to the wallet, but instead supplied to earn interest.
From there they can be withdrawn.
You can also notice that the borrowed balance is larger than the supplied balance, that's because the some of the interest are going to the reserve.

If we view the account info again, then the balances should increase:

Example result:
```javascript
{
  account_id: 'dev-1634682124572-99167526870966',
  supplied: [
    {
      token_id: 'dai.fakes.testnet',
      balance: '1000000000221528817',
      shares: '1000000000000000000'
    }
  ],
  collateral: [
    {
      token_id: 'usdt.fakes.testnet',
      balance: '5000000000000000000',
      shares: '5000000000000000000'
    }
  ],
  borrowed: [
    {
      token_id: 'dai.fakes.testnet',
      balance: '1000000000295371755',
      shares: '1000000000000000000'
    }
  ],
  farms: []
}
```

Let's view the DAI asset info:

```bash
near view $CONTRACT_ID get_asset '{"token_id": "'$DAI_TOKEN_ID'"}'
```

Example result:
```javascript
{
  supplied: { shares: '1000000000000000000', balance: '1000000000399150907' },
  borrowed: { shares: '1000000000000000000', balance: '1000000000532201209' },
  reserved: '2000000000000133050302',
  last_update_timestamp: '1634683708614246127',
  config: {
    reserve_ratio: 2500,
    target_utilization: 8000,
    target_utilization_rate: '1000000000002440418605283556',
    max_utilization_rate: '1000000000039724853136740579',
    volatility_ratio: 9500,
    extra_decimals: 0,
    can_deposit: true,
    can_withdraw: true,
    can_use_as_collateral: true,
    can_borrow: true
  }
}
```

### Withdrawing the asset

Let's withdraw all DAI including interest.

Withdrawing doesn't need oracle prices, because it can only be taken from the supplied and not from the collateral.

```bash
near call $CONTRACT_ID --accountId=$ACCOUNT_ID --gas=$GAS --amount=$ONE_YOCTO execute '{
  "actions": [
    {
      "Withdraw": {
        "token_id": "'$DAI_TOKEN_ID'"
      }
    }
  ]
}'
```

You should see the log, e.g. `Account dev-1634682124572-99167526870966 withdraws 1000000001658903820 of dai.fakes.testnet`

Now let's check the DAI balance (in the wallet) of the account:

```bash
near view $DAI_TOKEN_ID ft_balance_of '{"account_id": "'$ACCOUNT_ID'"}'
```

Example result: `10001000000001658903820`, which corresponds roughly to `10001` DAI, plus some extra earned interests.

Withdrawal from the contract was possible, because the owner has supplied DAI into the reserve.

Let's view the account info again:

```bash
near view $CONTRACT_ID get_account '{"account_id": "'$ACCOUNT_ID'"}'
```

Example result:
```javascript
{
  account_id: 'dev-1634682124572-99167526870966',
  supplied: [],
  collateral: [
    {
      token_id: 'usdt.fakes.testnet',
      balance: '5000000000000000000',
      shares: '5000000000000000000'
    }
  ],
  borrowed: [
    {
      token_id: 'dai.fakes.testnet',
      balance: '1000000002496596924',
      shares: '1000000000000000000'
    }
  ],
  farms: []
}
```

Notice, there is no supplied DAI anymore.

Let's view the DAI asset info:

```bash
near view $CONTRACT_ID get_asset '{"token_id": "'$DAI_TOKEN_ID'"}'
```

Example result:
```javascript
{
  supplied: { shares: '0', balance: '0' },
  borrowed: { shares: '1000000000000000000', balance: '1000000002551410252' },
  reserved: '2000000000000892506432',
  last_update_timestamp: '1634685033009246127',
  config: {
    reserve_ratio: 2500,
    target_utilization: 8000,
    target_utilization_rate: '1000000000002440418605283556',
    max_utilization_rate: '1000000000039724853136740579',
    volatility_ratio: 9500,
    extra_decimals: 0,
    can_deposit: true,
    can_withdraw: true,
    can_use_as_collateral: true,
    can_borrow: true
  }
}
```

### Deposit asset and repay it in one call.

Note, multiple actions can be combined into a single atomic update. Either all of them complete or all of them are reverted.
The invariants are only checked at the end, so this may be used to replace one collateral with another without repaying debts (but this requires oracle pricing). 

Let's deposit `5` DAI and use it to repay borrowed DAI. DAI has 18 decimal, so the amount is `5000000000000000000`
For this we need to pass a custom `msg` to `ft_transfer_call`.
The message has to be double-encoded into a string.

FYI: Non-encoded message in JSON:
```json
{
  "Execute": {
    "actions": [
      {
        "Repay": {
          "token_id": "dai.fakes.testnet"
        }
      }
    ]
  }
}
```

```bash
near call $DAI_TOKEN_ID --accountId=$ACCOUNT_ID --gas=$GAS --amount=$ONE_YOCTO ft_transfer_call '{
  "receiver_id": "'$CONTRACT_ID'",
  "amount": "5000000000000000000",
  "msg": "{\"Execute\": {\"actions\": [{\"Repay\": {\"token_id\": \"'$DAI_TOKEN_ID'\"}}]}}"
}'
```

You should see similar log messages:
```
Account dev-1634686749015-49146327775274 deposits 5000000000000000000 of dai.fakes.testnet
Account dev-1634686749015-49146327775274 repays 1000000001735752696 of dai.fakes.testnet
```

Let's view the account info again:

```bash
near view $CONTRACT_ID get_account '{"account_id": "'$ACCOUNT_ID'"}'
```

Example result:
```javascript
{
  account_id: 'dev-1634686749015-49146327775274',
  supplied: [
    {
      token_id: 'dai.fakes.testnet',
      balance: '3999999998264247304',
      shares: '3999999998264247304'
    }
  ],
  collateral: [
    {
      token_id: 'usdt.fakes.testnet',
      balance: '5000000000000000000',
      shares: '5000000000000000000'
    }
  ],
  borrowed: [],
  farms: []
}
```

Notice, there is no borrowed DAI anymore.

Let's view the DAI asset info:

```bash
near view $CONTRACT_ID get_asset '{"token_id": "'$DAI_TOKEN_ID'"}'
```

Example result:
```javascript
{
  supplied: { shares: '3999999998264247304', balance: '3999999998264247304' },
  borrowed: { shares: '0', balance: '0' },
  reserved: '2000000000001727179674',
  last_update_timestamp: '1634688121573861187',
  config: {
    reserve_ratio: 2500,
    target_utilization: 8000,
    target_utilization_rate: '1000000000002440418605283556',
    max_utilization_rate: '1000000000039724853136740579',
    volatility_ratio: 9500,
    extra_decimals: 0,
    can_deposit: true,
    can_withdraw: true,
    can_use_as_collateral: true,
    can_borrow: true
  }
}
```

And no borrowed balance or shares after repaying.

### Decreasing collateral

Since there is no borrowed assets, we can take the collateral without providing prices.

Let's get all USDT collateral back.

```bash
near call $CONTRACT_ID --accountId=$ACCOUNT_ID --gas=$GAS --amount=$ONE_YOCTO execute '{
  "actions": [
    {
      "DecreaseCollateral": {
        "token_id": "'$USDT_TOKEN_ID'"
      }
    }
  ]
}'
```

Let's view the account info again:

```bash
near view $CONTRACT_ID get_account '{"account_id": "'$ACCOUNT_ID'"}'
```

Example result:
```javascript
{
  account_id: 'dev-1634686749015-49146327775274',
  supplied: [
    {
      token_id: 'dai.fakes.testnet',
      balance: '3999999998264247304',
      shares: '3999999998264247304'
    },
    {
      token_id: 'usdt.fakes.testnet',
      balance: '5000000000000000000',
      shares: '5000000000000000000'
    }
  ],
  collateral: [],
  borrowed: [],
  farms: []
}
```
