use crate::*;

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct AssetView {
    pub token_id: TokenId,
    #[serde(with = "u128_dec_format")]
    pub balance: Balance,
    pub shares: Shares,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct AccountDetailedView {
    pub account_id: AccountId,
    /// A list of assets that are supplied by the account (but not used a collateral).
    pub supplied: Vec<AssetView>,
    /// A list of assets that are used as a collateral.
    pub collateral: Vec<AssetView>,
    /// A list of assets that are borrowed.
    pub borrowed: Vec<AssetView>,
    /// Account farms
    pub farms: Vec<AccountFarmView>,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct AccountFarmView {
    pub farm_id: FarmId,
    pub rewards: Vec<AccountFarmRewardView>,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct AccountFarmRewardView {
    pub asset_farm_reward: AssetFarmReward,
    #[serde(with = "u128_dec_format")]
    pub boosted_shares: Balance,
    #[serde(with = "u128_dec_format")]
    pub unclaimed_amount: Balance,
}

impl Contract {
    pub fn account_into_detailed_view(&self, account: Account) -> AccountDetailedView {
        let farms = account
            .farms
            .keys()
            .map(|farm_id| {
                let asset_farm = self.internal_unwrap_asset_farm(&farm_id);
                let (account_farm, new_rewards) =
                    self.internal_account_farm_claim(&account, &farm_id, &asset_farm);
                AccountFarmView {
                    farm_id,
                    rewards: account_farm
                        .rewards
                        .into_iter()
                        .zip(asset_farm.rewards.into_iter())
                        .map(
                            |(AccountFarmReward { boosted_shares, .. }, asset_farm_reward)| {
                                let unclaimed_amount = new_rewards
                                    .iter()
                                    .find(|(token_id, _)| token_id == &asset_farm_reward.token_id)
                                    .map(|(_, amount)| *amount)
                                    .unwrap_or(0);
                                AccountFarmRewardView {
                                    asset_farm_reward,
                                    boosted_shares,
                                    unclaimed_amount,
                                }
                            },
                        )
                        .collect(),
                }
            })
            .collect();
        AccountDetailedView {
            account_id: account.account_id,
            supplied: unordered_map_pagination(&account.supplied, None, None)
                .into_iter()
                .map(|(token_id, AccountAsset { shares })| {
                    let balance = self
                        .internal_unwrap_asset(&token_id)
                        .supplied
                        .shares_to_amount(shares, false);
                    AssetView {
                        token_id,
                        balance,
                        shares,
                    }
                })
                .collect(),
            collateral: account
                .collateral
                .into_iter()
                .map(|CollateralAsset { token_id, shares }| {
                    let balance = self
                        .internal_unwrap_asset(&token_id)
                        .supplied
                        .shares_to_amount(shares, false);
                    AssetView {
                        token_id,
                        balance,
                        shares,
                    }
                })
                .collect(),
            borrowed: account
                .borrowed
                .into_iter()
                .map(|BorrowedAsset { token_id, shares }| {
                    let balance = self
                        .internal_unwrap_asset(&token_id)
                        .borrowed
                        .shares_to_amount(shares, true);
                    AssetView {
                        token_id,
                        balance,
                        shares,
                    }
                })
                .collect(),
            farms,
        }
    }
}
