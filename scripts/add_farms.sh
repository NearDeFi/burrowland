#!/bin/bash
set -e

cd "$(dirname $0)/.."

export NEAR_ENV=testnet
LG='\033[1;30m' # Arrows color (Dark gray)
TC='\033[0;33m' # Text color (Orange)
NC='\033[0m' # No Color

echo -e "$LG>>>>>>>>>>>>>>$TC Adding the farms $LG<<<<<<<<<<<<<<$NC"

for TOKEN_ID in $DAI_TOKEN_ID $USDT_TOKEN_ID $WETH_TOKEN_ID $WNEAR_TOKEN_ID
do

  echo -e "$LG>>>>>>>>>>>>>>$TC Adding farms for $TOKEN_ID $LG<<<<<<<<<<<<<<$NC"

  near call $CONTRACT_ID --accountId=$OWNER_ID add_asset_farm_reward '{
    "farm_id": {
      "Supplied": "'$TOKEN_ID'"
    },
    "reward_token_id": "'$BOOSTER_TOKEN_ID'",
    "new_reward_per_day": "100000000000000000000",
    "new_booster_log_base": "100000000000000000000",
    "reward_amount": "700000000000000000000"
  }' --amount=$ONE_YOCTO --gas=$GAS

    near call $CONTRACT_ID --accountId=$OWNER_ID add_asset_farm_reward '{
      "farm_id": {
        "Borrowed": "'$TOKEN_ID'"
      },
      "reward_token_id": "'$BOOSTER_TOKEN_ID'",
      "new_reward_per_day": "250000000000000000000",
      "new_booster_log_base": "100000000000000000000",
      "reward_amount": "1750000000000000000000"
    }' --amount=$ONE_YOCTO --gas=$GAS

    near call $CONTRACT_ID --accountId=$OWNER_ID add_asset_farm_reward '{
      "farm_id": {
        "Supplied": "'$TOKEN_ID'"
      },
      "reward_token_id": "'$WNEAR_TOKEN_ID'",
      "new_reward_per_day": "100000000000000000000000",
      "new_booster_log_base": "10000000000000000000",
      "reward_amount": "700000000000000000000000"
    }' --amount=$ONE_YOCTO --gas=$GAS

    near call $CONTRACT_ID --accountId=$OWNER_ID add_asset_farm_reward '{
      "farm_id": {
        "Borrowed": "'$TOKEN_ID'"
      },
      "reward_token_id": "'$WNEAR_TOKEN_ID'",
      "new_reward_per_day": "250000000000000000000000",
      "new_booster_log_base": "10000000000000000000",
      "reward_amount": "1750000000000000000000000"
    }' --amount=$ONE_YOCTO --gas=$GAS

done

near view $CONTRACT_ID get_asset_farms_paged
