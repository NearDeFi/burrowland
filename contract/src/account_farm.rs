use crate::*;

#[derive(BorshSerialize, BorshDeserialize, Serialize, Clone, Hash, Eq, PartialEq)]
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
    pub boosted_shares: Balance,
    pub last_reward_per_share: Vec<BigDecimal>,
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
        &mut self,
        account: &mut Account,
        farm_id: &FarmId,
        asset_farm: &AssetFarm,
    ) -> (AccountFarm, Vec<(TokenId, Balance)>) {
        let mut new_rewards = Vec::new();
        let block_timestamp = env::block_timestamp();
        let mut account_farm: AccountFarm = account
            .farms
            .get(farm_id)
            .map(|v| v.into())
            .unwrap_or_else(|| AccountFarm {
                block_timestamp,
                boosted_shares: 0,
                last_reward_per_share: vec![],
            });
        if account_farm.block_timestamp != block_timestamp {
            account_farm.block_timestamp = block_timestamp;
            for (
                i,
                Reward {
                    token_id,
                    reward_per_share,
                    ..
                },
            ) in asset_farm.rewards.iter().enumerate()
            {
                if let Some(last_reward_per_share) = account_farm.last_reward_per_share.get_mut(i) {
                    let diff = reward_per_share.clone() - last_reward_per_share.clone();
                    *last_reward_per_share = reward_per_share.clone();
                    let amount = diff.round_mul_u128(account_farm.boosted_shares);
                    if amount > 0 {
                        new_rewards.push((token_id.clone(), amount));
                    }
                } else {
                    account_farm
                        .last_reward_per_share
                        .push(reward_per_share.clone());
                }
            }
        }
        (account_farm, new_rewards)
    }

    pub fn internal_account_farm_claim_all(&mut self, account: &mut Account) {
        assert!(account.affected_farms.is_empty());
        account.affected_farms.extend(account.farms.keys());
        self.internal_account_apply_affected_farms(account);
    }

    pub fn internal_account_apply_affected_farms(&mut self, account: &mut Account) {
        let mut all_rewards: HashMap<TokenId, Balance> = HashMap::new();
        let mut i = 0;
        let mut farms = vec![];
        while i < account.affected_farms.len() {
            let farm_id = account.affected_farms[i].clone();
            if let Some(asset_farm) = self.internal_get_asset_farm(&farm_id) {
                let (account_farm, new_rewards) =
                    self.internal_account_farm_claim(account, &farm_id, &asset_farm);
                for (token_id, amount) in new_rewards {
                    let new_farm_id = FarmId::Supplied(token_id.clone());
                    account.add_affected_farm(new_farm_id);
                    *all_rewards.entry(token_id).or_default() += amount;
                }
                farms.push((farm_id, account_farm, asset_farm));
            }
            i += 1;
        }
        for (token_id, &reward) in &all_rewards {
            self.internal_deposit(account, &token_id, reward);
        }
        // TODO: Compute booster
        let booster = account.compute_booster();
        for (farm_id, mut account_farm, mut asset_farm) in farms {
            asset_farm.boosted_shares -= account_farm.boosted_shares;
            match &farm_id {
                FarmId::Supplied(token_id) => {
                    account_farm.boosted_shares =
                        booster.round_mul_u128(account.get_supplied_shares(token_id));
                }
                FarmId::Borrowed(token_id) => {
                    account_farm.boosted_shares =
                        booster.round_mul_u128(account.get_borrowed_shares(token_id));
                }
            }
            asset_farm.boosted_shares += account_farm.boosted_shares;
            account.farms.insert(&farm_id, &account_farm.into());
            self.internal_set_asset_farm(&farm_id, asset_farm);
        }
    }
}
