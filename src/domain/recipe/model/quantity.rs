use std::cmp::Ordering;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::ops::{Add, Div};

use getset::CopyGetters;
use snafu::prelude::*;

use super::{Flow, Period};

#[derive(Debug, Clone, Copy, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct Quantity {
    value: f64,
}

impl PartialEq for Quantity {
    fn eq(&self, other: &Self) -> bool {
        approx::abs_diff_eq!(self.value, other.value, epsilon = 1e-9)
    }
}

impl Eq for Quantity {}

impl PartialOrd for Quantity {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self == other {
            Some(Ordering::Equal)
        } else {
            self.value.partial_cmp(&other.value)
        }
    }
}

impl Quantity {
    pub fn new(value: f64) -> Result<Self, NewQuantityError> {
        ensure!(value >= 0.0, NegativeSnafu);
        Ok(Self { value })
    }
}

impl Display for Quantity {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{} items", self.value)
    }
}

impl Add for Quantity {
    type Output = Quantity;

    fn add(self, other: Self) -> Self::Output {
        Quantity::new(self.value + other.value)
            .expect("the result should be non-negative because both operands are non-negative")
    }
}

impl Div<Period> for Quantity {
    type Output = Flow;

    fn div(self, other: Period) -> Self::Output {
        const SECONDS_PER_MINUTE: f64 = 60.0;
        Flow::new(self.value / other.value() * SECONDS_PER_MINUTE)
            .expect("the result should be non-negative because both operands are non-negative")
    }
}

impl Div<Flow> for Quantity {
    type Output = Period;

    fn div(self, other: Flow) -> Self::Output {
        const SECONDS_PER_MINUTE: f64 = 60.0;
        Period::new(self.value / other.value() * SECONDS_PER_MINUTE)
            .expect("the result should be non-negative because both operands are non-negative")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Snafu)]
pub enum NewQuantityError {
    #[snafu(display("quantity should not be negative"))]
    Negative,
}

#[cfg(test)]
mod tests {
    use anyhow::Result as AnyhowResult;

    use super::*;

    #[test]
    fn test_valid_quantity() -> AnyhowResult<()> {
        assert_eq!(Quantity::new(123.45)?.value(), 123.45);
        assert_eq!(Quantity::new(0.0)?.value(), 0.0);
        Ok(())
    }

    #[test]
    fn test_negative_quantity_returns_error() {
        assert!(matches!(
            Quantity::new(-10.0),
            Err(NewQuantityError::Negative),
        ));
    }

    #[test]
    fn test_display() -> AnyhowResult<()> {
        assert_eq!(format!("{}", Quantity::new(123.45)?), "123.45 items");
        Ok(())
    }

    #[test]
    fn test_add() -> AnyhowResult<()> {
        let q1 = Quantity::new(10.0)?;
        let q2 = Quantity::new(20.0)?;
        assert_eq!(q1 + q2, Quantity::new(30.0)?);
        Ok(())
    }

    #[test]
    fn test_div_period() -> AnyhowResult<()> {
        let result = Quantity::new(30.0)? / Period::new(30.0)?;
        assert_eq!(result, Flow::new(60.0)?);
        Ok(())
    }

    #[test]
    fn test_div_flow() -> AnyhowResult<()> {
        let result = Quantity::new(120.0)? / Flow::new(60.0)?;
        assert_eq!(result, Period::new(120.0)?);
        Ok(())
    }
}
