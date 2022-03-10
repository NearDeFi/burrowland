use crate::*;

pub const MIN_BOOSTER_MULTIPLIER: u32 = 10000;

/// Contract config
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Config {
    /// The account ID of the oracle contract
    pub oracle_account_id: AccountId,

    /// The account ID of the contract owner that allows to modify config, assets and use reserves.
    pub owner_id: AccountId,

    /// The account ID of the booster token contract.
    pub booster_token_id: TokenId,

    /// The number of decimals of the booster fungible token.
    pub booster_decimals: u8,

    /// The total number of different assets
    pub max_num_assets: u32,

    /// The maximum number of seconds expected from the oracle price call.
    pub maximum_recency_duration_sec: DurationSec,

    /// Maximum staleness duration of the price data timestamp.
    /// Because NEAR protocol doesn't implement the gas auction right now, the only reason to
    /// delay the price updates are due to the shard congestion.
    /// This parameter can be updated in the future by the owner.
    pub maximum_staleness_duration_sec: DurationSec,

    /// The minimum duration to stake booster token in seconds.
    pub minimum_staking_duration_sec: DurationSec,

    /// The maximum duration to stake booster token in seconds.
    pub maximum_staking_duration_sec: DurationSec,

    /// The rate of xBooster for the amount of Booster given for the maximum staking duration.
    /// Assuming the 100% multiplier at the minimum staking duration. Should be no less than 100%.
    /// E.g. 20000 means 200% multiplier (or 2X).
    pub x_booster_multiplier_at_maximum_staking_duration: u32,

    /// Whether an account with bad debt can be liquidated using reserves.
    /// The account should have borrowed sum larger than the collateral sum.
    pub force_closing_enabled: bool,
}

impl Config {
    pub fn assert_valid(&self) {
        assert!(
            self.minimum_staking_duration_sec < self.maximum_staking_duration_sec,
            "The maximum staking duration must be greater than minimum staking duration"
        );
        assert!(
            self.x_booster_multiplier_at_maximum_staking_duration >= MIN_BOOSTER_MULTIPLIER,
            "xBooster multiplier should be no less than 100%"
        );
    }
}

impl Contract {
    pub fn internal_config(&self) -> Config {
        self.config.get().unwrap()
    }

    pub fn get_oracle_account_id(&self) -> AccountId {
        self.internal_config().oracle_account_id.into()
    }

    pub fn assert_owner(&self) {
        assert_eq!(
            &env::predecessor_account_id(),
            &self.internal_config().owner_id,
            "Not an owner"
        );
    }
}

#[near_bindgen]
impl Contract {
    /// Returns the current config.
    pub fn get_config(&self) -> Config {
        self.internal_config()
    }

    /// Updates the current config.
    /// - Requires one yoctoNEAR.
    /// - Requires to be called by the contract owner.
    #[payable]
    pub fn update_config(&mut self, config: Config) {
        assert_one_yocto();
        self.assert_owner();
        config.assert_valid();
        self.config.set(&config);
    }

    /// Adds an asset with a given token_id and a given asset_config.
    /// - Panics if the asset config is invalid.
    /// - Panics if an asset with the given token_id already exists.
    /// - Requires one yoctoNEAR.
    /// - Requires to be called by the contract owner.
    #[payable]
    pub fn add_asset(&mut self, token_id: AccountId, asset_config: AssetConfig) {
        assert_one_yocto();
        asset_config.assert_valid();
        self.assert_owner();
        assert!(self.asset_ids.insert(&token_id));
        self.internal_set_asset(&token_id, Asset::new(env::block_timestamp(), asset_config))
    }

    /// Updates the asset config for the asset with the a given token_id.
    /// - Panics if the asset config is invalid.
    /// - Panics if an asset with the given token_id doesn't exist.
    /// - Requires one yoctoNEAR.
    /// - Requires to be called by the contract owner.
    #[payable]
    pub fn update_asset(&mut self, token_id: AccountId, asset_config: AssetConfig) {
        assert_one_yocto();
        asset_config.assert_valid();
        self.assert_owner();
        let mut asset = self.internal_unwrap_asset(&token_id);
        if asset.config.extra_decimals != asset_config.extra_decimals {
            assert!(
                asset.borrowed.balance == 0 && asset.supplied.balance == 0 && asset.reserved == 0,
                "Can't change extra decimals if any of the balances are not 0"
            );
        }
        asset.config = asset_config;
        self.internal_set_asset(&token_id, asset);
    }

    /// Adds an asset farm reward for the farm with a given farm_id. The reward is of token_id with
    /// the new reward per day amount and a new booster log base. The extra amount of reward is
    /// taken from the asset reserved balance.
    /// - The booster log base should include decimals of the token for better precision of the log
    ///    base. For example, if token decimals is `6` the log base of `10_500_000` will be `10.5`.
    /// - Panics if the farm asset token_id doesn't exists.
    /// - Panics if an asset with the given token_id doesn't exists.
    /// - Panics if an asset with the given token_id doesn't have enough reserved balance.
    /// - Requires one yoctoNEAR.
    /// - Requires to be called by the contract owner.
    #[payable]
    pub fn add_asset_farm_reward(
        &mut self,
        farm_id: FarmId,
        reward_token_id: AccountId,
        new_reward_per_day: U128,
        new_booster_log_base: U128,
        reward_amount: U128,
    ) {
        assert_one_yocto();
        self.assert_owner();
        assert!(self.assets.contains_key(farm_id.get_token_id()));
        let reward_token_id: TokenId = reward_token_id.into();
        let mut reward_asset = self.internal_unwrap_asset(&reward_token_id);
        assert!(
            reward_asset.reserved >= reward_amount.0
                && reward_asset.available_amount() >= reward_amount.0,
            "Not enough reserved reward balance"
        );
        reward_asset.reserved -= reward_amount.0;
        self.internal_set_asset(&reward_token_id, reward_asset);
        let mut asset_farm = self
            .internal_get_asset_farm(&farm_id, false)
            .unwrap_or_else(|| AssetFarm {
                block_timestamp: env::block_timestamp(),
                rewards: HashMap::new(),
                inactive_rewards: LookupMap::new(StorageKey::InactiveAssetFarmRewards {
                    farm_id: farm_id.clone(),
                }),
            });

        let mut asset_farm_reward = asset_farm
            .rewards
            .remove(&reward_token_id)
            .or_else(|| asset_farm.internal_remove_inactive_asset_farm_reward(&reward_token_id))
            .unwrap_or_default();
        asset_farm_reward.reward_per_day = new_reward_per_day.into();
        asset_farm_reward.booster_log_base = new_booster_log_base.into();
        asset_farm_reward.remaining_rewards += reward_amount.0;
        asset_farm
            .rewards
            .insert(reward_token_id, asset_farm_reward);
        self.internal_set_asset_farm(&farm_id, asset_farm);
    }
}
