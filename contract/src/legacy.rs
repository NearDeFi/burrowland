use crate::*;

/// V0 legacy version of Account structure, before staking of the burrow token was introduced.
#[derive(BorshSerialize, BorshDeserialize)]
pub struct AccountV0 {
    /// A copy of an account ID. Saves one storage_read when iterating on accounts.
    pub account_id: AccountId,
    /// A list of assets that are supplied by the account (but not used a collateral).
    /// It's not returned for account pagination.
    pub supplied: UnorderedMap<TokenId, VAccountAsset>,
    /// A list of collateral assets.
    pub collateral: Vec<CollateralAsset>,
    /// A list of borrowed assets.
    pub borrowed: Vec<BorrowedAsset>,
    /// Keeping track of data required for farms for this account.
    pub farms: UnorderedMap<FarmId, VAccountFarm>,
}

impl From<AccountV0> for Account {
    fn from(a: AccountV0) -> Self {
        let AccountV0 {
            account_id,
            supplied,
            collateral,
            borrowed,
            farms,
        } = a;
        Self {
            account_id,
            supplied,
            collateral,
            borrowed,
            farms,
            affected_farms: Default::default(),
            storage_tracker: Default::default(),
            booster_staking: None,
        }
    }
}
