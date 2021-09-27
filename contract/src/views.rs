use crate::*;

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct AssetView {
    pub token_account_id: TokenAccountId,
    #[serde(with = "u128_dec_format")]
    pub balance: Balance,
    pub shares: Shares,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct AccountDetailedView {
    pub account_id: AccountId,
    pub supplied: Vec<AssetView>,
    pub collateral: Vec<AssetView>,
    pub borrowed: Vec<AssetView>,
}

impl Contract {
    pub fn account_into_detailed_view(&self, account: Account) -> AccountDetailedView {
        AccountDetailedView {
            account_id: account.account_id,
            supplied: unordered_map_pagination(&account.supplied, None, None)
                .into_iter()
                .map(|(token_account_id, AccountAsset { shares })| {
                    let balance = self
                        .internal_unwrap_asset(&token_account_id)
                        .supplied
                        .shares_to_amount(shares, false);
                    AssetView {
                        token_account_id,
                        balance,
                        shares,
                    }
                })
                .collect(),
            collateral: account
                .collateral
                .into_iter()
                .map(
                    |CollateralAsset {
                         token_account_id,
                         shares,
                     }| {
                        let balance = self
                            .internal_unwrap_asset(&token_account_id)
                            .supplied
                            .shares_to_amount(shares, false);
                        AssetView {
                            token_account_id,
                            balance,
                            shares,
                        }
                    },
                )
                .collect(),
            borrowed: account
                .borrowed
                .into_iter()
                .map(
                    |BorrowedAsset {
                         token_account_id,
                         shares,
                     }| {
                        let balance = self
                            .internal_unwrap_asset(&token_account_id)
                            .borrowed
                            .shares_to_amount(shares, true);
                        AssetView {
                            token_account_id,
                            balance,
                            shares,
                        }
                    },
                )
                .collect(),
        }
    }
}
