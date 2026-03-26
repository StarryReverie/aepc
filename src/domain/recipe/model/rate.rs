use std::backtrace::Backtrace;
use std::cmp::Ordering;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::ops::{Add, Mul};

use snafu::prelude::*;

use super::{Flow, Replica};

#[derive(Debug, Clone, Copy)]
pub struct Rate(f64);

impl Rate {
    pub fn new(value: f64) -> Result<Self, NewRateError> {
        ensure!(
            value > 0.0 || approx::relative_eq!(value, 0.0, epsilon = 1e-9),
            NegativeSnafu,
        );
        Ok(Self(value))
    }

    pub fn value(&self) -> f64 {
        self.0
    }
}

impl PartialEq for Rate {
    fn eq(&self, other: &Self) -> bool {
        approx::relative_eq!(self.value(), other.value(), epsilon = 1e-9)
    }
}

impl Eq for Rate {}

impl PartialOrd for Rate {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self == other {
            Some(Ordering::Equal)
        } else {
            self.value().partial_cmp(&other.value())
        }
    }
}

impl Display for Rate {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(
            f,
            "{} items/(min*times)",
            ((self.value() * 100.0).round()) / 100.0
        )
    }
}

impl Add for Rate {
    type Output = Rate;

    fn add(self, other: Self) -> Self::Output {
        Rate::new(self.value() + other.value())
            .expect("the result should be non-negative because both operands are non-negative")
    }
}

impl Mul<Replica> for Rate {
    type Output = Flow;

    fn mul(self, other: Replica) -> Self::Output {
        Flow::new(self.value() * other.value())
            .expect("the result should be non-negative because both operands are non-negative")
    }
}

impl Mul<Rate> for Replica {
    type Output = Flow;

    fn mul(self, other: Rate) -> Self::Output {
        Flow::new(self.value() * other.value())
            .expect("the result should be non-negative because both operands are non-negative")
    }
}

#[derive(Debug, Snafu)]
#[non_exhaustive]
pub enum NewRateError {
    #[snafu(display("rate should not be negative"))]
    Negative { backtrace: Backtrace },
}

#[cfg(test)]
mod tests {
    use anyhow::Result as AnyhowResult;

    use super::*;

    #[test]
    fn test_valid_rate() -> AnyhowResult<()> {
        assert_eq!(Rate::new(123.45)?.value(), 123.45);
        assert_eq!(Rate::new(0.0)?.value(), 0.0);
        Ok(())
    }

    #[test]
    fn test_negative_rate_returns_error() {
        assert!(matches!(
            Rate::new(-10.0),
            Err(NewRateError::Negative { .. })
        ));
    }

    #[test]
    fn test_display() -> AnyhowResult<()> {
        assert_eq!(
            format!("{}", Rate::new(123.45)?),
            "123.45 items/(min*times)"
        );
        Ok(())
    }

    #[test]
    fn test_add() -> AnyhowResult<()> {
        let r1 = Rate::new(10.0)?;
        let r2 = Rate::new(20.0)?;
        assert_eq!(r1 + r2, Rate::new(30.0)?);
        Ok(())
    }

    #[test]
    fn test_mul_replica() -> AnyhowResult<()> {
        let result = Rate::new(40.0)? * Replica::new(3.0)?;
        assert_eq!(result, Flow::new(120.0)?);
        Ok(())
    }

    #[test]
    fn test_replica_mul_rate() -> AnyhowResult<()> {
        let result = Replica::new(3.0)? * Rate::new(40.0)?;
        assert_eq!(result, Flow::new(120.0)?);
        Ok(())
    }
}
