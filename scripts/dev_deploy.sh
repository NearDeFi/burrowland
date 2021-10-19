#!/bin/bash
set -e

cd "$(dirname $0)/.."

export NEAR_ENV=testnet
LG='\033[1;30m' # Arrows color (Dark gray)
TC='\033[0;33m' # Text color (Orange)
NC='\033[0m' # No Color

echo -e "$LG>>>>>>>>>>>>>>$TC Deploy an empty contract to create an owner account $LG<<<<<<<<<<<<<<$NC"
echo -n "" > /tmp/empty
near dev-deploy -f /tmp/empty
OWNER_ID="$(cat neardev/dev-account)"

echo -e "$LG>>>>>>>>>>>>>>$TC Deploy the test oracle $LG<<<<<<<<<<<<<<$NC"
near dev-deploy -f res/test_oracle.wasm
ORACLE_ID="$(cat neardev/dev-account)"

echo -e "$LG>>>>>>>>>>>>>>$TC Deploy the main contract $LG<<<<<<<<<<<<<<$NC"
near dev-deploy -f res/burrowland.wasm
CONTRACT_ID="$(cat neardev/dev-account)"

echo -e "$LG>>>>>>>>>>>>>>$TC Preparing Booster token $LG<<<<<<<<<<<<<<$NC"
BOOSTER_TOKEN_ID="ref.fakes.testnet"

near call $BOOSTER_TOKEN_ID --accountId=$CONTRACT_ID storage_deposit '' --amount=0.00125
near call $BOOSTER_TOKEN_ID --accountId=$OWNER_ID storage_deposit '' --amount=0.00125

echo -e "$LG>>>>>>>>>>>>>>$TC Initializing the main contract $LG<<<<<<<<<<<<<<$NC"
near call $CONTRACT_ID --accountId=$CONTRACT_ID new '{"config": {
  "oracle_account_id": "'$ORACLE_ID'",
  "owner_id": "'$OWNER_ID'",
  "booster_token_id": "'$BOOSTER_TOKEN_ID'",
  "booster_decimals": 18
}}'

echo -e "$LG>>>>>>>>>>>>>>$TC Preparing wETH $LG<<<<<<<<<<<<<<$NC"
WETH_TOKEN_ID="weth.fakes.testnet"

near call $WETH_TOKEN_ID --accountId=$CONTRACT_ID storage_deposit '' --amount=0.00125
near call $WETH_TOKEN_ID --accountId=$OWNER_ID storage_deposit '' --amount=0.00125

echo -e "$LG>>>>>>>>>>>>>>$TC Preparing DAI $LG<<<<<<<<<<<<<<$NC"
DAI_TOKEN_ID="dai.fakes.testnet"

near call $DAI_TOKEN_ID --accountId=$CONTRACT_ID storage_deposit '' --amount=0.00125
near call $DAI_TOKEN_ID --accountId=$OWNER_ID storage_deposit '' --amount=0.00125

echo -e "$LG>>>>>>>>>>>>>>$TC Preparing USDT $LG<<<<<<<<<<<<<<$NC"
USDT_TOKEN_ID="usdt.fakes.testnet"

near call $USDT_TOKEN_ID --accountId=$CONTRACT_ID storage_deposit '' --amount=0.00125
near call $USDT_TOKEN_ID --accountId=$OWNER_ID storage_deposit '' --amount=0.00125

echo -e "$LG>>>>>>>>>>>>>>$TC Preparing wNEAR $LG<<<<<<<<<<<<<<$NC"
WNEAR_TOKEN_ID="wrap.testnet"

near call $WNEAR_TOKEN_ID --accountId=$CONTRACT_ID storage_deposit '' --amount=0.00125
near call $WNEAR_TOKEN_ID --accountId=$OWNER_ID storage_deposit '' --amount=0.00125

echo -e "$LG>>>>>>>>>>>>>>$TC Initializing assets $LG<<<<<<<<<<<<<<$NC"
ONE_YOCTO="0.000000000000000000000001"
GAS="200000000000000"

# Booster APR is 30%, to verify run ./scripts/apr_to_rate.py 30
# Max APR for all assets is 250%
# Booster can't be used as a collateral or borrowed (for now), so APR doesn't matter.
near call $CONTRACT_ID --accountId=$OWNER_ID add_asset '{
  "token_id": "'$BOOSTER_TOKEN_ID'",
  "asset_config": {
    "reserve_ratio": 2500,
    "target_utilization": 8000,
    "target_utilization_rate": "1000000000008319516250272147",
    "max_utilization_rate": "1000000000039724853136740579",
    "volatility_ratio": 2000,
    "extra_decimals": 0,
    "can_deposit": true,
    "can_withdraw": true,
    "can_use_as_collateral": false,
    "can_borrow": false
  }
}' --amount=$ONE_YOCTO --gas=$GAS


# wETH APR is 6%, to verify run ./scripts/apr_to_rate.py 5
# Volatility ratio is 60%, since it's somewhat liquid on NEAR
near call $CONTRACT_ID --accountId=$OWNER_ID add_asset '{
  "token_id": "'$WETH_TOKEN_ID'",
  "asset_config": {
    "reserve_ratio": 2500,
    "target_utilization": 8000,
    "target_utilization_rate": "1000000000001547125956667610",
    "max_utilization_rate": "1000000000039724853136740579",
    "volatility_ratio": 6000,
    "extra_decimals": 0,
    "can_deposit": true,
    "can_withdraw": true,
    "can_use_as_collateral": true,
    "can_borrow": true
  }
}' --amount=$ONE_YOCTO --gas=$GAS

# DAI APR is 8%, to verify run ./scripts/apr_to_rate.py 8
# Volatility ratio is 95%, since it's stable and liquid on NEAR
near call $CONTRACT_ID --accountId=$OWNER_ID add_asset '{
  "token_id": "'$DAI_TOKEN_ID'",
  "asset_config": {
    "reserve_ratio": 2500,
    "target_utilization": 8000,
    "target_utilization_rate": "1000000000002440418605283556",
    "max_utilization_rate": "1000000000039724853136740579",
    "volatility_ratio": 9500,
    "extra_decimals": 0,
    "can_deposit": true,
    "can_withdraw": true,
    "can_use_as_collateral": true,
    "can_borrow": true
  }
}' --amount=$ONE_YOCTO --gas=$GAS

# USDT APR is 8%, to verify run ./scripts/apr_to_rate.py 8
# Volatility ratio is 95%, since it's stable and liquid on NEAR
# It has extra 12 decimals, to bring total to 18
near call $CONTRACT_ID --accountId=$OWNER_ID add_asset '{
  "token_id": "'$USDT_TOKEN_ID'",
  "asset_config": {
    "reserve_ratio": 2500,
    "target_utilization": 8000,
    "target_utilization_rate": "1000000000002440418605283556",
    "max_utilization_rate": "1000000000039724853136740579",
    "volatility_ratio": 9500,
    "extra_decimals": 0,
    "can_deposit": true,
    "can_withdraw": true,
    "can_use_as_collateral": true,
    "can_borrow": true
  }
}' --amount=$ONE_YOCTO --gas=$GAS

# wNEAR APR is 12%, to verify run ./scripts/apr_to_rate.py 12
# Target utilization is 60% (for some reason)
# Volatility ratio is 60%
near call $CONTRACT_ID --accountId=$OWNER_ID add_asset '{
  "token_id": "'$WNEAR_TOKEN_ID'",
  "asset_config": {
    "reserve_ratio": 2500,
    "target_utilization": 6000,
    "target_utilization_rate": "1000000000003593629036885046",
    "max_utilization_rate": "1000000000039724853136740579",
    "volatility_ratio": 6000,
    "extra_decimals": 0,
    "can_deposit": true,
    "can_withdraw": true,
    "can_use_as_collateral": true,
    "can_borrow": true
  }
}' --amount=$ONE_YOCTO --gas=$GAS

echo -e "$LG>>>>>>>>>>>>>>$TC Minting tokens for the owner and the user: $LG<<<<<<<<<<<<<<$NC"
echo -e "$LG>>>>>>>>>>>>>>$TC * 120000 BOOSTER $NC"
echo -e "$LG>>>>>>>>>>>>>>$TC * 12 wETH $NC"
echo -e "$LG>>>>>>>>>>>>>>$TC * 12000 DAI $NC"
echo -e "$LG>>>>>>>>>>>>>>$TC * 12000 USDT $NC"
echo -e "$LG>>>>>>>>>>>>>>$TC * 120 wNEAR (wrapped) $NC"
near call $BOOSTER_TOKEN_ID --accountId=$OWNER_ID mint '{
  "account_id": "'$OWNER_ID'",
  "amount": "120000000000000000000000"
}'
near call $WETH_TOKEN_ID --accountId=$OWNER_ID mint '{
  "account_id": "'$OWNER_ID'",
  "amount": "12000000000000000000"
}'
near call $DAI_TOKEN_ID --accountId=$OWNER_ID mint '{
  "account_id": "'$OWNER_ID'",
  "amount": "12000000000000000000000"
}'
near call $USDT_TOKEN_ID --accountId=$OWNER_ID mint '{
  "account_id": "'$OWNER_ID'",
  "amount": "12000000000"
}'


near call $WNEAR_TOKEN_ID --accountId=$OWNER_ID near_deposit '{}' --amount=120

echo -e "$LG>>>>>>>>>>>>>>$TC Adding some reserves from the owner: $LG<<<<<<<<<<<<<<$NC"
echo -e "$LG>>>>>>>>>>>>>>$TC * 2 wETH $NC"
echo -e "$LG>>>>>>>>>>>>>>$TC * 2000 DAI $NC"
echo -e "$LG>>>>>>>>>>>>>>$TC * 2000 USDT $NC"
echo -e "$LG>>>>>>>>>>>>>>$TC * 20 wNEAR $NC"
near call $WETH_TOKEN_ID --accountId=$OWNER_ID ft_transfer_call '{
  "receiver_id": "'$CONTRACT_ID'",
  "amount": "2000000000000000000",
  "msg": "\"DepositToReserve\""
}' --amount=$ONE_YOCTO --gas=$GAS

near call $DAI_TOKEN_ID --accountId=$OWNER_ID ft_transfer_call '{
  "receiver_id": "'$CONTRACT_ID'",
  "amount": "2000000000000000000000",
  "msg": "\"DepositToReserve\""
}' --amount=$ONE_YOCTO --gas=$GAS

near call $USDT_TOKEN_ID --accountId=$OWNER_ID ft_transfer_call '{
  "receiver_id": "'$CONTRACT_ID'",
  "amount": "2000000000",
  "msg": "\"DepositToReserve\""
}' --amount=$ONE_YOCTO --gas=$GAS

near call $WNEAR_TOKEN_ID --accountId=$OWNER_ID ft_transfer_call '{
  "receiver_id": "'$CONTRACT_ID'",
  "amount": "20000000000000000000000000",
  "msg": "\"DepositToReserve\""
}' --amount=$ONE_YOCTO --gas=$GAS

echo -e "$LG>>>>>>>>>>>>>>$TC Dropping info to continue working from NEAR CLI: $LG<<<<<<<<<<<<<<$NC"
echo -e "export NEAR_ENV=testnet"
echo -e "export OWNER_ID=$OWNER_ID"
echo -e "export ORACLE_ID=$ORACLE_ID"
echo -e "export CONTRACT_ID=$CONTRACT_ID"
echo -e "export BOOSTER_TOKEN_ID=$BOOSTER_TOKEN_ID"
echo -e "export WETH_TOKEN_ID=$WETH_TOKEN_ID"
echo -e "export DAI_TOKEN_ID=$DAI_TOKEN_ID"
echo -e "export USDT_TOKEN_ID=$USDT_TOKEN_ID"
echo -e "export WNEAR_TOKEN_ID=$WNEAR_TOKEN_ID"
echo -e "export ONE_YOCTO=$ONE_YOCTO"
echo -e "export GAS=$GAS"
