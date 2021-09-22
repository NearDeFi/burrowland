use crate::*;

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct AccountDetailedView {
    pub supplied: Vec<(TokenAccountId, WrappedBalance)>,
    pub collateral: Vec<(TokenAccountId, WrappedBalance)>,
    pub borrowed: Vec<(TokenAccountId, WrappedBalance)>,
}

impl Contract {
    pub fn account_into_detailed_view(&self, account: Account) -> AccountDetailedView {
        AccountDetailedView {
            supplied: unordered_map_pagination(&account.supplied, None, None)
                .into_iter()
                .map(|(token_account_id, AccountAsset { shares })| {
                    let balance = self
                        .internal_unwrap_asset(&token_account_id)
                        .supplied
                        .shares_to_amount(shares, false);
                    (token_account_id, balance.into())
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
                        (token_account_id, balance.into())
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
                        (token_account_id, balance.into())
                    },
                )
                .collect(),
        }
    }
}
