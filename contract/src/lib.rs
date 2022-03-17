mod account;
mod account_asset;
mod account_farm;
mod account_view;
mod actions;
mod asset;
mod asset_config;
mod asset_farm;
mod asset_view;
mod big_decimal;
mod booster_staking;
mod config;
mod fungible_token;
mod legacy;
mod pool;
mod price_receiver;
mod prices;
mod storage;
mod storage_tracker;
mod upgrade;
mod utils;

pub use crate::account::*;
pub use crate::account_asset::*;
pub use crate::account_farm::*;
pub use crate::account_view::*;
pub use crate::actions::*;
pub use crate::asset::*;
pub use crate::asset_config::*;
pub use crate::asset_farm::*;
pub use crate::asset_view::*;
pub use crate::big_decimal::*;
pub use crate::booster_staking::*;
pub use crate::config::*;
pub use crate::fungible_token::*;
pub use crate::legacy::*;
pub use crate::pool::*;
pub use crate::price_receiver::*;
pub use crate::prices::*;
pub use crate::storage::*;
use crate::storage_tracker::*;
use crate::utils::*;

use common::*;

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, LookupMap, UnorderedMap, UnorderedSet};
use near_sdk::json_types::U128;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{
    assert_one_yocto, env, ext_contract, log, near_bindgen, AccountId, Balance, BorshStorageKey,
    Duration, Gas, PanicOnDefault, Promise, Timestamp,
};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Mutex;

#[derive(BorshSerialize, BorshStorageKey)]
enum StorageKey {
    Accounts,
    AccountAssets { account_id: AccountId },
    AccountFarms { account_id: AccountId },
    Storage,
    Assets,
    AssetFarms,
    InactiveAssetFarmRewards { farm_id: FarmId },
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
        config.assert_valid();
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
