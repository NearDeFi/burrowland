mod account;
mod account_asset;
mod actions;
mod asset;
mod asset_config;
mod big_decimal;
mod config;
mod pool;
mod price_receiver;
mod storage;
mod token_receiver;
mod utils;

use crate::account::*;
use crate::account_asset::*;
use crate::actions::*;
use crate::asset::*;
use crate::asset_config::*;
use crate::big_decimal::*;
use crate::config::*;
use crate::pool::*;
use crate::price_receiver::*;
use crate::storage::*;
use crate::token_receiver::*;
use crate::utils::*;

use common::*;

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, LookupMap, UnorderedMap};
use near_sdk::json_types::ValidAccountId;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{
    assert_one_yocto, env, ext_contract, near_bindgen, AccountId, Balance, BorshStorageKey, Gas,
    PanicOnDefault, Promise, Timestamp,
};

near_sdk::setup_alloc!();

#[derive(BorshSerialize, BorshStorageKey)]
enum StorageKey {
    Accounts,
    AccountAssets { account_id: AccountId },
    Storage,
    Assets,
    Config,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    pub accounts: UnorderedMap<AccountId, VAccount>,
    pub storage: LookupMap<AccountId, VStorage>,
    pub assets: UnorderedMap<TokenAccountId, VAsset>,
    pub config: LazyOption<Config>,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(config: Config) -> Self {
        Self {
            accounts: UnorderedMap::new(StorageKey::Accounts),
            storage: LookupMap::new(StorageKey::Storage),
            assets: UnorderedMap::new(StorageKey::Assets),
            config: LazyOption::new(StorageKey::Config, Some(&config)),
        }
    }
}
