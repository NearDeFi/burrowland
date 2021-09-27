use crate::*;

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Config {
    pub oracle_account_id: ValidAccountId,

    pub owner_id: ValidAccountId,
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
    pub fn get_config(&self) -> Config {
        self.internal_config()
    }

    #[payable]
    pub fn update_config(&mut self, config: Config) {
        assert_one_yocto();
        self.assert_owner();
        self.config.set(&config);
    }

    #[payable]
    pub fn add_asset(&mut self, token_account_id: ValidAccountId, asset_config: AssetConfig) {
        assert_one_yocto();
        asset_config.assert_valid();
        self.assert_owner();
        assert!(self.asset_ids.insert(token_account_id.as_ref()));
        self.internal_set_asset(
            token_account_id.as_ref(),
            Asset::new(env::block_timestamp(), asset_config),
        )
    }
    #[payable]
    pub fn update_asset(&mut self, token_account_id: ValidAccountId, asset_config: AssetConfig) {
        assert_one_yocto();
        asset_config.assert_valid();
        self.assert_owner();
        let mut asset = self.internal_unwrap_asset(token_account_id.as_ref());
        asset.config = asset_config;
        self.internal_set_asset(token_account_id.as_ref(), asset);
    }
}
