#!/bin/bash
set -e

MASTER_ACCOUNT=$1
TIME=$(date +%s)

cd "$(dirname $0)/.."

export NEAR_ENV=testnet
LG='\033[1;30m' # Arrows color (Dark gray)
TC='\033[0;33m' # Text color (Orange)
NC='\033[0m' # No Color

echo -e "$LG>>>>>>>>>>>>>>$TC Deploy an empty contract to fund main account $LG<<<<<<<<<<<<<<$NC"
echo -n "" > /tmp/empty
near dev-deploy -f /tmp/empty
TMP_ACCOUNT="$(cat neardev/dev-account)"

MAIN="${TIME}.${MASTER_ACCOUNT}"

echo -e "$LG>>>>>>>>>>>>>>$TC Creating main account: $MAIN $LG<<<<<<<<<<<<<<$NC"
near create-account $MAIN --masterAccount=$MASTER_ACCOUNT --initialBalance=0.01

echo -e "$LG>>>>>>>>>>>>>>$TC Funding main account: $MAIN $LG<<<<<<<<<<<<<<$NC"
near delete $TMP_ACCOUNT $MAIN

OWNER_ID="owner.$MAIN"
echo -e "$LG>>>>>>>>>>>>>>$TC Creating owner account: $OWNER_ID $LG<<<<<<<<<<<<<<$NC"
near create-account $OWNER_ID --masterAccount=$MAIN --initialBalance=130

BOOSTER_TOKEN_ID="token.$MAIN"
echo -e "$LG>>>>>>>>>>>>>>$TC Creating and deploying booster token: $BOOSTER_TOKEN_ID $LG<<<<<<<<<<<<<<$NC"
near create-account $BOOSTER_TOKEN_ID --masterAccount=$MAIN --initialBalance=3
near deploy $BOOSTER_TOKEN_ID res/fungible_token.wasm new '{
   "owner_id": "'$OWNER_ID'",
   "total_supply": "1000000000000000000000000000",
   "metadata": {
       "spec": "ft-1.0.0",
       "name": "Booster Token ('$TIME')",
       "symbol": "BOOSTER-'$TIME'",
       "decimals": 18
   }
}'

ORACLE_ID="priceoracle.testnet"

CONTRACT_ID="contract.$MAIN"

echo -e "$LG>>>>>>>>>>>>>>$TC Creating and deploying contract account: $CONTRACT_ID $LG<<<<<<<<<<<<<<$NC"
near create-account $CONTRACT_ID --masterAccount=$MAIN --initialBalance=10
near deploy $CONTRACT_ID res/burrowland.wasm new '{"config": {
  "oracle_account_id": "'$ORACLE_ID'",
  "owner_id": "'$OWNER_ID'",
  "booster_token_id": "'$BOOSTER_TOKEN_ID'",
  "booster_decimals": 18
}}'

echo -e "$LG>>>>>>>>>>>>>>$TC Booster token storage for contract $LG<<<<<<<<<<<<<<$NC"
near call $BOOSTER_TOKEN_ID --accountId=$CONTRACT_ID storage_deposit '' --amount=0.00125

echo -e "$LG>>>>>>>>>>>>>>$TC Preparing nETH $LG<<<<<<<<<<<<<<$NC"
NETH_TOKEN_ID="aurora"

near call $NETH_TOKEN_ID --accountId=$CONTRACT_ID storage_deposit '' --amount=0.0125
near call $NETH_TOKEN_ID --accountId=$OWNER_ID storage_deposit '' --amount=0.0125

echo -e "$LG>>>>>>>>>>>>>>$TC Preparing nDAI $LG<<<<<<<<<<<<<<$NC"
DAI_TOKEN_ID="dai.fakes.testnet"

near call $DAI_TOKEN_ID --accountId=$CONTRACT_ID storage_deposit '' --amount=0.00125
near call $DAI_TOKEN_ID --accountId=$OWNER_ID storage_deposit '' --amount=0.00125

echo -e "$LG>>>>>>>>>>>>>>$TC Preparing nUSDT $LG<<<<<<<<<<<<<<$NC"
USDT_TOKEN_ID="usdt.fakes.testnet"

near call $USDT_TOKEN_ID --accountId=$CONTRACT_ID storage_deposit '' --amount=0.00125
near call $USDT_TOKEN_ID --accountId=$OWNER_ID storage_deposit '' --amount=0.00125

echo -e "$LG>>>>>>>>>>>>>>$TC Preparing nUSDC $LG<<<<<<<<<<<<<<$NC"
USDC_TOKEN_ID="usdc.fakes.testnet"

near call $USDC_TOKEN_ID --accountId=$CONTRACT_ID storage_deposit '' --amount=0.00125
near call $USDC_TOKEN_ID --accountId=$OWNER_ID storage_deposit '' --amount=0.00125

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
#near call $CONTRACT_ID --accountId=$OWNER_ID add_asset '{
#  "token_id": "'$BOOSTER_TOKEN_ID'",
#  "asset_config": {
#    "reserve_ratio": 2500,
#    "target_utilization": 8000,
#    "target_utilization_rate": "1000000000008319516250272147",
#    "max_utilization_rate": "1000000000039724853136740579",
#    "volatility_ratio": 2000,
#    "extra_decimals": 0,
#    "can_deposit": true,
#    "can_withdraw": true,
#    "can_use_as_collateral": false,
#    "can_borrow": false
#  }
#}' --amount=$ONE_YOCTO --gas=$GAS


# nETH APR is 6%, to verify run ./scripts/apr_to_rate.py 5
# Volatility ratio is 60%, since it's somewhat liquid on NEAR
near call $CONTRACT_ID --accountId=$OWNER_ID add_asset '{
  "token_id": "'$NETH_TOKEN_ID'",
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
# USDT by default has 6 decimals, the config adds extra 12 decimals, to bring the total to 18
near call $CONTRACT_ID --accountId=$OWNER_ID add_asset '{
  "token_id": "'$USDT_TOKEN_ID'",
  "asset_config": {
    "reserve_ratio": 2500,
    "target_utilization": 8000,
    "target_utilization_rate": "1000000000002440418605283556",
    "max_utilization_rate": "1000000000039724853136740579",
    "volatility_ratio": 9500,
    "extra_decimals": 12,
    "can_deposit": true,
    "can_withdraw": true,
    "can_use_as_collateral": true,
    "can_borrow": true
  }
}' --amount=$ONE_YOCTO --gas=$GAS


# USDC APR is 8%, to verify run ./scripts/apr_to_rate.py 8
# Volatility ratio is 95%, since it's stable and liquid on NEAR
# USDC by default has 6 decimals, the config adds extra 12 decimals, to bring the total to 18
near call $CONTRACT_ID --accountId=$OWNER_ID add_asset '{
  "token_id": "'$USDC_TOKEN_ID'",
  "asset_config": {
    "reserve_ratio": 2500,
    "target_utilization": 8000,
    "target_utilization_rate": "1000000000002440418605283556",
    "max_utilization_rate": "1000000000039724853136740579",
    "volatility_ratio": 9500,
    "extra_decimals": 12,
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
# echo -e "$LG>>>>>>>>>>>>>>$TC * 12 nETH $NC"
echo -e "$LG>>>>>>>>>>>>>>$TC * 12000 DAI $NC"
echo -e "$LG>>>>>>>>>>>>>>$TC * 12000 USDT $NC"
echo -e "$LG>>>>>>>>>>>>>>$TC * 12000 USDC $NC"
echo -e "$LG>>>>>>>>>>>>>>$TC * 120 wNEAR (wrapped) $NC"
#near call $NETH_TOKEN_ID --accountId=$OWNER_ID mint '{
#  "account_id": "'$OWNER_ID'",
#  "amount": "12000000000000000000"
#}'
near call $DAI_TOKEN_ID --accountId=$OWNER_ID mint '{
  "account_id": "'$OWNER_ID'",
  "amount": "12000000000000000000000"
}'
near call $USDT_TOKEN_ID --accountId=$OWNER_ID mint '{
  "account_id": "'$OWNER_ID'",
  "amount": "12000000000"
}'
near call $USDC_TOKEN_ID --accountId=$OWNER_ID mint '{
  "account_id": "'$OWNER_ID'",
  "amount": "12000000000"
}'


near call $WNEAR_TOKEN_ID --accountId=$OWNER_ID near_deposit '{}' --amount=120

echo -e "$LG>>>>>>>>>>>>>>$TC Adding some reserves from the owner: $LG<<<<<<<<<<<<<<$NC"
# echo -e "$LG>>>>>>>>>>>>>>$TC * 20000 BOOSTER $NC"
# echo -e "$LG>>>>>>>>>>>>>>$TC * 2 wETH $NC"
echo -e "$LG>>>>>>>>>>>>>>$TC * 2000 DAI $NC"
echo -e "$LG>>>>>>>>>>>>>>$TC * 2000 USDT $NC"
echo -e "$LG>>>>>>>>>>>>>>$TC * 2000 USDC $NC"
echo -e "$LG>>>>>>>>>>>>>>$TC * 20 wNEAR $NC"
#near call $BOOSTER_TOKEN_ID --accountId=$OWNER_ID ft_transfer_call '{
#  "receiver_id": "'$CONTRACT_ID'",
#  "amount": "20000000000000000000000",
#  "msg": "\"DepositToReserve\""
#}' --amount=$ONE_YOCTO --gas=$GAS

near call $NETH_TOKEN_ID --accountId=$OWNER_ID ft_transfer_call '{
  "receiver_id": "'$CONTRACT_ID'",
  "amount": "1000000000000000000",
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

near call $USDC_TOKEN_ID --accountId=$OWNER_ID ft_transfer_call '{
  "receiver_id": "'$CONTRACT_ID'",
  "amount": "2000000000",
  "msg": "\"DepositToReserve\""
}' --amount=$ONE_YOCTO --gas=$GAS

near call $WNEAR_TOKEN_ID --accountId=$OWNER_ID ft_transfer_call '{
  "receiver_id": "'$CONTRACT_ID'",
  "amount": "20000000000000000000000000",
  "msg": "\"DepositToReserve\""
}' --amount=$ONE_YOCTO --gas=$GAS

echo -e "$LG>>>>>>>>>>>>>>$TC Registering the owner: $LG<<<<<<<<<<<<<<$NC"

near call $CONTRACT_ID --accountId=$OWNER_ID storage_deposit '' --amount=0.1

echo -e "$LG>>>>>>>>>>>>>>$TC Adding regular deposits from the owner: $LG<<<<<<<<<<<<<<$NC"
# echo -e "$LG>>>>>>>>>>>>>>$TC * 20000 BOOSTER $NC"
# echo -e "$LG>>>>>>>>>>>>>>$TC * 2 wETH $NC"
echo -e "$LG>>>>>>>>>>>>>>$TC * 100 DAI $NC"
echo -e "$LG>>>>>>>>>>>>>>$TC * 100 USDT $NC"
echo -e "$LG>>>>>>>>>>>>>>$TC * 100 USDC $NC"
echo -e "$LG>>>>>>>>>>>>>>$TC * 1 wNEAR $NC"

near call $DAI_TOKEN_ID --accountId=$OWNER_ID ft_transfer_call '{
  "receiver_id": "'$CONTRACT_ID'",
  "amount": "100000000000000000000",
  "msg": ""
}' --amount=$ONE_YOCTO --gas=$GAS

near call $USDT_TOKEN_ID --accountId=$OWNER_ID ft_transfer_call '{
  "receiver_id": "'$CONTRACT_ID'",
  "amount": "100000000",
  "msg": ""
}' --amount=$ONE_YOCTO --gas=$GAS

near call $USDC_TOKEN_ID --accountId=$OWNER_ID ft_transfer_call '{
  "receiver_id": "'$CONTRACT_ID'",
  "amount": "2000000000",
  "msg": ""
}' --amount=$ONE_YOCTO --gas=$GAS

near call $WNEAR_TOKEN_ID --accountId=$OWNER_ID ft_transfer_call '{
  "receiver_id": "'$CONTRACT_ID'",
  "amount": "1000000000000000000000000",
  "msg": ""
}' --amount=$ONE_YOCTO --gas=$GAS

echo -e "$LG>>>>>>>>>>>>>>$TC Dropping info to continue working from NEAR CLI: $LG<<<<<<<<<<<<<<$NC"
echo -e "export NEAR_ENV=testnet"
echo -e "export OWNER_ID=$OWNER_ID"
echo -e "export ORACLE_ID=$ORACLE_ID"
echo -e "export CONTRACT_ID=$CONTRACT_ID"
echo -e "export BOOSTER_TOKEN_ID=$BOOSTER_TOKEN_ID"
echo -e "export NETH_TOKEN_ID=$NETH_TOKEN_ID"
echo -e "export DAI_TOKEN_ID=$DAI_TOKEN_ID"
echo -e "export USDT_TOKEN_ID=$USDT_TOKEN_ID"
echo -e "export USDC_TOKEN_ID=$USDC_TOKEN_ID"
echo -e "export WNEAR_TOKEN_ID=$WNEAR_TOKEN_ID"
echo -e "export ONE_YOCTO=$ONE_YOCTO"
echo -e "export GAS=$GAS"
