use crate::*;

const MAX_POS: u32 = 10000;
const MAX_RATIO: u32 = 10000;

#[derive(BorshSerialize, BorshDeserialize)]
pub struct AssetConfig {
    // E.g. 25% from borrowed interests goes to the reserve.
    pub reserve_ratio: u32,
    // E.g. 80% of assets are borrowed.
    pub opt_utilization_pos: u32,
    // Rate in the magic rate formula with BD denominator at opt_utilization_pos
    pub opt_utilization_rate: BigDecimal,
    // Rate at 100% utilization
    pub max_utilization_rate: BigDecimal,
    // collateral to loan ratio. 40% means you can borrow 40% of DAI for supplying 100% of NEAR.
    pub collateral_ratio: u32,
}

impl AssetConfig {
    pub fn assert_valid(&self) {
        assert!(self.reserve_ratio <= MAX_RATIO);
        assert!(self.opt_utilization_pos < MAX_POS);
        assert!(self.opt_utilization_rate <= self.max_utilization_rate);
    }

    pub fn get_rate(
        &self,
        borrowed_balance: Balance,
        total_supplied_balance: Balance,
    ) -> BigDecimal {
        if total_supplied_balance == 0 {
            BigDecimal::zero()
        } else {
            let pos = BigDecimal::from(borrowed_balance) / BigDecimal::from(total_supplied_balance);
            let opt_util_pos =
                BigDecimal::from(self.opt_utilization_pos) / BigDecimal::from(MAX_POS);
            if pos < opt_util_pos {
                pos * self.opt_utilization_rate / opt_util_pos
            } else {
                self.opt_utilization_rate
                    + (pos - opt_util_pos) * (self.max_utilization_rate - self.opt_utilization_rate)
                        / BigDecimal::from(MAX_POS - self.opt_utilization_pos)
            }
        }
    }
}
