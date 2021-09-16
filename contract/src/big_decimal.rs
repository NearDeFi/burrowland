use crate::*;
use std::cmp::Ordering;
use std::ops::{Add, Div, Mul, Sub};

uint::construct_uint!(
    pub struct U256(4);
);

const BIG_DIVISOR: u128 = 10u128.pow(27);
const HALF_DIVISOR: u128 = BIG_DIVISOR / 2;

#[derive(Copy, Clone, BorshSerialize, BorshDeserialize, PartialEq, PartialOrd)]
pub struct LowU128(u128);

#[derive(Copy, Clone)]
pub struct BigDecimal(U256);

impl From<u128> for BigDecimal {
    fn from(a: u128) -> Self {
        Self(U256::from(a) * U256::from(BIG_DIVISOR))
    }
}

impl From<u64> for BigDecimal {
    fn from(a: u64) -> Self {
        Self(U256::from(a) * U256::from(BIG_DIVISOR))
    }
}

impl From<u32> for BigDecimal {
    fn from(a: u32) -> Self {
        Self(U256::from(a) * U256::from(BIG_DIVISOR))
    }
}

impl Add<BigDecimal> for BigDecimal {
    type Output = Self;

    fn add(self, rhs: BigDecimal) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl Sub<BigDecimal> for BigDecimal {
    type Output = Self;

    fn sub(self, rhs: BigDecimal) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl Mul for BigDecimal {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self((self.0 * rhs.0 + U256::from(HALF_DIVISOR)) / U256::from(BIG_DIVISOR))
    }
}

impl Div for BigDecimal {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        Self((self.0 * U256::from(BIG_DIVISOR) + U256::from(HALF_DIVISOR)) / rhs.0)
    }
}

impl From<LowU128> for BigDecimal {
    fn from(low_u128: LowU128) -> Self {
        Self(U256::from(low_u128.0))
    }
}

impl From<BigDecimal> for LowU128 {
    fn from(bd: BigDecimal) -> Self {
        LowU128(bd.0.low_u128())
    }
}

impl BigDecimal {
    pub fn round_u128(&self) -> u128 {
        ((self.0 + U256::from(HALF_DIVISOR)) / U256::from(BIG_DIVISOR)).as_u128()
    }

    pub fn round_mul_u128(&self, rhs: u128) -> u128 {
        ((self.0 * U256::from(rhs) + U256::from(HALF_DIVISOR)) / U256::from(BIG_DIVISOR)).as_u128()
    }

    pub fn div_u128(&self, rhs: u128) -> BigDecimal {
        Self(self.0 / U256::from(rhs))
    }

    pub fn zero() -> Self {
        Self(U256::zero())
    }

    pub fn one() -> Self {
        Self(U256::from(BIG_DIVISOR))
    }

    pub fn pow(&self, mut exponent: u64) -> Self {
        let mut res = BigDecimal::one();
        let mut x = *self;

        while exponent != 0 {
            if (exponent & 1) != 0 {
                res = res * x;
            }
            exponent >>= 1;
            if exponent != 0 {
                x = x * x;
            }
        }

        res
    }
}

impl PartialEq<Self> for BigDecimal {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl PartialOrd for BigDecimal {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

// impl BorshSerialize for BigDecimal {
//     fn serialize<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
//         BorshSerialize::serialize(&self.0 .0, writer)
//     }
// }
//
// impl BorshDeserialize for BigDecimal {
//     fn deserialize(buf: &mut &[u8]) -> std::io::Result<Self> {
//         Ok(Self(U256(BorshDeserialize::deserialize(buf)?)))
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;
    use rand::RngCore;

    // Number of milliseconds in a regular year.
    const N: u64 = 31536000000;
    // X = 2
    const LOW_X: LowU128 = LowU128(2000000000000000000000000000);
    // R ** N = X. So R = X ** (1/N)
    const LOW_R: LowU128 = LowU128(1000000000021979552909930328);

    fn b(a: u128) -> BigDecimal {
        BigDecimal::from(a)
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
    fn test_simple_add() {
        assert_eq!((b(0) + b(0)).round_u128(), 0);
        assert_eq!((b(5) + b(2)).round_u128(), 7);
        assert_eq!((b(2) + b(5)).round_u128(), 7);
        assert_eq!((b(5) + b(0)).round_u128(), 5);
        assert_eq!((b(0) + b(5)).round_u128(), 5);
    }

    #[test]
    fn test_simple_div() {
        assert_eq!((b(17) / b(5)).round_u128(), 3);
        assert_eq!((b(18) / b(5)).round_u128(), 4);
        assert_eq!((b(3) / b(5)).round_u128(), 1);
    }

    #[test]
    fn test_pow() {
        let r = BigDecimal::from(LOW_R);
        let x = r.pow(N);
        let low_x = LowU128::from(x);
        almost_eq(LOW_X.0, low_x.0, 15);
    }

    #[test]
    fn test_compound_pow() {
        fn test(split_n: u64) {
            let r = BigDecimal::from(LOW_R);
            let initial_val = 12345 * 10u128.pow(24);
            let mut val = initial_val;
            for i in 1..=split_n {
                let exponent = (N * i / split_n) - (N * (i - 1) / split_n);
                let interest = r.pow(exponent);
                val = interest.round_mul_u128(val);
            }
            almost_eq(val, initial_val * 2, 15);
        }

        (1..=100).for_each(test);
    }

    #[test]
    fn test_compound_pow_precision() {
        fn test(split_n: u64) {
            let r = BigDecimal::from(LOW_R);
            let initial_val = 12345 * 10u128.pow(24);
            let mut val = initial_val;
            let exponent = N / split_n;
            assert_eq!(exponent * split_n, N);
            let interest = r.pow(exponent);
            for _ in 1..=split_n {
                val = interest.round_mul_u128(val);
            }
            almost_eq(val, initial_val * 2, 15);
        }
        test(N / 60000);
        test(N / 1000000);
        test(N / (24 * 60 * 60));
    }

    #[test]
    fn test_compound_pow_random() {
        const MAX_STEP: u64 = 1000000;
        let r = BigDecimal::from(LOW_R);
        let initial_val = 12345 * 10u128.pow(24);
        let mut val = initial_val;
        let mut total_exponent = 0;
        let mut rng = rand::thread_rng();
        while total_exponent < N {
            let exponent = std::cmp::min(N - total_exponent, rng.next_u64() % MAX_STEP + 1);
            total_exponent += exponent;
            let interest = r.pow(exponent);
            val = interest.round_mul_u128(val);
        }
        almost_eq(val, initial_val * 2, 15);
    }
}
