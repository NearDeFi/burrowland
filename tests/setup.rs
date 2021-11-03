use near_contract_standards::fungible_token::metadata::{FungibleTokenMetadata, FT_METADATA_SPEC};
use near_sdk::json_types::U128;
use near_sdk::serde_json::json;
use near_sdk::{env, Balance, Gas, Timestamp};
use near_sdk_sim::runtime::GenesisConfig;
use near_sdk_sim::{deploy, init_simulator, to_yocto, ContractAccount, UserAccount};
use std::convert::TryInto;

use contract::{Config, ContractContract as BurrowlandContract};
use test_oracle::ContractContract as OracleContract;

near_sdk_sim::lazy_static_include::lazy_static_include_bytes! {
    BURROWLAND_WASM_BYTES => "res/burrowland.wasm",
    TEST_ORACLE_WASM_BYTES => "res/test_oracle.wasm",

    FUNGIBLE_TOKEN_WASM_BYTES => "res/fungible_token.wasm",
}

pub const NEAR: &str = "near";
pub const ORACLE_ID: &str = "oracle.near";
pub const BURROWLAND_ID: &str = "burrowland.near";
pub const BOOSTER_TOKEN_ID: &str = "token.burrowland.near";
pub const OWNER_ID: &str = "owner.near";

pub const TOKENS_SUFFIX: &str = "tokens.near";

pub const BASE_GAS: Gas = 5_000_000_000_000;
pub const DEFAULT_GAS: Gas = 3 * BASE_GAS;
pub const BOOSTER_TOKEN_DECIMALS: u8 = 18;
pub const BOOSTER_TOKEN_TOTAL_SUPPLY: Balance =
    1_000_000_000 * 10u128.pow(BOOSTER_TOKEN_DECIMALS as _);

pub struct Env {
    pub root: UserAccount,
    pub near: UserAccount,
    pub owner: UserAccount,
    pub oracle: ContractAccount<OracleContract>,
    pub contract: ContractAccount<BurrowlandContract>,
    pub booster_token: UserAccount,
    pub tokens: Vec<UserAccount>,

    pub users: Vec<UserAccount>,
}

pub fn storage_deposit(user: &UserAccount, token_account_id: &str, account_id: &str) {
    user.call(
        token_account_id.to_string(),
        "storage_deposit",
        &json!({
            "account_id": account_id.to_string()
        })
        .to_string()
        .into_bytes(),
        DEFAULT_GAS,
        125 * env::STORAGE_PRICE_PER_BYTE, // attached deposit
    )
    .assert_success();
}

pub fn to_nano(timestamp: u32) -> Timestamp {
    Timestamp::from(timestamp) * 10u64.pow(9)
}

impl Env {
    pub fn init() -> Self {
        let mut genesis_config = GenesisConfig::default();
        genesis_config.block_prod_time = 0;
        let root = init_simulator(Some(genesis_config));
        let near = root.create_user(NEAR.to_string(), to_yocto("1000000"));
        let owner = near.create_user(OWNER_ID.to_string(), to_yocto("10000"));

        let oracle = deploy!(
            contract: OracleContract,
            contract_id: ORACLE_ID.to_string(),
            bytes: &TEST_ORACLE_WASM_BYTES,
            signer_account: near,
            deposit: to_yocto("10")
        );

        let contract = deploy!(
            contract: BurrowlandContract,
            contract_id: BURROWLAND_ID.to_string(),
            bytes: &BURROWLAND_WASM_BYTES,
            signer_account: near,
            deposit: to_yocto("20"),
            gas: DEFAULT_GAS,
            init_method: new(
                Config {
                    oracle_account_id: ORACLE_ID.to_string().try_into().unwrap(),
                    owner_id: owner.valid_account_id(),
                    booster_token_id: BOOSTER_TOKEN_ID.to_string(),
                    booster_decimals: BOOSTER_TOKEN_DECIMALS,
                }
            )
        );

        let booster_token = contract.user_account.deploy_and_init(
            &FUNGIBLE_TOKEN_WASM_BYTES,
            BOOSTER_TOKEN_ID.to_string(),
            "new",
            &json!({
                "owner_id": owner.valid_account_id(),
                "total_supply": U128::from(BOOSTER_TOKEN_TOTAL_SUPPLY),
                "metadata": FungibleTokenMetadata {
                    spec: FT_METADATA_SPEC.to_string(),
                    name: "Booster Token".to_string(),
                    symbol: "BOOSTER".to_string(),
                    icon: None,
                    reference: None,
                    reference_hash: None,
                    decimals: BOOSTER_TOKEN_DECIMALS,
                }
            })
            .to_string()
            .into_bytes(),
            to_yocto("10"),
            DEFAULT_GAS,
        );

        storage_deposit(&owner, BOOSTER_TOKEN_ID, BURROWLAND_ID);

        Self {
            root,
            near,
            owner,
            contract,
            oracle,
            booster_token,
            tokens: vec![],
            users: vec![],
        }
    }
}
