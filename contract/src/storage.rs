use crate::*;
use near_contract_standards::storage_management::{
    StorageBalance, StorageBalanceBounds, StorageManagement,
};
use near_sdk::json_types::U128;
use near_sdk::StorageUsage;

/// 10000 bytes
const MIN_STORAGE_BALANCE: Balance = 10000u128 * env::STORAGE_PRICE_PER_BYTE;

#[derive(BorshSerialize, BorshDeserialize, Clone)]
#[borsh_init(init)]
pub struct Storage {
    pub storage_balance: Balance,
    pub used_bytes: StorageUsage,
    #[borsh_skip]
    pub initial_storage_usage: StorageUsage,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub enum VStorage {
    Current(Storage),
}

impl From<VStorage> for Storage {
    fn from(v: VStorage) -> Self {
        match v {
            VStorage::Current(c) => c,
        }
    }
}

impl From<Storage> for VStorage {
    fn from(c: Storage) -> Self {
        VStorage::Current(c)
    }
}

impl Storage {
    pub fn new() -> Self {
        Self {
            storage_balance: 0,
            used_bytes: 0,
            initial_storage_usage: env::storage_usage(),
        }
    }

    pub(crate) fn available_bytes(&self) -> StorageUsage {
        let covered_bytes = (self.storage_balance / env::storage_byte_cost()) as StorageUsage;
        covered_bytes - self.used_bytes
    }

    fn init(&mut self) {
        self.initial_storage_usage = env::storage_usage();
    }

    fn assert_storage_covered(&self) {
        let storage_balance_needed = Balance::from(self.used_bytes) * env::storage_byte_cost();
        assert!(
            storage_balance_needed <= self.storage_balance,
            "Not enough storage balance"
        );
    }
}

impl Contract {
    pub fn internal_get_storage(&self, account_id: &AccountId) -> Option<Storage> {
        self.storage.get(account_id).map(|o| o.into())
    }

    pub fn internal_unwrap_storage(&self, account_id: &AccountId) -> Storage {
        self.internal_get_storage(account_id)
            .expect("Storage for account is missing")
    }

    pub fn internal_set_storage(&mut self, account_id: &AccountId, mut storage: Storage) {
        let storage_usage = env::storage_usage();
        if storage_usage > storage.initial_storage_usage {
            let extra_bytes_used = storage_usage - storage.initial_storage_usage;
            storage.used_bytes += extra_bytes_used;
            storage.assert_storage_covered();
        } else {
            let bytes_released = storage.initial_storage_usage - storage_usage;
            assert!(
                storage.used_bytes >= bytes_released,
                "Internal storage accounting bug"
            );
            storage.used_bytes -= bytes_released;
        }
        self.storage.insert(account_id, &storage.into());
    }

    pub fn internal_storage_balance_of(&self, account_id: &AccountId) -> Option<StorageBalance> {
        self.internal_get_storage(account_id)
            .map(|storage| StorageBalance {
                total: storage.storage_balance.into(),
                available: U128(
                    storage.storage_balance
                        - std::cmp::max(
                            Balance::from(storage.used_bytes) * env::storage_byte_cost(),
                            self.storage_balance_bounds().min.0,
                        ),
                ),
            })
    }
}

#[near_bindgen]
impl StorageManagement for Contract {
    #[payable]
    fn storage_deposit(
        &mut self,
        account_id: Option<AccountId>,
        registration_only: Option<bool>,
    ) -> StorageBalance {
        let amount: Balance = env::attached_deposit();
        let account_id = account_id
            .map(|a| a.into())
            .unwrap_or_else(|| env::predecessor_account_id());
        let storage = self.internal_get_storage(&account_id);
        let registration_only = registration_only.unwrap_or(false);
        if let Some(mut storage) = storage {
            if registration_only && amount > 0 {
                Promise::new(env::predecessor_account_id()).transfer(amount);
            } else {
                storage.storage_balance += amount;
                self.internal_set_storage(&account_id, storage);
            }
        } else {
            let min_balance = self.storage_balance_bounds().min.0;
            if amount < min_balance {
                env::panic_str("The attached deposit is less than the mimimum storage balance");
            }

            let mut storage = Storage::new();
            if registration_only {
                let refund = amount - min_balance;
                if refund > 0 {
                    Promise::new(env::predecessor_account_id()).transfer(refund);
                }
                storage.storage_balance = min_balance;
            } else {
                storage.storage_balance = amount;
            }
            // Saving storage object copy into the persistent storage to account for used bytes.
            self.internal_set_storage(&account_id, storage.clone());
            self.internal_set_account(&account_id, Account::new(&account_id), storage);
        }
        self.internal_storage_balance_of(&account_id).unwrap()
    }

    #[payable]
    fn storage_withdraw(&mut self, amount: Option<U128>) -> StorageBalance {
        assert_one_yocto();
        let account_id = env::predecessor_account_id();
        if let Some(storage_balance) = self.internal_storage_balance_of(&account_id) {
            let amount = amount.unwrap_or(storage_balance.available).0;
            if amount > storage_balance.available.0 {
                env::panic_str("The amount is greater than the available storage balance");
            }
            if amount > 0 {
                let mut storage = self.internal_unwrap_storage(&account_id);
                storage.storage_balance -= amount;
                self.internal_set_storage(&account_id, storage);
                Promise::new(account_id.clone()).transfer(amount);
            }
            self.internal_storage_balance_of(&account_id).unwrap()
        } else {
            env::panic_str(&format!("The account {} is not registered", &account_id));
        }
    }

    #[allow(unused_variables)]
    #[payable]
    fn storage_unregister(&mut self, force: Option<bool>) -> bool {
        env::panic_str("The account can't be unregistered");
    }

    fn storage_balance_bounds(&self) -> StorageBalanceBounds {
        StorageBalanceBounds {
            min: U128(MIN_STORAGE_BALANCE),
            max: None,
        }
    }

    fn storage_balance_of(&self, account_id: AccountId) -> Option<StorageBalance> {
        self.internal_storage_balance_of(&account_id)
    }
}
