# The list of APIs that are provided by the contract

Notes:
- `u128_dec_format`, `WrappedBalance`, `Shares` means the value is passed as a decimal string representation. E.g. `1` serialized as `"1"`
- `BigDecimal` is serialized as floating string representation. E.g. `1.5` serialized as `"1.5"`
- `u64` means the value is passed as an integer.
- `Option<_>` means the value can be omitted, or provided as `null`.
- Rust enums are serialized using JSON objects. E.g. `FarmId::Supplied("token.near")` is serialized as `{"Supplied": "token.near"}`
- `HashMap<_, _>` is serialized using JSON objects.

```rust
trait Contract {
    /// Initializes the contract with the given config. Needs to be called once.
    #[init]
    fn new(config: Config) -> Self;

    /// Returns detailed information about an account for a given account_id.
    /// The information includes all supplied assets, collateral and borrowed.
    /// Each asset includes the current balance and the number of shares.
    fn get_account(&self, account_id: ValidAccountId) -> Option<AccountDetailedView>;

    /// Returns limited account information for accounts from a given index up to a given limit.
    /// The information includes number of shares for collateral and borrowed assets.
    /// This method can be used to iterate on the accounts for liquidation.
    fn get_accounts_paged(&self, from_index: Option<u64>, limit: Option<u64>) -> Vec<Account>;

    /// Executes a given list actions on behalf of the predecessor account.
    /// - Requires one yoctoNEAR.
    #[payable]
    fn execute(&mut self, actions: Vec<Action>);

    /// Returns a detailed view asset for a given token_id.
    /// The detailed view includes current APR and corresponding farms.
    fn get_asset(&self, token_id: ValidAccountId) -> Option<AssetDetailedView>;

    /// Returns an list of detailed view assets a given list of token_id.
    /// Only returns existing assets.
    fn get_assets(&self, token_ids: Vec<ValidAccountId>) -> Vec<AssetDetailedView>;

    /// Returns a list of pairs (token_id, asset) for assets from a given index up to a given limit.
    fn get_assets_paged(
        &self,
        from_index: Option<u64>,
        limit: Option<u64>,
    ) -> Vec<(TokenId, Asset)>;

    /// Returns a list of detailed view assets from a given index up to a given limit.
    fn get_assets_paged_detailed(
        &self,
        from_index: Option<u64>,
        limit: Option<u64>,
    ) -> Vec<AssetDetailedView>;

    /// Returns the current config.
    fn get_config(&self) -> Config;

    /// Updates the current config.
    /// - Requires one yoctoNEAR.
    /// - Requires to be called by the contract owner.
    #[payable]
    fn update_config(&mut self, config: Config);

    /// Adds an asset with a given token_id and a given asset_config.
    /// - Panics if the asset config is invalid.
    /// - Panics if an asset with the given token_id already exists.
    /// - Requires one yoctoNEAR.
    /// - Requires to be called by the contract owner.
    #[payable]
    fn add_asset(&mut self, token_id: ValidAccountId, asset_config: AssetConfig);

    /// Updates the asset config for the asset with the a given token_id.
    /// - Panics if the asset config is invalid.
    /// - Panics if an asset with the given token_id doesn't exist.
    /// - Requires one yoctoNEAR.
    /// - Requires to be called by the contract owner.
    #[payable]
    fn update_asset(&mut self, token_id: ValidAccountId, asset_config: AssetConfig);

    /// Receives the transfer from the fungible token and executes a list of actions given in the
    /// message on behalf of the sender. The actions that can be executed should be limited to a set
    /// that doesn't require pricing.
    /// - Requires to be called by the fungible token account.
    fn ft_on_transfer(
        &mut self,
        sender_id: ValidAccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128>;

    /// The method will execute a given list of actions in the msg using the prices from the `data`
    /// provided by the oracle on behalf of the sender_id.
    /// - Requires to be called by the oracle account ID.
    fn oracle_on_call(&mut self, sender_id: ValidAccountId, data: PriceData, msg: String);

    /// Claims all unclaimed farm rewards.
    fn account_farm_claim_all(&mut self);

    /// Returns an asset farm for a given farm ID.
    fn get_asset_farm(&self, farm_id: FarmId) -> Option<AssetFarm>;

    /// Returns a list of pairs (farm ID, asset farm) for a given list of farm IDs.
    fn get_asset_farms(&self, farm_ids: Vec<FarmId>) -> Vec<(FarmId, AssetFarm)>;

    /// Returns a list of pairs (farm ID, asset farm) from a given index up to a given limit.
    ///
    /// Note, the number of returned elements may be twice larger than the limit, due to the
    /// pagination implementation. To continue to the next page use `from_index + limit`.
    fn get_asset_farms_paged(
        &self,
        from_index: Option<u64>,
        limit: Option<u64>,
    ) -> Vec<(FarmId, AssetFarm)>;

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
    fn add_asset_farm_reward(
        &mut self,
        farm_id: FarmId,
        token_id: ValidAccountId,
        new_reward_per_day: WrappedBalance,
        new_booster_log_base: WrappedBalance,
        extra_amount: WrappedBalance,
    );
}
```

## Structures and types

```rust
pub struct AssetView {
    pub token_id: TokenId,
    #[serde(with = "u128_dec_format")]
    pub balance: Balance,
    /// The number of shares this account holds in the corresponding asset pool
    pub shares: Shares,
    /// The current APR for this asset (either supply or borrow APR).
    pub apr: BigDecimal,
}

pub enum FarmId {
    Supplied(TokenId),
    Borrowed(TokenId),
}

pub struct AccountFarmView {
    pub farm_id: FarmId,
    pub rewards: Vec<AccountFarmRewardView>,
}

pub struct AccountFarmRewardView {
    pub reward_token_id: TokenId,
    pub asset_farm_reward: AssetFarmReward,
    #[serde(with = "u128_dec_format")]
    pub boosted_shares: Balance,
    #[serde(with = "u128_dec_format")]
    pub unclaimed_amount: Balance,
}

pub struct AccountDetailedView {
    pub account_id: AccountId,
    /// A list of assets that are supplied by the account (but not used a collateral).
    pub supplied: Vec<AssetView>,
    /// A list of assets that are used as a collateral.
    pub collateral: Vec<AssetView>,
    /// A list of assets that are borrowed.
    pub borrowed: Vec<AssetView>,
    /// Account farms
    pub farms: Vec<AccountFarmView>,
}

/// Limited view of the account structure for liquidations
pub struct Account {
    /// A copy of an account ID. Saves one storage_read when iterating on accounts.
    pub account_id: AccountId,
    /// A list of collateral assets.
    pub collateral: Vec<CollateralAsset>,
    /// A list of borrowed assets.
    pub borrowed: Vec<BorrowedAsset>,
}

pub struct CollateralAsset {
    pub token_id: TokenId,
    pub shares: Shares,
}

pub struct BorrowedAsset {
    pub token_id: TokenId,
    pub shares: Shares,
}

pub struct AssetDetailedView {
    pub token_id: TokenId,
    /// Total supplied including collateral, but excluding reserved.
    pub supplied: Pool,
    /// Total borrowed.
    pub borrowed: Pool,
    /// The amount reserved for the stability. This amount can also be borrowed and affects
    /// borrowing rate.
    #[serde(with = "u128_dec_format")]
    pub reserved: Balance,
    /// When the asset was last updated. It's always going to be the current block timestamp.
    #[serde(with = "u64_dec_format")]
    pub last_update_timestamp: Timestamp,
    /// The asset config.
    pub config: AssetConfig,
    /// Current APR excluding farms for supplying the asset.
    pub supply_apr: BigDecimal,
    /// Current APR excluding farms for borrowing the asset.
    pub borrow_apr: BigDecimal,
    /// Asset farms
    pub farms: Vec<AssetFarmView>,
}

pub struct AssetFarmView {
    pub farm_id: FarmId,
    /// Active rewards for the farm
    pub rewards: HashMap<TokenId, AssetFarmReward>,
}

pub struct AssetFarm {
    #[serde(with = "u64_dec_format")]
    pub block_timestamp: Timestamp,
    /// Active rewards for the farm
    pub rewards: HashMap<TokenId, AssetFarmReward>,
}

pub struct AssetFarmReward {
    /// The amount of reward distributed per day.
    #[serde(with = "u128_dec_format")]
    pub reward_per_day: Balance,
    /// The log base for the booster. Used to compute boosted shares per account.
    /// Including decimals of the booster.
    #[serde(with = "u128_dec_format")]
    pub booster_log_base: Balance,

    /// The amount of rewards remaining to distribute.
    #[serde(with = "u128_dec_format")]
    pub remaining_rewards: Balance,

    /// The total number of boosted shares.
    #[serde(with = "u128_dec_format")]
    pub boosted_shares: Balance,
}

pub struct Asset {
    /// Total supplied including collateral, but excluding reserved.
    pub supplied: Pool,
    /// Total borrowed.
    pub borrowed: Pool,
    /// The amount reserved for the stability. This amount can also be borrowed and affects
    /// borrowing rate.
    #[serde(with = "u128_dec_format")]
    pub reserved: Balance,
    /// When the asset was last updated. It's always going to be the current block timestamp.
    #[serde(with = "u64_dec_format")]
    pub last_update_timestamp: Timestamp,
    /// The asset config.
    pub config: AssetConfig,
}

pub struct Pool {
    pub shares: Shares,
    #[serde(with = "u128_dec_format")]
    pub balance: Balance,
}

/// Represents an asset config.
/// Example:
/// 25% reserve, 80% target utilization, 12% target APR, 250% max APR, 60% vol
/// no extra decimals, can be deposited, withdrawn, used as a collateral, borrowed
/// JSON:
/// ```json
/// {
///   "reserve_ratio": 2500,
///   "target_utilization": 8000,
///   "target_utilization_rate": "1000000000003593629036885046",
///   "max_utilization_rate": "1000000000039724853136740579",
///   "volatility_ratio": 6000,
///   "extra_decimals": 0,
///   "can_deposit": true,
///   "can_withdraw": true,
///   "can_use_as_collateral": true,
///   "can_borrow": true
/// }
/// ```
pub struct AssetConfig {
    /// The ratio of interest that is reserved by the protocol (multiplied by 10000).
    /// E.g. 2500 means 25% from borrowed interests goes to the reserve.
    pub reserve_ratio: u32,
    /// Target utilization ratio (multiplied by 10000).
    /// E.g. 8000 means the protocol targets 80% of assets are borrowed.
    pub target_utilization: u32,
    /// The compounding rate at target utilization ratio.
    /// Use `apr_to_rate.py` script to compute the value for a given APR.
    /// Given as a decimal string. E.g. "1000000000003593629036885046" for 12% APR.
    pub target_utilization_rate: LowU128,
    /// The compounding rate at 100% utilization.
    /// Use `apr_to_rate.py` script to compute the value for a given APR.
    /// Given as a decimal string. E.g. "1000000000039724853136740579" for 250% APR.
    pub max_utilization_rate: LowU128,
    /// Volatility ratio (multiplied by 10000).
    /// It defines which percentage collateral that covers borrowing as well as which percentage of
    /// borrowed asset can be taken.
    /// E.g. 6000 means 60%. If an account has 100 $ABC in collateral and $ABC is at 10$ per token,
    /// the collateral value is 1000$, but the borrowing power is 60% or $600.
    /// Now if you're trying to borrow $XYZ and it's volatility ratio is 80%, then you can only
    /// borrow less than 80% of $600 = $480 of XYZ before liquidation can begin.
    pub volatility_ratio: u32,
    /// The amount of extra decimals to use for the fungible token. For example, if the asset like
    /// USDT has `6` decimals in the metadata, the `extra_decimals` can be set to `12`, to make the
    /// inner balance of USDT at `18` decimals.
    pub extra_decimals: u8,
    /// Whether the deposits of this assets are enabled.
    pub can_deposit: bool,
    /// Whether the withdrawals of this assets are enabled.
    pub can_withdraw: bool,
    /// Whether this assets can be used as collateral.
    pub can_use_as_collateral: bool,
    /// Whether this assets can be borrowed.
    pub can_borrow: bool,
}

pub struct AssetAmount {
    pub token_id: TokenId,
    /// The amount of tokens intended to be used for the action.
    /// If `None`, then the maximum amount will be tried.
    pub amount: Option<WrappedBalance>,
    /// The maximum amount of tokens that can be used for the action.
    /// If `None`, then the maximum `available` amount will be used.
    pub max_amount: Option<WrappedBalance>,
}

/// Contract config
pub struct Config {
    /// The account ID of the oracle contract
    pub oracle_account_id: ValidAccountId,

    /// The account ID of the contract owner that allows to modify config, assets and use reserves.
    pub owner_id: ValidAccountId,

    /// The account ID of the booster token contract.
    pub booster_token_id: TokenId,

    /// The number of decimals of the booster fungible token.
    pub booster_decimals: u8,
}

pub enum Action {
    Withdraw(AssetAmount),
    IncreaseCollateral(AssetAmount),
    DecreaseCollateral(AssetAmount),
    Borrow(AssetAmount),
    Repay(AssetAmount),
    Liquidate {
        account_id: ValidAccountId,
        in_assets: Vec<AssetAmount>,
        out_assets: Vec<AssetAmount>,
    },
}

pub enum TokenReceiverMsg {
    Execute { actions: Vec<Action> },
    /// The entire amount will be deposited to the asset reserve. 
    DepositToReserve,
}

enum PriceReceiverMsg {
    Execute { actions: Vec<Action> },
}

pub type TokenId = AccountId;
```

## Also storage management

```rust
pub struct StorageBalance {
    pub total: U128,
    pub available: U128,
}

pub struct StorageBalanceBounds {
    pub min: U128,
    pub max: Option<U128>,
}

pub trait StorageManagement {
    // if `registration_only=true` MUST refund above the minimum balance if the account didn't exist and
    //     refund full deposit if the account exists.
    fn storage_deposit(
        &mut self,
        account_id: Option<ValidAccountId>,
        registration_only: Option<bool>,
    ) -> StorageBalance;

    /// Withdraw specified amount of available Ⓝ for predecessor account.
    ///
    /// This method is safe to call. It MUST NOT remove data.
    ///
    /// `amount` is sent as a string representing an unsigned 128-bit integer. If
    /// omitted, contract MUST refund full `available` balance. If `amount` exceeds
    /// predecessor account's available balance, contract MUST panic.
    ///
    /// If predecessor account not registered, contract MUST panic.
    ///
    /// MUST require exactly 1 yoctoNEAR attached balance to prevent restricted
    /// function-call access-key call (UX wallet security)
    ///
    /// Returns the StorageBalance structure showing updated balances.
    fn storage_withdraw(&mut self, amount: Option<U128>) -> StorageBalance;

    /// Unregestering the account is not allowed to not break the order of accounts.
    fn storage_unregister(&mut self, force: Option<bool>) -> bool;

    fn storage_balance_bounds(&self) -> StorageBalanceBounds;

    fn storage_balance_of(&self, account_id: ValidAccountId) -> Option<StorageBalance>;
}
```
