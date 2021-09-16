use crate::*;

#[derive(
    BorshSerialize, BorshDeserialize, Copy, Clone, Default, Debug, PartialEq, PartialOrd, Eq, Ord,
)]
pub struct Shares(pub Balance);

#[derive(BorshSerialize, BorshDeserialize, Clone)]
pub struct Pool {
    pub shares: Shares,
    pub balance: Balance,
}

impl Pool {
    pub fn new() -> Self {
        Self {
            shares: Shares(0),
            balance: 0,
        }
    }

    pub fn amount_to_shares(&self, amount: Balance, round_up: bool) -> Shares {
        Shares(if self.balance == 0 {
            amount
        } else {
            let extra = if round_up {
                U256::from(self.balance - 1)
            } else {
                U256::zero()
            };
            ((U256::from(self.shares.0) * U256::from(amount) + extra) / U256::from(self.balance))
                .as_u128()
        })
    }

    pub fn shares_to_amount(&self, shares: Shares, round_up: bool) -> Balance {
        if shares.0 >= self.balance {
            self.balance
        } else {
            let extra = if round_up {
                U256::from(self.shares.0 - 1)
            } else {
                U256::zero()
            };
            ((U256::from(self.balance) * U256::from(shares.0) + extra) / U256::from(self.shares.0))
                .as_u128()
        }
    }

    pub fn deposit(&mut self, shares: Shares, amount: Balance) {
        self.shares.0 += shares.0;
        self.balance += amount;
    }

    pub fn withdraw(&mut self, shares: Shares, amount: Balance) {
        self.shares.0 -= shares.0;
        self.balance -= amount;
    }
}
