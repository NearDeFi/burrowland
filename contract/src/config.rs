use crate::*;

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Config {
    pub oracle_account_id: ValidAccountId,
}

impl Contract {
    pub fn internal_config(&self) -> Config {
        self.config.get().unwrap()
    }

    pub fn get_oracle_account_id(&self) -> AccountId {
        self.internal_config().oracle_account_id.into()
    }
}
