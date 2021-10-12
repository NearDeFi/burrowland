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
    /// Represents the current farming booster.
    pub farming_booster: String,
}

impl Contract {
    pub fn account_into_detailed_view(&self, account: Account) -> AccountDetailedView {
        AccountDetailedView {
            farming_booster: self.internal_compute_account_booster(&account).to_string(),
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
        }
    }
}
