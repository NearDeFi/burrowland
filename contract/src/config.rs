use crate::*;

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Config {
    pub oracle_account_id: ValidAccountId,

    pub owner_id: ValidAccountId,

    pub booster_token_id: TokenId,

    pub booster_decimals: u8,
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
            self.internal_config().owner_id.as_ref(),
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
        self.config.set(&config);
    }

    #[payable]
    pub fn debug_nuke_state(&mut self) {
        assert_one_yocto();
        self.assert_owner();
        for token_id in self.asset_ids.to_vec() {
            self.assets.remove(&token_id);
        }
        self.asset_ids.clear();
        for account in self.accounts.values() {
            let mut account: Account = account.into();
            self.storage.remove(&account.account_id);
            account.supplied.clear();
        }
        self.accounts.clear();
    }

    /// Adds an asset with a given token_id and a given asset_config.
    /// - Panics if the asset config is invalid.
    /// - Panics if an asset with the given token_id already exists.
    /// - Requires one yoctoNEAR.
    /// - Requires to be called by the contract owner.
    #[payable]
    pub fn add_asset(&mut self, token_id: ValidAccountId, asset_config: AssetConfig) {
        assert_one_yocto();
        asset_config.assert_valid();
        self.assert_owner();
        assert!(self.asset_ids.insert(token_id.as_ref()));
        self.internal_set_asset(
            token_id.as_ref(),
            Asset::new(env::block_timestamp(), asset_config),
        )
    }

    /// Updates the asset config for the asset with the a given token_id.
    /// - Panics if the asset config is invalid.
    /// - Panics if an asset with the given token_id doesn't exist.
    /// - Requires one yoctoNEAR.
    /// - Requires to be called by the contract owner.
    #[payable]
    pub fn update_asset(&mut self, token_id: ValidAccountId, asset_config: AssetConfig) {
        assert_one_yocto();
        asset_config.assert_valid();
        self.assert_owner();
        let mut asset = self.internal_unwrap_asset(token_id.as_ref());
        asset.config = asset_config;
        self.internal_set_asset(token_id.as_ref(), asset);
    }
}
