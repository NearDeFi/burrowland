#!/bin/bash
set -e

cd "$(dirname $0)/.."

near call $CONTRACT_ID --accountId=$ACCOUNT_ID --gas=$GAS --amount=0.1 storage_deposit '{}'

near call $USDT_TOKEN_ID --accountId=$ACCOUNT_ID --gas=$GAS --amount=$ONE_YOCTO ft_transfer_call '{
  "receiver_id": "'$CONTRACT_ID'",
  "amount": "5000000",
  "msg": ""
}'

near view $CONTRACT_ID get_account '{"account_id": "'$ACCOUNT_ID'"}'

near view $CONTRACT_ID get_asset '{"token_id": "'$USDT_TOKEN_ID'"}'

near call $CONTRACT_ID --accountId=$ACCOUNT_ID --gas=$GAS --amount=$ONE_YOCTO execute '{
  "actions": [
    {
      "IncreaseCollateral": {
        "token_id": "'$USDT_TOKEN_ID'"
      }
    }
  ]
}'

near view $CONTRACT_ID get_account '{"account_id": "'$ACCOUNT_ID'"}'

near call $ORACLE_ID --accountId=$ACCOUNT_ID --gas=$GAS --amount=$ONE_YOCTO oracle_call '{
  "receiver_id": "'$CONTRACT_ID'",
  "asset_ids": [
    "'$USDT_TOKEN_ID'",
    "'$DAI_TOKEN_ID'"
  ],
  "msg": "{\"Execute\": {\"actions\": [{\"Borrow\": {\"token_id\": \"'$DAI_TOKEN_ID'\", \"amount\": \"1000000000000000000\"}}]}}"
}'

near view $CONTRACT_ID get_account '{"account_id": "'$ACCOUNT_ID'"}'
sleep 1

near view $CONTRACT_ID get_account '{"account_id": "'$ACCOUNT_ID'"}'

near view $CONTRACT_ID get_asset '{"token_id": "'$DAI_TOKEN_ID'"}'

near call $CONTRACT_ID --accountId=$ACCOUNT_ID --gas=$GAS --amount=$ONE_YOCTO execute '{
  "actions": [
    {
      "Withdraw": {
        "token_id": "'$DAI_TOKEN_ID'"
      }
    }
  ]
}'

near view $DAI_TOKEN_ID ft_balance_of '{"account_id": "'$ACCOUNT_ID'"}'

near view $CONTRACT_ID get_account '{"account_id": "'$ACCOUNT_ID'"}'

near view $CONTRACT_ID get_asset '{"token_id": "'$DAI_TOKEN_ID'"}'

near call $DAI_TOKEN_ID --accountId=$ACCOUNT_ID --gas=$GAS --amount=$ONE_YOCTO ft_transfer_call '{
  "receiver_id": "'$CONTRACT_ID'",
  "amount": "5000000000000000000",
  "msg": "{\"Execute\": {\"actions\": [{\"Repay\": {\"token_id\": \"'$DAI_TOKEN_ID'\"}}]}}"
}'

near view $CONTRACT_ID get_account '{"account_id": "'$ACCOUNT_ID'"}'

near view $CONTRACT_ID get_asset '{"token_id": "'$DAI_TOKEN_ID'"}'

near call $CONTRACT_ID --accountId=$ACCOUNT_ID --gas=$GAS --amount=$ONE_YOCTO execute '{
  "actions": [
    {
      "DecreaseCollateral": {
        "token_id": "'$USDT_TOKEN_ID'"
      }
    }
  ]
}'

near view $CONTRACT_ID get_account '{"account_id": "'$ACCOUNT_ID'"}'
