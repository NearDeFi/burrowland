use crate::*;

#[near_bindgen]
impl Contract {
    /// A method to migrate a state during the contract upgrade.
    /// Can only be called after upgrade method.
    #[private]
    #[init(ignore_state)]
    pub fn migrate_state() -> Self {
        #[derive(BorshDeserialize, BorshSerialize)]
        pub struct OldConfig {
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
        }

        #[derive(BorshDeserialize)]
        pub struct OldContract {
            pub accounts: UnorderedMap<AccountId, VAccount>,
            pub storage: LookupMap<AccountId, VStorage>,
            pub assets: LookupMap<TokenId, VAsset>,
            pub asset_farms: LookupMap<FarmId, VAssetFarm>,
            pub asset_ids: UnorderedSet<TokenId>,
            pub config: LazyOption<OldConfig>,
        }

        let OldContract {
            accounts,
            storage,
            assets,
            asset_farms,
            asset_ids,
            config: old_config,
        } = env::state_read().expect("Failed to read old contract state");

        let OldConfig {
            oracle_account_id,
            owner_id,
            booster_token_id,
            booster_decimals,
            max_num_assets,
            maximum_recency_duration_sec,
            maximum_staleness_duration_sec,
        } = old_config.get().expect("Failed to read old config");

        let new_config = Config {
            oracle_account_id,
            owner_id,
            booster_token_id,
            booster_decimals,
            max_num_assets,
            maximum_recency_duration_sec,
            maximum_staleness_duration_sec,
            minimum_staking_duration_sec: 2678400,
            maximum_staking_duration_sec: 31536000,
            x_booster_multiplier_at_maximum_staking_duration: 40000,
        };

        Self {
            accounts,
            storage,
            assets,
            asset_farms,
            asset_ids,
            config: LazyOption::new(StorageKey::Config, Some(&new_config)),
        }
    }

    // TODO: Upgrade by owner.
}
