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
            pub oracle_account_id: AccountId,
            pub owner_id: AccountId,
            pub booster_token_id: TokenId,
            pub booster_decimals: u8,
            pub max_num_assets: u32,
            pub maximum_recency_duration_sec: DurationSec,
            pub maximum_staleness_duration_sec: DurationSec,
            pub minimum_staking_duration_sec: DurationSec,
            pub maximum_staking_duration_sec: DurationSec,
            pub x_booster_multiplier_at_maximum_staking_duration: u32,
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
            minimum_staking_duration_sec,
            maximum_staking_duration_sec,
            x_booster_multiplier_at_maximum_staking_duration,
        } = old_config.get().expect("Failed to read old config");

        let new_config = Config {
            oracle_account_id,
            owner_id,
            booster_token_id,
            booster_decimals,
            max_num_assets,
            maximum_recency_duration_sec,
            maximum_staleness_duration_sec,
            minimum_staking_duration_sec,
            maximum_staking_duration_sec,
            x_booster_multiplier_at_maximum_staking_duration,
            force_closing_enabled: true,
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

    /// Returns semver of this contract.
    pub fn get_version(&self) -> String {
        env!("CARGO_PKG_VERSION").to_string()
    }
}

mod upgrade {
    use near_sdk::{require, Gas};

    use super::*;
    use near_sys as sys;

    const GAS_TO_COMPLETE_UPGRADE_CALL: Gas = Gas(Gas::ONE_TERA.0 * 10);
    const GAS_FOR_GET_CONFIG_CALL: Gas = Gas(Gas::ONE_TERA.0 * 5);
    const MIN_GAS_FOR_MIGRATE_STATE_CALL: Gas = Gas(Gas::ONE_TERA.0 * 10);

    /// Self upgrade and call migrate, optimizes gas by not loading into memory the code.
    /// Takes as input non serialized set of bytes of the code.
    #[no_mangle]
    pub extern "C" fn upgrade() {
        env::setup_panic_hook();
        let contract: Contract = env::state_read().expect("ERR_CONTRACT_IS_NOT_INITIALIZED");
        contract.assert_owner();
        let current_account_id = env::current_account_id().as_bytes().to_vec();
        let migrate_method_name = b"migrate_state".to_vec();
        let get_config_method_name = b"get_config".to_vec();
        let empty_args = b"{}".to_vec();
        unsafe {
            sys::input(0);
            let promise_id = sys::promise_batch_create(
                current_account_id.len() as _,
                current_account_id.as_ptr() as _,
            );
            sys::promise_batch_action_deploy_contract(promise_id, u64::MAX as _, 0);
            // Gas required to complete this call.
            let required_gas =
                env::used_gas() + GAS_TO_COMPLETE_UPGRADE_CALL + GAS_FOR_GET_CONFIG_CALL;
            require!(
                env::prepaid_gas() >= required_gas + MIN_GAS_FOR_MIGRATE_STATE_CALL,
                "Not enough gas to complete state migration"
            );
            let migrate_state_attached_gas = env::prepaid_gas() - required_gas;
            // Scheduling state migration.
            sys::promise_batch_action_function_call(
                promise_id,
                migrate_method_name.len() as _,
                migrate_method_name.as_ptr() as _,
                empty_args.len() as _,
                empty_args.as_ptr() as _,
                0 as _,
                migrate_state_attached_gas.0,
            );
            // Scheduling to return config after the migration is completed.
            sys::promise_batch_action_function_call(
                promise_id,
                get_config_method_name.len() as _,
                get_config_method_name.as_ptr() as _,
                empty_args.len() as _,
                empty_args.as_ptr() as _,
                0 as _,
                GAS_FOR_GET_CONFIG_CALL.0,
            );
            sys::promise_return(promise_id);
        }
    }
}
