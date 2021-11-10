use crate::*;
use near_sdk::json_types::U128;

pub type Shares = U128;

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug))]
#[serde(crate = "near_sdk::serde")]
pub struct Pool {
    pub shares: Shares,
    #[serde(with = "u128_dec_format")]
    pub balance: Balance,
}

impl Pool {
    pub fn new() -> Self {
        Self {
            shares: 0.into(),
            balance: 0,
        }
    }

    pub fn amount_to_shares(&self, amount: Balance, round_up: bool) -> Shares {
        let shares = if self.balance == 0 {
            amount
        } else {
            let extra = if round_up {
                U256::from(self.balance - 1)
            } else {
                U256::zero()
            };
            ((U256::from(self.shares.0) * U256::from(amount) + extra) / U256::from(self.balance))
                .as_u128()
        };
        shares.into()
    }

    pub fn shares_to_amount(&self, shares: Shares, round_up: bool) -> Balance {
        if shares.0 >= self.balance || shares.0 == self.shares.0 {
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
