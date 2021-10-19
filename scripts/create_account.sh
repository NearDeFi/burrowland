#!/bin/bash
set -e

cd "$(dirname $0)/.."

export NEAR_ENV=testnet
LG='\033[1;30m' # Arrows color (Dark gray)
TC='\033[0;33m' # Text color (Orange)
NC='\033[0m' # No Color

mv -f neardev/dev-account neardev/dev-account-old 2> /dev/null || true

echo -e "$LG>>>>>>>>>>>>>>$TC Deploy an empty contract to create a user's account $LG<<<<<<<<<<<<<<$NC"
echo -n "" > /tmp/empty
near dev-deploy -f /tmp/empty
ACCOUNT_ID="$(cat neardev/dev-account)"

mv -f neardev/dev-account-old neardev/dev-account 2> /dev/null || true

echo -e "$LG>>>>>>>>>>>>>>$TC Registering storage for the user: $LG<<<<<<<<<<<<<<$NC"
near call $BOOSTER_TOKEN_ID --accountId=$ACCOUNT_ID storage_deposit '' --amount=0.00125
near call $WETH_TOKEN_ID --accountId=$ACCOUNT_ID storage_deposit '' --amount=0.00125
near call $DAI_TOKEN_ID --accountId=$ACCOUNT_ID storage_deposit '' --amount=0.00125
near call $USDT_TOKEN_ID --accountId=$ACCOUNT_ID storage_deposit '' --amount=0.00125
near call $WNEAR_TOKEN_ID --accountId=$ACCOUNT_ID storage_deposit '' --amount=0.00125

echo -e "$LG>>>>>>>>>>>>>>$TC Minting tokens for the user: $LG<<<<<<<<<<<<<<$NC"
echo -e "$LG>>>>>>>>>>>>>>$TC * 100000 BOOSTER $NC"
echo -e "$LG>>>>>>>>>>>>>>$TC * 10 wETH $NC"
echo -e "$LG>>>>>>>>>>>>>>$TC * 10000 DAI $NC"
echo -e "$LG>>>>>>>>>>>>>>$TC * 10000 USDT $NC"
echo -e "$LG>>>>>>>>>>>>>>$TC * 100 wNEAR (wrapped) $NC"
near call $BOOSTER_TOKEN_ID --accountId=$ACCOUNT_ID mint '{
  "account_id": "'$ACCOUNT_ID'",
  "amount": "100000000000000000000000"
}'
near call $WETH_TOKEN_ID --accountId=$ACCOUNT_ID mint '{
  "account_id": "'$ACCOUNT_ID'",
  "amount": "10000000000000000000"
}'
near call $DAI_TOKEN_ID --accountId=$ACCOUNT_ID mint '{
  "account_id": "'$ACCOUNT_ID'",
  "amount": "10000000000000000000000"
}'
near call $USDT_TOKEN_ID --accountId=$ACCOUNT_ID mint '{
  "account_id": "'$ACCOUNT_ID'",
  "amount": "10000000000"
}'
near call $WNEAR_TOKEN_ID --accountId=$ACCOUNT_ID near_deposit '{}' --amount=100

echo -e "$LG>>>>>>>>>>>>>>$TC Dropping info to continue working from NEAR CLI: $LG<<<<<<<<<<<<<<$NC"
echo -e "export ACCOUNT_ID=$ACCOUNT_ID"
