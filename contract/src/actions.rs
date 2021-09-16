use crate::*;
use near_sdk::json_types::WrappedBalance;

pub struct AssetAmount {
    pub token_account_id: TokenAccountId,
    pub min_amount: Option<WrappedBalance>,
    pub max_amount: Option<WrappedBalance>,
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

impl Contract {
    pub fn internal_execute(
        &mut self,
        actor_account_id: &AccountId,
        account: &mut Account,
        actions: Vec<Action>,
        price_data: Option<PriceData>,
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
                    self.internal_decrease_collateral(account, &asset_amount);
                }
                Action::Borrow(asset_amount) => {
                    need_risk_check = true;
                    let amount = self.internal_borrow(account, &asset_amount);
                }
                Action::Repay(asset_amount) => {
                    let amount = self.internal_repay(account, &asset_amount);
                }
                Action::Liquidate {
                    account_id,
                    in_assets,
                    out_assets,
                } => {
                    assert_ne!(
                        actor_account_id,
                        account_id.as_ref(),
                        "Can't liquidate yourself"
                    );
                }
            }
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
        let mut asset = self.internal_unwrap_asset(&asset_amount.token_account_id);
        let mut account_asset = account.internal_unwrap_asset(&asset_amount.token_account_id);

        let (shares, _amount) =
            asset_amount_to_shares(&asset.supplied, account_asset.shares, &asset_amount);

        account_asset.withdraw_shares(shares);
        account.set_asset(&asset_amount.token_account_id, account_asset);

        account.increase_collateral(&asset_amount.token_account_id, shares);
    }

    pub fn internal_decrease_collateral(
        &mut self,
        account: &mut Account,
        asset_amount: &AssetAmount,
    ) {
        let mut asset = self.internal_unwrap_asset(&asset_amount.token_account_id);
        let mut account_asset = account.internal_unwrap_asset(&asset_amount.token_account_id);
        let collateral_shares = account.internal_unwrap_collateral(&asset_amount.token_account_id);

        let (shares, _amount) =
            asset_amount_to_shares(&asset.supplied, collateral_shares, &asset_amount);

        account.decrease_collateral(&asset_amount.token_account_id, shares);

        account_asset.deposit_shares(shares);
        account.set_asset(&asset_amount.token_account_id, account_asset);
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

    pub fn internal_repay(&mut self, account: &mut Account, asset_amount: &AssetAmount) -> Balance {
        let mut asset = self.internal_unwrap_asset(&asset_amount.token_account_id);
        let mut account_asset = account.internal_unwrap_asset(&asset_amount.token_account_id);
        let available_borrowed_shares =
            account.internal_unwrap_borrowed(&asset_amount.token_account_id);

        let (mut borrowed_shares, mut amount) =
            asset_amount_to_shares(&asset.borrowed, available_borrowed_shares, &asset_amount);

        let mut supplied_shares = asset.supplied.amount_to_shares(amount, true);
        if supplied_shares > account_asset.shares {
            supplied_shares = account_asset.shares;
            amount = asset.supplied.shares_to_amount(supplied_shares, false);
            if let Some(min_amount) = &asset_amount.min_amount {
                assert!(amount >= min_amount.0, "Not enough supplied balance");
            }
            assert!(amount > 0, "Repayment amount can't be 0");

            borrowed_shares = asset.borrowed.amount_to_shares(amount, true);
            assert!(borrowed_shares.0 > 0, "Shares can't be 0");
            assert!(borrowed_shares <= available_borrowed_shares);
        }

        asset.supplied.withdraw(supplied_shares, amount);
        asset.borrowed.withdraw(borrowed_shares, amount);
        self.internal_set_asset(&asset_amount.token_account_id, asset);

        account.decrease_borrowed(&asset_amount.token_account_id, borrowed_shares);

        account_asset.withdraw_shares(supplied_shares);
        account.set_asset(&asset_amount.token_account_id, account_asset);

        amount
    }
}

fn asset_amount_to_shares(
    pool: &Pool,
    available_shares: Shares,
    asset_amount: &AssetAmount,
) -> (Shares, Balance) {
    let (shares, amount) = if let Some(min_amount) = &asset_amount.min_amount {
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
