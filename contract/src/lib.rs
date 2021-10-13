mod account;
mod account_asset;
mod account_farm;
mod actions;
mod asset;
mod asset_config;
mod asset_farm;
mod big_decimal;
mod config;
mod fungible_token;
mod pool;
mod price_receiver;
mod prices;
mod storage;
mod utils;
mod views;

use crate::account::*;
use crate::account_asset::*;
use crate::account_farm::*;
use crate::actions::*;
use crate::asset::*;
use crate::asset_config::*;
use crate::asset_farm::*;
use crate::big_decimal::*;
use crate::config::*;
use crate::pool::*;
use crate::prices::*;
use crate::storage::*;
use crate::utils::*;
use crate::views::*;

use common::*;

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, LookupMap, UnorderedMap, UnorderedSet};
use near_sdk::json_types::{ValidAccountId, WrappedBalance};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{
    assert_one_yocto, env, ext_contract, log, near_bindgen, AccountId, Balance, BorshStorageKey,
    Duration, Gas, PanicOnDefault, Promise, Timestamp,
};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Mutex;

near_sdk::setup_alloc!();

#[derive(BorshSerialize, BorshStorageKey)]
enum StorageKey {
    Accounts,
    AccountAssets { account_id: AccountId },
    AccountFarms { account_id: AccountId },
    Storage,
    Assets,
    AssetFarms,
    AssetIds,
    Config,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    pub accounts: UnorderedMap<AccountId, VAccount>,
    pub storage: LookupMap<AccountId, VStorage>,
    pub assets: LookupMap<TokenId, VAsset>,
    pub asset_farms: LookupMap<FarmId, VAssetFarm>,
    pub asset_ids: UnorderedSet<TokenId>,
    pub config: LazyOption<Config>,
}

#[near_bindgen]
impl Contract {
    /// Initializes the contract with the given config. Needs to be called once.
    #[init]
    pub fn new(config: Config) -> Self {
        Self {
            accounts: UnorderedMap::new(StorageKey::Accounts),
            storage: LookupMap::new(StorageKey::Storage),
            assets: LookupMap::new(StorageKey::Assets),
            asset_farms: LookupMap::new(StorageKey::AssetFarms),
            asset_ids: UnorderedSet::new(StorageKey::AssetIds),
            config: LazyOption::new(StorageKey::Config, Some(&config)),
        }
    }
}
