use crate::*;

const MAX_NUM_ASSETS: usize = 8;

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct AssetAmount {
    pub token_id: TokenId,
    /// The amount of tokens intended to be used for the action.
    /// If `None`, then the maximum amount will be tried.
    pub amount: Option<WrappedBalance>,
    /// The maximum amount of tokens that can be used for the action.
    /// If `None`, then the maximum `available` amount will be used.
    pub max_amount: Option<WrappedBalance>,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
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

impl Contract {
    pub fn internal_execute(
        &mut self,
        account_id: &AccountId,
        account: &mut Account,
        storage: &mut Storage,
        actions: Vec<Action>,
        prices: Prices,
    ) {
        let mut need_risk_check = false;
        let mut need_number_check = false;
        for action in actions {
            match action {
                Action::Withdraw(asset_amount) => {
                    account.add_affected_farm(FarmId::Supplied(asset_amount.token_id.clone()));
                    let amount = self.internal_withdraw(account, &asset_amount);
                    self.internal_ft_transfer(account_id, &asset_amount.token_id, amount);
                }
                Action::IncreaseCollateral(asset_amount) => {
                    need_number_check = true;
                    self.internal_increase_collateral(account, &asset_amount);
                }
                Action::DecreaseCollateral(asset_amount) => {
                    need_risk_check = true;
                    let mut account_asset =
                        account.internal_get_asset_or_default(&asset_amount.token_id);
                    self.internal_decrease_collateral(&mut account_asset, account, &asset_amount);
                    account.internal_set_asset(&asset_amount.token_id, account_asset);
                }
                Action::Borrow(asset_amount) => {
                    need_number_check = true;
                    need_risk_check = true;
                    account.add_affected_farm(FarmId::Supplied(asset_amount.token_id.clone()));
                    account.add_affected_farm(FarmId::Borrowed(asset_amount.token_id.clone()));
                    let amount = self.internal_borrow(account, &asset_amount);
                }
                Action::Repay(asset_amount) => {
                    let mut account_asset = account.internal_unwrap_asset(&asset_amount.token_id);
                    account.add_affected_farm(FarmId::Supplied(asset_amount.token_id.clone()));
                    account.add_affected_farm(FarmId::Borrowed(asset_amount.token_id.clone()));
                    let amount = self.internal_repay(&mut account_asset, account, &asset_amount);
                    account.internal_set_asset(&asset_amount.token_id, account_asset);
                }
                Action::Liquidate {
                    account_id: liquidation_account_id,
                    in_assets,
                    out_assets,
                } => {
                    assert_ne!(
                        account_id,
                        liquidation_account_id.as_ref(),
                        "Can't liquidate yourself"
                    );
                    assert!(!in_assets.is_empty() && !out_assets.is_empty());
                    self.internal_liquidate(
                        account,
                        storage,
                        &prices,
                        liquidation_account_id,
                        in_assets,
                        out_assets,
                    );
                }
            }
        }
        if need_number_check {
            assert!(account.collateral.len() + account.borrowed.len() <= MAX_NUM_ASSETS);
        }
        if need_risk_check {
            assert!(self.compute_max_discount(account, &prices) == BigDecimal::zero());
        }
    }

    pub fn internal_deposit(
        &mut self,
        account: &mut Account,
        token_id: &TokenId,
        amount: Balance,
    ) -> Shares {
        let mut asset = self.internal_unwrap_asset(token_id);
        let mut account_asset = account.internal_get_asset_or_default(token_id);

        let shares: Shares = asset.supplied.amount_to_shares(amount, false);

        account_asset.deposit_shares(shares);
        account.internal_set_asset(&token_id, account_asset);

        asset.supplied.deposit(shares, amount);
        self.internal_set_asset(token_id, asset);

        shares
    }

    pub fn internal_withdraw(
        &mut self,
        account: &mut Account,
        asset_amount: &AssetAmount,
    ) -> Balance {
        let mut asset = self.internal_unwrap_asset(&asset_amount.token_id);
        assert!(
            asset.config.can_withdraw,
            "Withdrawals for this asset are not enabled"
        );

        let mut account_asset = account.internal_unwrap_asset(&asset_amount.token_id);

        let (shares, amount) =
            asset_amount_to_shares(&asset.supplied, account_asset.shares, &asset_amount, false);

        account_asset.withdraw_shares(shares);
        account.internal_set_asset(&asset_amount.token_id, account_asset);

        asset.supplied.withdraw(shares, amount);
        self.internal_set_asset(&asset_amount.token_id, asset);

        amount
    }

    pub fn internal_increase_collateral(
        &mut self,
        account: &mut Account,
        asset_amount: &AssetAmount,
    ) -> Balance {
        let asset = self.internal_unwrap_asset(&asset_amount.token_id);
        assert!(
            asset.config.can_use_as_collateral,
            "Thi asset can't be used as a collateral"
        );

        let mut account_asset = account.internal_unwrap_asset(&asset_amount.token_id);

        let (shares, amount) =
            asset_amount_to_shares(&asset.supplied, account_asset.shares, &asset_amount, false);

        account_asset.withdraw_shares(shares);
        account.internal_set_asset(&asset_amount.token_id, account_asset);

        account.increase_collateral(&asset_amount.token_id, shares);

        amount
    }

    pub fn internal_decrease_collateral(
        &mut self,
        account_asset: &mut AccountAsset,
        account: &mut Account,
        asset_amount: &AssetAmount,
    ) -> Balance {
        let asset = self.internal_unwrap_asset(&asset_amount.token_id);
        let collateral_shares = account.internal_unwrap_collateral(&asset_amount.token_id);

        let (shares, amount) =
            asset_amount_to_shares(&asset.supplied, collateral_shares, &asset_amount, false);

        account.decrease_collateral(&asset_amount.token_id, shares);

        account_asset.deposit_shares(shares);

        amount
    }

    pub fn internal_borrow(
        &mut self,
        account: &mut Account,
        asset_amount: &AssetAmount,
    ) -> Balance {
        let mut asset = self.internal_unwrap_asset(&asset_amount.token_id);
        assert!(asset.config.can_borrow, "Thi asset can't be used borrowed");

        let mut account_asset = account.internal_get_asset_or_default(&asset_amount.token_id);

        let available_amount = asset.available_amount();
        let max_borrow_shares = asset.borrowed.amount_to_shares(available_amount, false);

        let (borrowed_shares, amount) =
            asset_amount_to_shares(&asset.borrowed, max_borrow_shares, &asset_amount, true);

        assert!(amount <= available_amount);

        let supplied_shares: Shares = asset.supplied.amount_to_shares(amount, false);

        asset.borrowed.deposit(borrowed_shares, amount);
        asset.supplied.deposit(supplied_shares, amount);
        self.internal_set_asset(&asset_amount.token_id, asset);

        account.increase_borrowed(&asset_amount.token_id, borrowed_shares);

        account_asset.deposit_shares(supplied_shares);
        account.internal_set_asset(&asset_amount.token_id, account_asset);

        amount
    }

    pub fn internal_repay(
        &mut self,
        account_asset: &mut AccountAsset,
        account: &mut Account,
        asset_amount: &AssetAmount,
    ) -> Balance {
        let mut asset = self.internal_unwrap_asset(&asset_amount.token_id);
        let available_borrowed_shares = account.internal_unwrap_borrowed(&asset_amount.token_id);

        let (mut borrowed_shares, mut amount) = asset_amount_to_shares(
            &asset.borrowed,
            available_borrowed_shares,
            &asset_amount,
            true,
        );

        let mut supplied_shares = asset.supplied.amount_to_shares(amount, true);
        if supplied_shares.0 > account_asset.shares.0 {
            supplied_shares = account_asset.shares;
            amount = asset.supplied.shares_to_amount(supplied_shares, false);
            if let Some(min_amount) = &asset_amount.amount {
                assert!(amount >= min_amount.0, "Not enough supplied balance");
            }
            assert!(amount > 0, "Repayment amount can't be 0");

            borrowed_shares = asset.borrowed.amount_to_shares(amount, false);
            assert!(borrowed_shares.0 > 0, "Shares can't be 0");
            assert!(borrowed_shares.0 <= available_borrowed_shares.0);
        }

        asset.supplied.withdraw(supplied_shares, amount);
        asset.borrowed.withdraw(borrowed_shares, amount);
        self.internal_set_asset(&asset_amount.token_id, asset);

        account.decrease_borrowed(&asset_amount.token_id, borrowed_shares);

        account_asset.withdraw_shares(supplied_shares);

        amount
    }

    pub fn internal_liquidate(
        &mut self,
        account: &mut Account,
        storage: &mut Storage,
        prices: &Prices,
        liquidation_account_id: ValidAccountId,
        in_assets: Vec<AssetAmount>,
        out_assets: Vec<AssetAmount>,
    ) {
        let (mut liquidation_account, liquidation_storage) =
            self.internal_unwrap_account_with_storage(liquidation_account_id.as_ref());

        let max_discount = self.compute_max_discount(&liquidation_account, &prices);
        assert!(
            max_discount > BigDecimal::zero(),
            "The liquidation account is not at risk"
        );

        let mut borrowed_repaid_sum = BigDecimal::zero();
        let mut collateral_taken_sum = BigDecimal::zero();

        for asset_amount in in_assets {
            account.add_affected_farm(FarmId::Supplied(asset_amount.token_id.clone()));
            liquidation_account.add_affected_farm(FarmId::Borrowed(asset_amount.token_id.clone()));
            let mut account_asset = account.internal_unwrap_asset(&asset_amount.token_id);
            let amount =
                self.internal_repay(&mut account_asset, &mut liquidation_account, &asset_amount);
            account.internal_set_asset(&asset_amount.token_id, account_asset);

            borrowed_repaid_sum = borrowed_repaid_sum
                + BigDecimal::from_balance_price(amount, prices.get_unwrap(&asset_amount.token_id));
        }

        for asset_amount in out_assets {
            account.add_affected_farm(FarmId::Supplied(asset_amount.token_id.clone()));
            liquidation_account.add_affected_farm(FarmId::Supplied(asset_amount.token_id.clone()));
            let mut account_asset = account.internal_get_asset_or_default(&asset_amount.token_id);
            let amount = self.internal_decrease_collateral(
                &mut account_asset,
                &mut liquidation_account,
                &asset_amount,
            );
            account.internal_set_asset(&asset_amount.token_id, account_asset);

            collateral_taken_sum = collateral_taken_sum
                + BigDecimal::from_balance_price(amount, prices.get_unwrap(&asset_amount.token_id));
        }

        let discounted_collateral_taken = collateral_taken_sum * (BigDecimal::one() - max_discount);
        assert!(
            discounted_collateral_taken <= borrowed_repaid_sum,
            "Not enough balances repaid"
        );

        let new_max_discount = self.compute_max_discount(&liquidation_account, &prices);
        assert!(
            new_max_discount > BigDecimal::zero(),
            "The liquidation amount is too large. The liquidation account should stay in risk"
        );

        self.internal_account_apply_affected_farms(&mut liquidation_account, true);
        // TODO: Fix storage increase due to farming.
        // NOTE: This method can only decrease storage, by repaying some burrowed assets and taking some
        // collateral.
        let released_bytes = env::storage_usage() - liquidation_storage.initial_storage_usage;
        // We have to adjust the initial_storage_usage for the acting account to not double count
        // the released bytes, since these released bytes belongs to the liquidation account.
        storage.initial_storage_usage += released_bytes;
        self.internal_set_account(
            liquidation_account_id.as_ref(),
            liquidation_account,
            liquidation_storage,
        );
    }

    pub fn compute_max_discount(&self, account: &Account, prices: &Prices) -> BigDecimal {
        let collateral_sum = account
            .collateral
            .iter()
            .fold(BigDecimal::zero(), |sum, c| {
                let asset = self.internal_unwrap_asset(&c.token_id);
                let balance = asset.supplied.shares_to_amount(c.shares, false);
                sum + BigDecimal::from_balance_price(balance, prices.get_unwrap(&c.token_id))
                    .mul_ratio(asset.config.volatility_ratio)
            });

        let borrowed_sum = account.borrowed.iter().fold(BigDecimal::zero(), |sum, b| {
            let asset = self.internal_unwrap_asset(&b.token_id);
            let balance = asset.borrowed.shares_to_amount(b.shares, true);
            sum + BigDecimal::from_balance_price(balance, prices.get_unwrap(&b.token_id))
                .mul_ratio(asset.config.volatility_ratio)
        });

        if borrowed_sum <= collateral_sum {
            BigDecimal::zero()
        } else {
            (borrowed_sum - collateral_sum) / borrowed_sum / BigDecimal::from(2u32)
        }
    }
}

fn asset_amount_to_shares(
    pool: &Pool,
    available_shares: Shares,
    asset_amount: &AssetAmount,
    inverse_round_direction: bool,
) -> (Shares, Balance) {
    let (shares, amount) = if let Some(min_amount) = &asset_amount.amount {
        (
            pool.amount_to_shares(min_amount.0, !inverse_round_direction),
            min_amount.0,
        )
    } else if let Some(max_amount) = &asset_amount.max_amount {
        let shares = std::cmp::min(
            available_shares.0,
            pool.amount_to_shares(max_amount.0, !inverse_round_direction)
                .0,
        )
        .into();
        (
            shares,
            std::cmp::min(
                pool.shares_to_amount(shares, inverse_round_direction),
                max_amount.0,
            ),
        )
    } else {
        (
            available_shares,
            pool.shares_to_amount(available_shares, inverse_round_direction),
        )
    };
    assert!(shares.0 > 0, "Shares can't be 0");
    assert!(amount > 0, "Amount can't be 0");
    (shares, amount)
}

#[near_bindgen]
impl Contract {
    /// Executes a given list actions on behalf of the predecessor account.
    /// - Requires one yoctoNEAR.
    #[payable]
    pub fn execute(&mut self, actions: Vec<Action>) {
        assert_one_yocto();
        let account_id = env::predecessor_account_id();
        let (mut account, mut storage) = self.internal_unwrap_account_with_storage(&account_id);
        self.internal_execute(
            &account_id,
            &mut account,
            &mut storage,
            actions,
            Prices::new(),
        );
        self.internal_set_account(&account_id, account, storage);
    }
}
