use crate::*;
use near_sdk::json_types::WrappedBalance;
use std::collections::HashMap;

const MAX_NUM_ASSETS: usize = 8;

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct AssetAmount {
    pub token_account_id: TokenAccountId,
    pub amount: Option<WrappedBalance>,
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
        actions: Vec<Action>,
        prices: HashMap<TokenAccountId, Price>,
    ) {
        let mut need_risk_check = false;
        let mut need_number_check = false;
        for action in actions {
            match action {
                Action::Withdraw(asset_amount) => {
                    let amount = self.internal_withdraw(account, &asset_amount);
                    // TODO: self.internal_ft_transfer(actor_account_id, amount);
                }
                Action::IncreaseCollateral(asset_amount) => {
                    need_number_check = true;
                    self.internal_increase_collateral(account, &asset_amount);
                }
                Action::DecreaseCollateral(asset_amount) => {
                    need_risk_check = true;
                    let mut account_asset =
                        account.internal_get_asset_or_default(&asset_amount.token_account_id);
                    self.internal_decrease_collateral(&mut account_asset, account, &asset_amount);
                    account.set_asset(&asset_amount.token_account_id, account_asset);
                }
                Action::Borrow(asset_amount) => {
                    need_number_check = true;
                    need_risk_check = true;
                    let amount = self.internal_borrow(account, &asset_amount);
                }
                Action::Repay(asset_amount) => {
                    let mut account_asset =
                        account.internal_unwrap_asset(&asset_amount.token_account_id);
                    let amount = self.internal_repay(&mut account_asset, account, &asset_amount);
                    account.set_asset(&asset_amount.token_account_id, account_asset);
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
}

impl Contract {
    pub fn internal_deposit(
        &mut self,
        account: &mut Account,
        token_account_id: &TokenAccountId,
        amount: Balance,
    ) {
        let mut asset = self.internal_unwrap_asset(token_account_id);
        let mut account_asset = account.internal_get_asset_or_default(token_account_id);

        let shares: Shares = asset.supplied.amount_to_shares(amount, false);

        account_asset.deposit_shares(shares);
        account.set_asset(&token_account_id, account_asset);

        asset.supplied.deposit(shares, amount);
        self.internal_set_asset(token_account_id, asset);
    }

    pub fn internal_withdraw(
        &mut self,
        account: &mut Account,
        asset_amount: &AssetAmount,
    ) -> Balance {
        let mut asset = self.internal_unwrap_asset(&asset_amount.token_account_id);
        let mut account_asset = account.internal_unwrap_asset(&asset_amount.token_account_id);

        let (shares, amount) =
            asset_amount_to_shares(&asset.supplied, account_asset.shares, &asset_amount);

        account_asset.withdraw_shares(shares);
        account.set_asset(&asset_amount.token_account_id, account_asset);

        asset.supplied.withdraw(shares, amount);
        self.internal_set_asset(&asset_amount.token_account_id, asset);

        amount
    }

    pub fn internal_increase_collateral(
        &mut self,
        account: &mut Account,
        asset_amount: &AssetAmount,
    ) {
        let asset = self.internal_unwrap_asset(&asset_amount.token_account_id);
        let mut account_asset = account.internal_unwrap_asset(&asset_amount.token_account_id);

        let (shares, _amount) =
            asset_amount_to_shares(&asset.supplied, account_asset.shares, &asset_amount);

        account_asset.withdraw_shares(shares);
        account.set_asset(&asset_amount.token_account_id, account_asset);

        account.increase_collateral(&asset_amount.token_account_id, shares);
    }

    pub fn internal_decrease_collateral(
        &mut self,
        account_asset: &mut AccountAsset,
        account: &mut Account,
        asset_amount: &AssetAmount,
    ) -> Balance {
        let asset = self.internal_unwrap_asset(&asset_amount.token_account_id);
        let collateral_shares = account.internal_unwrap_collateral(&asset_amount.token_account_id);

        let (shares, amount) =
            asset_amount_to_shares(&asset.supplied, collateral_shares, &asset_amount);

        account.decrease_collateral(&asset_amount.token_account_id, shares);

        account_asset.deposit_shares(shares);

        amount
    }

    pub fn internal_borrow(
        &mut self,
        account: &mut Account,
        asset_amount: &AssetAmount,
    ) -> Balance {
        let mut asset = self.internal_unwrap_asset(&asset_amount.token_account_id);
        let mut account_asset =
            account.internal_get_asset_or_default(&asset_amount.token_account_id);

        let available_amount = asset.available_amount();
        let max_borrow_shares = asset.borrowed.amount_to_shares(available_amount, false);

        // TODO: Round up shares
        let (borrowed_shares, amount) =
            asset_amount_to_shares(&asset.borrowed, max_borrow_shares, &asset_amount);

        assert!(amount <= available_amount);

        let supplied_shares: Shares = asset.supplied.amount_to_shares(amount, false);

        asset.borrowed.deposit(borrowed_shares, amount);
        asset.supplied.deposit(supplied_shares, amount);
        self.internal_set_asset(&asset_amount.token_account_id, asset);

        account.increase_borrowed(&asset_amount.token_account_id, borrowed_shares);

        account_asset.deposit_shares(supplied_shares);
        account.set_asset(&asset_amount.token_account_id, account_asset);

        amount
    }

    pub fn internal_repay(
        &mut self,
        account_asset: &mut AccountAsset,
        account: &mut Account,
        asset_amount: &AssetAmount,
    ) -> Balance {
        let mut asset = self.internal_unwrap_asset(&asset_amount.token_account_id);
        let available_borrowed_shares =
            account.internal_unwrap_borrowed(&asset_amount.token_account_id);

        let (mut borrowed_shares, mut amount) =
            asset_amount_to_shares(&asset.borrowed, available_borrowed_shares, &asset_amount);

        let mut supplied_shares = asset.supplied.amount_to_shares(amount, true);
        if supplied_shares > account_asset.shares {
            supplied_shares = account_asset.shares;
            amount = asset.supplied.shares_to_amount(supplied_shares, false);
            if let Some(min_amount) = &asset_amount.amount {
                assert!(amount >= min_amount.0, "Not enough supplied balance");
            }
            assert!(amount > 0, "Repayment amount can't be 0");

            // TODO: Round down?
            borrowed_shares = asset.borrowed.amount_to_shares(amount, true);
            assert!(borrowed_shares.0 > 0, "Shares can't be 0");
            assert!(borrowed_shares <= available_borrowed_shares);
        }

        asset.supplied.withdraw(supplied_shares, amount);
        asset.borrowed.withdraw(borrowed_shares, amount);
        self.internal_set_asset(&asset_amount.token_account_id, asset);

        account.decrease_borrowed(&asset_amount.token_account_id, borrowed_shares);

        account_asset.withdraw_shares(supplied_shares);

        amount
    }

    pub fn internal_liquidate(
        &mut self,
        account: &mut Account,
        prices: &HashMap<TokenAccountId, Price>,
        liquidation_account_id: ValidAccountId,
        in_assets: Vec<AssetAmount>,
        out_assets: Vec<AssetAmount>,
    ) {
        let mut liquidation_account = self.internal_unwrap_account(liquidation_account_id.as_ref());

        let max_discount = self.compute_max_discount(&liquidation_account, &prices);
        assert!(
            max_discount > BigDecimal::zero(),
            "The liquidation account is not at risk"
        );

        let mut borrowed_repaid_sum = BigDecimal::zero();
        let mut collateral_taken_sum = BigDecimal::zero();

        for asset_amount in in_assets {
            let mut account_asset = account.internal_unwrap_asset(&asset_amount.token_account_id);
            let amount =
                self.internal_repay(&mut account_asset, &mut liquidation_account, &asset_amount);
            account.set_asset(&asset_amount.token_account_id, account_asset);

            borrowed_repaid_sum = borrowed_repaid_sum
                + BigDecimal::from_balance_price(
                    amount,
                    prices
                        .get(&asset_amount.token_account_id)
                        .expect("Asset price is missing"),
                );
        }

        for asset_amount in out_assets {
            let mut account_asset =
                account.internal_get_asset_or_default(&asset_amount.token_account_id);
            let amount = self.internal_decrease_collateral(
                &mut account_asset,
                &mut liquidation_account,
                &asset_amount,
            );
            account.set_asset(&asset_amount.token_account_id, account_asset);

            collateral_taken_sum = collateral_taken_sum
                + BigDecimal::from_balance_price(
                    amount,
                    prices
                        .get(&asset_amount.token_account_id)
                        .expect("Asset price is missing"),
                );
        }

        // Backed loan = 100 NEAR * 10$ = 1000$
        // Collateral: 100 NEAR * 10$ * 40% = 400$
        // Borrowed: 500 DAI * 1$ = 500$
        // Max discount: 25% / 2 = 12.5%

        // Liquidation:
        // Repay dept: 200 DAI * 1$ = 200$
        // Take collateral: 22.85 NEAR * 10$ = 228.5$

        // discounted_collateral_taken = collateral_taken * (1 - max_discount) = 199.9375

        // New balance:
        // Backed loan = 77.15 NEAR * 10$ = 771.5$
        // Collateral: 77.15 NEAR * 10$ * 40% = 308.6$
        // Borrowed: 300 DAI * 1$ = 300$
        // Max discount: 0%

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

        self.internal_set_account(liquidation_account_id.as_ref(), liquidation_account);
    }

    pub fn compute_max_discount(
        &self,
        account: &Account,
        prices: &HashMap<TokenAccountId, Price>,
    ) -> BigDecimal {
        let collateral_sum = account
            .collateral
            .iter()
            .fold(BigDecimal::zero(), |sum, c| {
                let asset = self.internal_unwrap_asset(&c.token_account_id);
                let balance = asset.supplied.shares_to_amount(c.shares, false);
                sum + BigDecimal::from_balance_price(
                    balance,
                    prices
                        .get(&c.token_account_id)
                        .expect("Asset price is missing"),
                )
                .mul_ratio(asset.config.collateral_ratio)
            });

        let borrowed_sum = account.borrowed.iter().fold(BigDecimal::zero(), |sum, b| {
            let asset = self.internal_unwrap_asset(&b.token_account_id);
            let balance = asset.borrowed.shares_to_amount(b.shares, true);
            sum + BigDecimal::from_balance_price(
                balance,
                prices
                    .get(&b.token_account_id)
                    .expect("Asset price is missing"),
            )
        });

        if borrowed_sum <= collateral_sum {
            BigDecimal::zero()
        } else {
            assert!(
                collateral_sum > BigDecimal::zero(),
                "The collateral sum can't be 0"
            );
            (borrowed_sum - collateral_sum) / collateral_sum
        }
    }
}

fn asset_amount_to_shares(
    pool: &Pool,
    available_shares: Shares,
    asset_amount: &AssetAmount,
) -> (Shares, Balance) {
    let (shares, amount) = if let Some(min_amount) = &asset_amount.amount {
        (pool.amount_to_shares(min_amount.0, true), min_amount.0)
    } else if let Some(max_amount) = &asset_amount.max_amount {
        let shares = std::cmp::min(available_shares, pool.amount_to_shares(max_amount.0, true));
        (
            shares,
            std::cmp::min(pool.shares_to_amount(shares, false), max_amount.0),
        )
    } else {
        (
            available_shares,
            pool.shares_to_amount(available_shares, false),
        )
    };
    assert!(shares.0 > 0, "Shares can't be 0");
    assert!(amount > 0, "Amount can't be 0");
    (shares, amount)
}
