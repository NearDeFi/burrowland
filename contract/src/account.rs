use crate::*;

#[derive(BorshSerialize, BorshDeserialize)]
pub struct Account {
    pub assets: UnorderedMap<TokenAccountId, VAccountAsset>,
    pub collateral: Vec<CollateralAsset>,
    pub borrowed: Vec<BorrowedAsset>,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub enum VAccount {
    Current(Account),
}

impl From<VAccount> for Account {
    fn from(v: VAccount) -> Self {
        match v {
            VAccount::Current(c) => c,
        }
    }
}

impl From<Account> for VAccount {
    fn from(c: Account) -> Self {
        VAccount::Current(c)
    }
}

impl Account {
    pub fn new(account_id: &AccountId) -> Self {
        Self {
            assets: UnorderedMap::new(StorageKey::AccountAssets {
                account_id: account_id.clone(),
            }),
            collateral: vec![],
            borrowed: vec![],
        }
    }

    pub fn deposit_shares(&mut self, token_account_id: &TokenAccountId, shares: Shares) {
        let mut account_asset = self.internal_get_asset_or_default(token_account_id);
        account_asset.deposit_shares(shares);
        self.internal_set_asset(&token_account_id, account_asset);
    }
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct AccountAsset {
    pub shares: Shares,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct CollateralAsset {
    pub asset_id: TokenAccountId,
    pub shares: Balance,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct BorrowedAsset {
    pub asset_id: TokenAccountId,
    pub shares: Balance,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct AccountView {
    pub num_assets: u32,
    pub collateral: Vec<CollateralAssetView>,
    pub borrowed: Vec<BorrowedAssetView>,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct CollateralAssetView {
    pub asset_id: TokenAccountId,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct BorrowedAssetView {
    pub asset_id: TokenAccountId,
}

impl Contract {
    pub fn internal_get_account(&self, account_id: &AccountId) -> Option<Account> {
        self.accounts.get(account_id).map(|o| o.into())
    }

    pub fn internal_unwrap_account(&self, account_id: &AccountId) -> Account {
        self.internal_get_account(account_id)
            .expect("Account is not registered")
    }

    pub fn internal_set_account(&mut self, account_id: &AccountId, account: Account) {
        self.accounts.insert(account_id, &account.into());
    }
}
