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
export ACCOUNT_ID="$(cat neardev/dev-account)"

mv -f neardev/dev-account-old neardev/dev-account 2> /dev/null || true

scripts/mint_to_account.sh

echo -e "$LG>>>>>>>>>>>>>>$TC Dropping info to continue working from NEAR CLI: $LG<<<<<<<<<<<<<<$NC"
echo -e "export ACCOUNT_ID=$ACCOUNT_ID"
