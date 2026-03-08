use std::fmt::{Display, Formatter, Result as FmtResult};
use std::ops::{Add, Mul};

use getset::CopyGetters;
use snafu::prelude::*;

use super::{Flow, Replica};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct Rate {
    value: f64,
}

impl Rate {
    pub fn new(value: f64) -> Result<Self, NewRateError> {
        ensure!(value >= 0.0, NegativeSnafu);
        Ok(Self { value })
    }
}

impl Display for Rate {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{} items/(min*times)", self.value)
    }
}

impl Add for Rate {
    type Output = Rate;

    fn add(self, other: Self) -> Self::Output {
        Rate::new(self.value + other.value).unwrap()
    }
}

impl Mul<Replica> for Rate {
    type Output = Flow;

    fn mul(self, other: Replica) -> Self::Output {
        Flow::new(self.value * other.value()).unwrap()
    }
}

impl Mul<Rate> for Replica {
    type Output = Flow;

    fn mul(self, other: Rate) -> Self::Output {
        Flow::new(self.value() * other.value).unwrap()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Snafu)]
pub enum NewRateError {
    #[snafu(display("rate should not be negative"))]
    Negative,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_rate() {
        let rate = Rate::new(123.45).unwrap();
        assert_eq!(rate.value(), 123.45);

        let zero = Rate::new(0.0).unwrap();
        assert_eq!(zero.value(), 0.0);
    }

    #[test]
    fn test_negative_rate_returns_error() {
        assert!(matches!(Rate::new(-10.0), Err(NewRateError::Negative)));
    }

    #[test]
    fn test_display() {
        let rate = Rate::new(123.45).unwrap();
        assert_eq!(format!("{}", rate), "123.45 items/(min*times)");
    }

    #[test]
    fn test_add() {
        let r1 = Rate::new(10.0).unwrap();
        let r2 = Rate::new(20.0).unwrap();
        let result = r1 + r2;
        assert_eq!(result.value(), 30.0);
    }

    #[test]
    fn test_mul_replica() {
        let rate = Rate::new(40.0).unwrap();
        let replica = Replica::new(3.0).unwrap();
        let result = rate * replica;
        assert_eq!(result.value(), 120.0);
    }

    #[test]
    fn test_replica_mul_rate() {
        let replica = Replica::new(3.0).unwrap();
        let rate = Rate::new(40.0).unwrap();
        let result = replica * rate;
        assert_eq!(result.value(), 120.0);
    }
}
