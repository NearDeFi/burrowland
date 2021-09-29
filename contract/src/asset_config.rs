use crate::*;

const MAX_POS: u32 = 10000;
const MAX_RATIO: u32 = 10000;

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct AssetConfig {
    // E.g. 25% from borrowed interests goes to the reserve.
    pub reserve_ratio: u32,
    // E.g. 80% of assets are borrowed.
    pub target_utilization: u32,
    // Rate in the magic rate formula with BigDecimal denominator at opt_utilization_pos
    pub target_utilization_rate: LowU128,
    // Rate at 100% utilization
    pub max_utilization_rate: LowU128,
    // Volatility ratio.
    // E.g. 40% for NEAR and 90% for DAI means you can borrow 40% * 90% of DAI for supplying NEAR.
    pub volatility_ratio: u32,
}

// "wrap.near" example. 25% reserve, 80% target utilization, 12% target APR, 250% max APR, 60% vol
// {
// "reserve_ratio": 2500,
// "target_utilization": 8000,
// "target_utilization_rate": "1000000000003593629036885046",
// "max_utilization_rate": "1000000000039724853136740579",
// "volatility_ratio": 6000
// }

impl AssetConfig {
    pub fn assert_valid(&self) {
        assert!(self.reserve_ratio <= MAX_RATIO);
        assert!(self.target_utilization < MAX_POS);
        assert!(self.target_utilization_rate.0 <= self.max_utilization_rate.0);
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
            let target_utilization = BigDecimal::from_ratio(self.target_utilization);
            if pos < target_utilization {
                BigDecimal::one()
                    + pos * (BigDecimal::from(self.target_utilization_rate) - BigDecimal::one())
                        / target_utilization
            } else {
                BigDecimal::from(self.target_utilization_rate)
                    + (pos - target_utilization)
                        * (BigDecimal::from(self.max_utilization_rate)
                            - BigDecimal::from(self.target_utilization_rate))
                        / BigDecimal::from(MAX_POS - self.target_utilization)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ONE_NEAR: u128 = 10u128.pow(24);

    fn test_config() -> AssetConfig {
        AssetConfig {
            reserve_ratio: 2500,
            target_utilization: 8000,
            target_utilization_rate: 1000000000003593629036885046u128.into(),
            max_utilization_rate: 1000000000039724853136740579u128.into(),
            volatility_ratio: 6000,
        }
    }

    fn almost_eq(a: u128, b: u128, prec: u32) {
        let p = 10u128.pow(27 - prec);
        let ap = (a + p / 2) / p;
        let bp = (b + p / 2) / p;
        assert_eq!(
            ap,
            bp,
            "{}",
            format!("Expected {} to eq {}, with precision {}", a, b, prec)
        );
    }

    #[test]
    fn test_get_rate() {
        let config = test_config();
        let rate = config.get_rate(3 * ONE_NEAR, 18 * ONE_NEAR);
        println!("{}", rate)
    }
}
