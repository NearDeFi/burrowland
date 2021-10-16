use crate::*;

static ASSET_FARMS: Lazy<Mutex<HashMap<FarmId, Option<AssetFarm>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

const NANOS_PER_DAY: Duration = 24 * 60 * 60 * 10u64.pow(9);

/// A data required to keep track of a farm for an account.
#[derive(BorshSerialize, BorshDeserialize, Clone, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct AssetFarm {
    #[serde(with = "u64_dec_format")]
    pub block_timestamp: Timestamp,
    pub rewards: Vec<AssetFarmReward>,
}

#[derive(BorshSerialize, BorshDeserialize, Clone, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct AssetFarmReward {
    pub token_id: TokenId,
    #[serde(with = "u128_dec_format")]
    pub reward_per_day: Balance,
    /// Including decimals
    #[serde(with = "u128_dec_format")]
    pub booster_log_base: Balance,

    #[serde(with = "u128_dec_format")]
    pub remaining_rewards: Balance,

    #[serde(with = "u128_dec_format")]
    pub boosted_shares: Balance,
    #[serde(skip)]
    pub reward_per_share: BigDecimal,
}

impl AssetFarm {
    pub fn update(&mut self) {
        let block_timestamp = env::block_timestamp();
        if block_timestamp == self.block_timestamp {
            return;
        }
        let time_diff = block_timestamp - self.block_timestamp;
        self.block_timestamp = block_timestamp;
        for reward in &mut self.rewards {
            if reward.boosted_shares == 0 {
                continue;
            }
            let acquired_rewards = std::cmp::min(
                reward.remaining_rewards,
                u128_ratio(
                    reward.reward_per_day,
                    u128::from(time_diff),
                    u128::from(NANOS_PER_DAY),
                ),
            );
            reward.remaining_rewards -= acquired_rewards;
            reward.reward_per_share = reward.reward_per_share
                + BigDecimal::from(acquired_rewards) / BigDecimal::from(reward.boosted_shares);
        }
    }
}

#[derive(BorshSerialize, BorshDeserialize)]
pub enum VAssetFarm {
    Current(AssetFarm),
}

impl From<VAssetFarm> for AssetFarm {
    fn from(v: VAssetFarm) -> Self {
        match v {
            VAssetFarm::Current(c) => c,
        }
    }
}

impl From<AssetFarm> for VAssetFarm {
    fn from(c: AssetFarm) -> Self {
        VAssetFarm::Current(c)
    }
}

impl Contract {
    pub fn internal_unwrap_asset_farm(&self, farm_id: &FarmId) -> AssetFarm {
        self.internal_get_asset_farm(farm_id)
            .expect("Asset farm not found")
    }

    pub fn internal_get_asset_farm(&self, farm_id: &FarmId) -> Option<AssetFarm> {
        let mut cache = ASSET_FARMS.lock().unwrap();
        cache.get(farm_id).cloned().unwrap_or_else(|| {
            let asset_farm = self.asset_farms.get(farm_id).map(|v| {
                let mut asset_farm: AssetFarm = v.into();
                asset_farm.update();
                asset_farm
            });
            cache.insert(farm_id.clone(), asset_farm.clone());
            asset_farm
        })
    }

    pub fn internal_set_asset_farm(&mut self, farm_id: &FarmId, asset_farm: AssetFarm) {
        ASSET_FARMS
            .lock()
            .unwrap()
            .insert(farm_id.clone(), Some(asset_farm.clone()));
        self.asset_farms.insert(farm_id, &asset_farm.into());
    }
}
