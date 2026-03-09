use std::fmt::{Display, Formatter, Result as FmtResult};
use std::ops::{Add, Div, Mul};

use getset::CopyGetters;
use snafu::prelude::*;

use super::{Period, Quantity, Rate, Replica};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct Flow {
    value: f64,
}

impl Flow {
    pub fn new(value: f64) -> Result<Self, NewFlowError> {
        ensure!(value >= 0.0, NegativeSnafu);
        Ok(Self { value })
    }
}

impl Display for Flow {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{} items/min", self.value)
    }
}

impl Add for Flow {
    type Output = Flow;

    fn add(self, other: Self) -> Self::Output {
        Flow::new(self.value + other.value).unwrap()
    }
}

impl Div<Replica> for Flow {
    type Output = Rate;

    fn div(self, other: Replica) -> Self::Output {
        Rate::new(self.value / other.value()).unwrap()
    }
}

impl Div<Rate> for Flow {
    type Output = Replica;

    fn div(self, other: Rate) -> Self::Output {
        Replica::new(self.value / other.value()).unwrap()
    }
}

impl Mul<Period> for Flow {
    type Output = Quantity;

    fn mul(self, other: Period) -> Self::Output {
        const SECONDS_PER_MINUTE: f64 = 60.0;
        Quantity::new(self.value * other.value() / SECONDS_PER_MINUTE).unwrap()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Snafu)]
pub enum NewFlowError {
    #[snafu(display("flow should not be negative"))]
    Negative,
}

#[cfg(test)]
mod tests {
    use anyhow::Result as AnyhowResult;

    use super::*;

    #[test]
    fn test_valid_flow() -> AnyhowResult<()> {
        assert_eq!(Flow::new(123.45)?.value(), 123.45);
        assert_eq!(Flow::new(0.0)?.value(), 0.0);
        Ok(())
    }

    #[test]
    fn test_negative_flow_returns_error() {
        assert!(matches!(Flow::new(-10.0), Err(NewFlowError::Negative)));
    }

    #[test]
    fn test_display() -> AnyhowResult<()> {
        assert_eq!(format!("{}", Flow::new(123.45)?), "123.45 items/min");
        Ok(())
    }

    #[test]
    fn test_add() -> AnyhowResult<()> {
        let result = Flow::new(10.0)? + Flow::new(20.0)?;
        assert_eq!(result, Flow::new(30.0)?);
        Ok(())
    }

    #[test]
    fn test_div_replica() -> AnyhowResult<()> {
        let result = Flow::new(120.0)? / Replica::new(3.0)?;
        assert_eq!(result, Rate::new(40.0)?);
        Ok(())
    }

    #[test]
    fn test_div_rate() -> AnyhowResult<()> {
        let result = Flow::new(120.0)? / Rate::new(40.0)?;
        assert_eq!(result, Replica::new(3.0)?);
        Ok(())
    }

    #[test]
    fn test_mul_period() -> AnyhowResult<()> {
        let result = Flow::new(60.0)? * Period::new(30.0)?;
        assert_eq!(result, Quantity::new(30.0)?);
        Ok(())
    }
}
