# Burrowland contact

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
