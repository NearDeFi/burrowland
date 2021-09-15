use crate::*;
use near_sdk::StorageUsage;

#[derive(BorshSerialize, BorshDeserialize)]
pub struct Storage {
    pub storage_balance: Balance,
    pub used_bytes: StorageUsage,
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
        }
    }
}

impl Contract {
    pub fn internal_get_storage(&self, account_id: &AccountId) -> Option<Storage> {
        self.storage.get(account_id).map(|o| o.into())
    }

    pub fn internal_set_storage(&mut self, account_id: &AccountId, storage: Storage) {
        self.storage.insert(account_id, &storage.into());
    }
}
