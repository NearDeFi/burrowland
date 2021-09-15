use crate::*;

#[derive(BorshSerialize, BorshDeserialize)]
pub struct AccountAsset {
    pub shares: Shares,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub enum VAccountAsset {
    Current(AccountAsset),
}

impl From<VAccountAsset> for AccountAsset {
    fn from(v: VAccountAsset) -> Self {
        match v {
            VAccountAsset::Current(c) => c,
        }
    }
}

impl From<AccountAsset> for VAccountAsset {
    fn from(c: AccountAsset) -> Self {
        VAccountAsset::Current(c)
    }
}

impl AccountAsset {
    pub fn new() -> Self {
        Self { shares: Shares(0) }
    }

    pub fn deposit_shares(&mut self, shares: Shares) {
        self.shares.0 += shares.0;
    }
}

impl Account {
    pub fn internal_unwrap_asset(&self, token_account_id: &TokenAccountId) -> AccountAsset {
        self.internal_get_asset(token_account_id)
            .expect("Asset not found")
    }

    pub fn internal_get_asset(&self, token_account_id: &TokenAccountId) -> Option<AccountAsset> {
        self.assets.get(token_account_id).map(|o| o.into())
    }

    pub fn internal_get_asset_or_default(
        &mut self,
        token_account_id: &TokenAccountId,
    ) -> AccountAsset {
        self.internal_get_asset(token_account_id)
            .unwrap_or_else(AccountAsset::new)
    }

    pub fn internal_set_asset(
        &mut self,
        token_account_id: &TokenAccountId,
        account_asset: AccountAsset,
    ) {
        self.assets.insert(token_account_id, &account_asset.into());
    }
}
