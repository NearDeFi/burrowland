use crate::*;
use std::cmp::Ordering;
use std::io::Write;
use std::ops::{Add, Div, Mul, Sub};

uint::construct_uint!(
    pub struct U256(4);
);

const BIG_DIVISOR: u128 = 10u128.pow(27);
const HALF_DIVISOR: u128 = BIG_DIVISOR / 2;

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

impl BigDecimal {
    pub fn as_u128(&self) -> u128 {
        ((self.0 + U256::from(HALF_DIVISOR)) / U256::from(BIG_DIVISOR)).as_u128()
    }

    pub fn zero() -> Self {
        BigDecimal(U256::zero())
    }

    pub fn one() -> Self {
        BigDecimal(U256::from(BIG_DIVISOR))
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

impl BorshSerialize for BigDecimal {
    fn serialize<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        BorshSerialize::serialize(&self.0 .0, writer)
    }
}

impl BorshDeserialize for BigDecimal {
    fn deserialize(buf: &mut &[u8]) -> std::io::Result<Self> {
        Ok(Self(U256(BorshDeserialize::deserialize(buf)?)))
    }
}
