use crate::*;

const MAX_POS: u32 = 10000;
const MAX_RATIO: u32 = 10000;

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct AssetConfig {
    // E.g. 25% from borrowed interests goes to the reserve.
    pub reserve_ratio: u32,
    // E.g. 80% of assets are borrowed.
    pub opt_utilization_pos: u32,
    // Rate in the magic rate formula with BigDecimal denominator at opt_utilization_pos
    pub opt_utilization_rate: LowU128,
    // Rate at 100% utilization
    pub max_utilization_rate: LowU128,
    // Volatility ratio.
    // E.g. 40% for NEAR and 90% for DAI means you can borrow 40% * 90% of DAI for supplying NEAR.
    pub volatility_ratio: u32,
}

impl AssetConfig {
    pub fn assert_valid(&self) {
        assert!(self.reserve_ratio <= MAX_RATIO);
        assert!(self.opt_utilization_pos < MAX_POS);
        assert!(self.opt_utilization_rate.0 <= self.max_utilization_rate.0);
    }

    pub fn get_rate(
        &self,
        borrowed_balance: Balance,
        total_supplied_balance: Balance,
    ) -> BigDecimal {
        if total_supplied_balance == 0 {
            BigDecimal::one()
        } else {
            // Fix overflow
            let pos = BigDecimal::from(borrowed_balance).div_u128(total_supplied_balance);
            let opt_util_pos =
                BigDecimal::from(self.opt_utilization_pos) / BigDecimal::from(MAX_POS);
            if pos < opt_util_pos {
                BigDecimal::one() + pos * BigDecimal::from(self.opt_utilization_rate) / opt_util_pos
            } else {
                BigDecimal::from(self.opt_utilization_rate)
                    + (pos - opt_util_pos)
                        * (BigDecimal::from(self.max_utilization_rate)
                            - BigDecimal::from(self.opt_utilization_rate))
                        / BigDecimal::from(MAX_POS - self.opt_utilization_pos)
            }
        }
    }
}
