use std::fmt::{Display, Formatter, Result as FmtResult};
use std::ops::{Add, Div};

use getset::CopyGetters;
use snafu::prelude::*;

use super::{Flow, Period};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct Quantity {
    value: f64,
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
        Quantity::new(self.value + other.value).unwrap()
    }
}

impl Div<Period> for Quantity {
    type Output = Flow;

    fn div(self, other: Period) -> Self::Output {
        const SECONDS_PER_MINUTE: f64 = 60.0;
        Flow::new(self.value / other.value() * SECONDS_PER_MINUTE).unwrap()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Snafu)]
pub enum NewQuantityError {
    #[snafu(display("quantity should not be negative"))]
    Negative,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_quantity() {
        let quantity = Quantity::new(123.45).unwrap();
        assert_eq!(quantity.value(), 123.45);

        let zero = Quantity::new(0.0).unwrap();
        assert_eq!(zero.value(), 0.0);
    }

    #[test]
    fn test_negative_quantity_returns_error() {
        assert!(matches!(
            Quantity::new(-10.0),
            Err(NewQuantityError::Negative),
        ));
    }

    #[test]
    fn test_display() {
        let quantity = Quantity::new(123.45).unwrap();
        assert_eq!(format!("{}", quantity), "123.45 items");
    }

    #[test]
    fn test_add() {
        let q1 = Quantity::new(10.0).unwrap();
        let q2 = Quantity::new(20.0).unwrap();
        let result = q1 + q2;
        assert_eq!(result.value(), 30.0);
    }

    #[test]
    fn test_div_period() {
        let quantity = Quantity::new(30.0).unwrap();
        let period = Period::new(30.0).unwrap();
        let result = quantity / period;
        assert_eq!(result.value(), 60.0);
    }
}
