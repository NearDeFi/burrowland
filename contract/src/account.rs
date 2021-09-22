use crate::*;

#[derive(BorshSerialize, BorshDeserialize)]
pub struct Account {
    pub supplied: UnorderedMap<TokenAccountId, VAccountAsset>,
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
            supplied: UnorderedMap::new(StorageKey::AccountAssets {
                account_id: account_id.clone(),
            }),
            collateral: vec![],
            borrowed: vec![],
        }
    }

    pub fn increase_collateral(&mut self, token_account_id: &TokenAccountId, shares: Shares) {
        if let Some(collateral) = self
            .collateral
            .iter_mut()
            .find(|c| &c.token_account_id == token_account_id)
        {
            collateral.shares.0 += shares.0;
        } else {
            self.collateral.push(CollateralAsset {
                token_account_id: token_account_id.clone(),
                shares,
            })
        }
    }

    pub fn decrease_collateral(&mut self, token_account_id: &TokenAccountId, shares: Shares) {
        let index = self
            .collateral
            .iter()
            .position(|c| &c.token_account_id == token_account_id)
            .expect("Collateral not found");
        if let Some(new_balance) = self.collateral[index].shares.0.checked_sub(shares.0) {
            if new_balance > 0 {
                self.collateral[index].shares.0 = new_balance;
            } else {
                self.collateral.swap_remove(index);
            }
        } else {
            env::panic(b"Not enough collateral balance");
        }
    }

    pub fn increase_borrowed(&mut self, token_account_id: &TokenAccountId, shares: Shares) {
        if let Some(borrowed) = self
            .borrowed
            .iter_mut()
            .find(|c| &c.token_account_id == token_account_id)
        {
            borrowed.shares.0 += shares.0;
        } else {
            self.borrowed.push(BorrowedAsset {
                token_account_id: token_account_id.clone(),
                shares,
            })
        }
    }

    pub fn decrease_borrowed(&mut self, token_account_id: &TokenAccountId, shares: Shares) {
        let index = self
            .borrowed
            .iter()
            .position(|c| &c.token_account_id == token_account_id)
            .expect("Borrowed asset not found");
        if let Some(new_balance) = self.borrowed[index].shares.0.checked_sub(shares.0) {
            if new_balance > 0 {
                self.borrowed[index].shares.0 = new_balance;
            } else {
                self.borrowed.swap_remove(index);
            }
        } else {
            env::panic(b"Not enough borrowed balance");
        }
    }

    pub fn internal_unwrap_collateral(&mut self, token_account_id: &TokenAccountId) -> Shares {
        self.collateral
            .iter()
            .find(|c| &c.token_account_id == token_account_id)
            .expect("Collateral not found")
            .shares
    }

    pub fn internal_unwrap_borrowed(&mut self, token_account_id: &TokenAccountId) -> Shares {
        self.borrowed
            .iter()
            .find(|c| &c.token_account_id == token_account_id)
            .expect("Borrowed asset not found")
            .shares
    }
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct CollateralAsset {
    pub token_account_id: TokenAccountId,
    pub shares: Shares,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct BorrowedAsset {
    pub token_account_id: TokenAccountId,
    pub shares: Shares,
}

#[near_bindgen]
impl Contract {
    pub fn get_account(&self, account_id: ValidAccountId) -> Option<AccountDetailedView> {
        self.internal_get_account(account_id.as_ref())
            .map(|account| self.account_into_detailed_view(account))
    }
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
