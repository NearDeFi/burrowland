use crate::*;
use near_sdk::Duration;

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Asset {
    pub supplied: Pool,
    pub borrowed: Pool,
    #[serde(with = "u128_dec_format")]
    pub reserved: Balance,
    #[serde(with = "u64_dec_format")]
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
        let rate = self.get_rate();
        log!("Rate is {}", rate);
        let interest =
            rate.pow(time_diff_ms).round_mul_u128(self.borrowed.balance) - self.borrowed.balance;
        log!("Interest added: {}", interest);
        let reserved = ratio(interest, self.config.reserve_ratio);
        self.supplied.balance += interest - reserved;
        self.reserved += reserved;
        self.borrowed.balance += interest;
    }

    pub fn update(&mut self) {
        let timestamp = env::block_timestamp();
        let time_diff_ms = nano_to_ms(timestamp - self.last_update_timestamp);
        if time_diff_ms > 0 {
            // update
            self.last_update_timestamp += ms_to_nano(time_diff_ms);
            self.compound(time_diff_ms);
        }
    }

    pub fn available_amount(&self) -> Balance {
        self.supplied.balance + self.reserved - self.borrowed.balance
    }
}

impl Contract {
    pub fn internal_unwrap_asset(&self, token_id: &TokenId) -> Asset {
        self.internal_get_asset(token_id).expect("Asset not found")
    }

    pub fn internal_get_asset(&self, token_id: &TokenId) -> Option<Asset> {
        self.assets.get(token_id).map(|o| {
            let mut asset: Asset = o.into();
            asset.update();
            asset
        })
    }

    pub fn internal_set_asset(&mut self, token_id: &TokenId, asset: Asset) {
        self.assets.insert(token_id, &asset.into());
    }
}

#[near_bindgen]
impl Contract {
    pub fn get_asset(&self, token_id: ValidAccountId) -> Option<Asset> {
        self.internal_get_asset(token_id.as_ref())
    }

    pub fn get_assets(&self, token_ids: Vec<ValidAccountId>) -> Vec<(TokenId, Asset)> {
        token_ids
            .into_iter()
            .filter_map(|token_id| {
                self.internal_get_asset(token_id.as_ref())
                    .map(|asset| (token_id.into(), asset))
            })
            .collect()
    }

    pub fn get_assets_paged(
        &self,
        from_index: Option<u64>,
        limit: Option<u64>,
    ) -> Vec<(TokenId, Asset)> {
        let keys = self.asset_ids.as_vector();
        let from_index = from_index.unwrap_or(0);
        let limit = limit.unwrap_or(keys.len());
        (from_index..std::cmp::min(keys.len(), limit))
            .map(|index| {
                let key = keys.get(index).unwrap();
                let mut asset: Asset = self.assets.get(&key).unwrap().into();
                asset.update();
                (key, asset)
            })
            .collect()
    }

    pub fn debug_get_assets_paged(
        &self,
        from_index: Option<u64>,
        limit: Option<u64>,
    ) -> Vec<(TokenId, Asset)> {
        let keys = self.asset_ids.as_vector();
        let from_index = from_index.unwrap_or(0);
        let limit = limit.unwrap_or(keys.len());
        (from_index..std::cmp::min(keys.len(), limit))
            .map(|index| {
                let key = keys.get(index).unwrap();
                let asset = self.assets.get(&key).unwrap().into();
                (key, asset)
            })
            .collect()
    }
}
