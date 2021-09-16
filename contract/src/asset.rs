use crate::*;
use near_sdk::Duration;

#[derive(BorshSerialize, BorshDeserialize)]
pub struct Asset {
    pub supplied: Pool,
    pub borrowed: Pool,
    pub reserved: Balance,
    pub last_update_timestamp: Timestamp,
    pub config: AssetConfig,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub enum VAsset {
    Current(Asset),
}

impl From<VAsset> for Asset {
    fn from(v: VAsset) -> Self {
        match v {
            VAsset::Current(c) => c,
        }
    }
}

impl From<Asset> for VAsset {
    fn from(c: Asset) -> Self {
        VAsset::Current(c)
    }
}

impl Asset {
    pub fn new(timestamp: Timestamp, config: AssetConfig) -> Self {
        Self {
            supplied: Pool::new(),
            borrowed: Pool::new(),
            reserved: 0,
            last_update_timestamp: timestamp,
            config,
        }
    }

    pub fn get_rate(&self) -> BigDecimal {
        self.config
            .get_rate(self.borrowed.balance, self.supplied.balance + self.reserved)
    }

    // n = 31536000000 ms in a year (365 days)
    //
    // Compute `r` from `X`. `X` is desired APY
    // (1 + r / n) ** n = X (2 == 200%)
    // n * log(1 + r / n) = log(x)
    // log(1 + r / n) = log(x) / n
    // log(1 + r  / n) = log( x ** (1 / n))
    // 1 + r / n = x ** (1 / n)
    // r / n = (x ** (1 / n)) - 1
    // r = n * ((x ** (1 / n)) - 1)
    // n = in millis
    fn compound(&mut self, time_diff_ms: Duration) {
        let total_supplied_balance = self.supplied.balance + self.reserved;
        let rate = self.get_rate();
        let interest = rate
            .pow(time_diff_ms)
            .round_mul_u128(total_supplied_balance);
        let reserved = ratio(interest, self.config.reserve_ratio);
        self.supplied.balance += interest - reserved;
        self.reserved += reserved;
        self.borrowed.balance += interest;
    }

    pub fn touch(&mut self) {
        let timestamp = env::block_timestamp();
        let time_diff_ms = nano_to_ms(timestamp - self.last_update_timestamp);
        if time_diff_ms > 0 {
            // update
            self.last_update_timestamp += ms_to_nano(time_diff_ms);
            self.compound(time_diff_ms);
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
        let shares: Shares = asset.supplied.amount_to_shares(amount, false);
        asset.supplied.deposit(shares, amount);
        account.deposit_shares(token_account_id, shares);
        self.internal_set_asset(token_account_id, asset);
    }

    pub fn internal_unwrap_asset(&self, token_account_id: &TokenAccountId) -> Asset {
        self.internal_get_asset(token_account_id)
            .expect("Asset not found")
    }

    pub fn internal_get_asset(&self, token_account_id: &TokenAccountId) -> Option<Asset> {
        self.assets.get(token_account_id).map(|o| o.into())
    }

    pub fn internal_set_asset(&mut self, token_account_id: &TokenAccountId, asset: Asset) {
        self.assets.insert(token_account_id, &asset.into());
    }
}
