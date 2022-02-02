use crate::*;

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone, Hash, Eq, PartialEq)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug))]
#[serde(crate = "near_sdk::serde")]
pub enum FarmId {
    Supplied(TokenId),
    Borrowed(TokenId),
}

impl FarmId {
    pub fn get_token_id(&self) -> &TokenId {
        match self {
            FarmId::Supplied(token_id) => token_id,
            FarmId::Borrowed(token_id) => token_id,
        }
    }
}

/// A data required to keep track of a farm for an account.
#[derive(BorshSerialize, BorshDeserialize)]
pub struct AccountFarm {
    pub block_timestamp: Timestamp,
    pub rewards: HashMap<TokenId, AccountFarmReward>,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct AccountFarmReward {
    pub boosted_shares: Balance,
    pub last_reward_per_share: BigDecimal,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub enum VAccountFarm {
    Current(AccountFarm),
}

impl From<VAccountFarm> for AccountFarm {
    fn from(v: VAccountFarm) -> Self {
        match v {
            VAccountFarm::Current(c) => c,
        }
    }
}

impl From<AccountFarm> for VAccountFarm {
    fn from(c: AccountFarm) -> Self {
        VAccountFarm::Current(c)
    }
}

impl Contract {
    pub fn internal_account_farm_claim(
        &self,
        account: &Account,
        farm_id: &FarmId,
        asset_farm: &AssetFarm,
    ) -> (
        AccountFarm,
        Vec<(TokenId, Balance)>,
        Vec<(TokenId, Balance)>,
    ) {
        let mut new_rewards = vec![];
        let mut inactive_rewards = vec![];
        let block_timestamp = env::block_timestamp();
        let mut account_farm: AccountFarm = account
            .farms
            .get(farm_id)
            .map(|v| v.into())
            .unwrap_or_else(|| AccountFarm {
                block_timestamp: 0,
                rewards: HashMap::new(),
            });
        if account_farm.block_timestamp != block_timestamp {
            account_farm.block_timestamp = block_timestamp;
            let mut old_rewards = std::mem::take(&mut account_farm.rewards);
            for (
                token_id,
                AssetFarmReward {
                    reward_per_share, ..
                },
            ) in &asset_farm.rewards
            {
                let boosted_shares = if let Some(AccountFarmReward {
                    boosted_shares,
                    last_reward_per_share,
                }) = old_rewards.remove(token_id)
                {
                    let diff = reward_per_share.clone() - last_reward_per_share;
                    let amount = diff.round_mul_u128(boosted_shares);
                    if amount > 0 {
                        new_rewards.push((token_id.clone(), amount));
                    }
                    boosted_shares
                } else {
                    0
                };
                account_farm.rewards.insert(
                    token_id.clone(),
                    AccountFarmReward {
                        boosted_shares,
                        last_reward_per_share: reward_per_share.clone(),
                    },
                );
            }
            for (
                token_id,
                AccountFarmReward {
                    boosted_shares,
                    last_reward_per_share,
                },
            ) in old_rewards
            {
                let AssetFarmReward {
                    reward_per_share, ..
                } = asset_farm
                    .internal_get_inactive_asset_farm_reward(&token_id)
                    .unwrap();
                let diff = reward_per_share - last_reward_per_share;
                let amount = diff.round_mul_u128(boosted_shares);
                inactive_rewards.push((token_id.clone(), boosted_shares));
                if amount > 0 {
                    new_rewards.push((token_id, amount));
                }
            }
        }
        (account_farm, new_rewards, inactive_rewards)
    }

    pub fn internal_account_apply_affected_farms(
        &mut self,
        account: &mut Account,
        verify_booster: bool,
    ) {
        let config = self.internal_config();
        if verify_booster
            && account
                .affected_farms
                .contains(&FarmId::Supplied(config.booster_token_id.clone()))
        {
            account.add_all_affected_farms();
        }
        if account.affected_farms.is_empty() {
            return;
        }
        let mut all_rewards: HashMap<TokenId, Balance> = HashMap::new();
        let mut i = 0;
        let mut farms = vec![];
        while i < account.affected_farms.len() {
            let farm_id = account.affected_farms[i].clone();
            if let Some(asset_farm) = self.internal_get_asset_farm(&farm_id, false) {
                let (account_farm, new_rewards, inactive_rewards) =
                    self.internal_account_farm_claim(account, &farm_id, &asset_farm);
                for (token_id, amount) in new_rewards {
                    let new_farm_id = FarmId::Supplied(token_id.clone());
                    account.add_affected_farm(new_farm_id);
                    *all_rewards.entry(token_id).or_default() += amount;
                }
                farms.push((farm_id, account_farm, asset_farm, inactive_rewards));
            }
            i += 1;
        }
        for (token_id, &reward) in &all_rewards {
            self.internal_deposit(account, &token_id, reward);
        }
        let booster_balance = self
            .internal_get_asset(&config.booster_token_id)
            .map(|booster| {
                booster
                    .supplied
                    .shares_to_amount(account.get_supplied_shares(&config.booster_token_id), false)
            })
            .unwrap_or(0);
        let booster_base = 10u128.pow(config.booster_decimals as u32);

        for (farm_id, mut account_farm, mut asset_farm, inactive_rewards) in farms {
            let shares = match &farm_id {
                FarmId::Supplied(token_id) => account.get_supplied_shares(token_id).0,
                FarmId::Borrowed(token_id) => account.get_borrowed_shares(token_id).0,
            };
            for (token_id, asset_farm_reward) in asset_farm.rewards.iter_mut() {
                let account_farm_reward = account_farm.rewards.get_mut(token_id).unwrap();
                asset_farm_reward.boosted_shares -= account_farm_reward.boosted_shares;
                if shares > 0 {
                    let extra_shares = if booster_balance > booster_base {
                        let log_base =
                            (asset_farm_reward.booster_log_base as f64) / (booster_base as f64);
                        ((shares as f64)
                            * ((booster_balance as f64) / (booster_base as f64)).log(log_base))
                            as u128
                    } else {
                        0
                    };
                    account_farm_reward.boosted_shares = shares + extra_shares;
                    asset_farm_reward.boosted_shares += account_farm_reward.boosted_shares;
                }
            }
            for (token_id, boosted_shares) in inactive_rewards {
                let mut asset_farm_reward = asset_farm
                    .internal_get_inactive_asset_farm_reward(&token_id)
                    .unwrap();
                asset_farm_reward.boosted_shares -= boosted_shares;
                asset_farm.internal_set_inactive_asset_farm_reward(&token_id, asset_farm_reward);
            }
            account.storage_tracker.start();
            if shares > 0 {
                account.farms.insert(&farm_id, &account_farm.into());
            } else {
                account.farms.remove(&farm_id);
            }
            account.storage_tracker.stop();
            self.internal_set_asset_farm(&farm_id, asset_farm);
        }
    }
}

#[near_bindgen]
impl Contract {
    /// Claims all unclaimed farm rewards.
    pub fn account_farm_claim_all(&mut self) {
        let account_id = env::predecessor_account_id();
        let mut account = self.internal_unwrap_account(&env::predecessor_account_id());
        account.add_all_affected_farms();
        self.internal_account_apply_affected_farms(&mut account, false);
        self.internal_set_account(&account_id, account);
    }
}
