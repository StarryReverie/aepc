use std::fmt::{Display, Formatter, Result as FmtResult};

use getset::CopyGetters;
use snafu::prelude::*;

#[derive(Debug, Clone, PartialEq, PartialOrd, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct Power {
    value: f64,
}

impl Power {
    pub fn new(value: f64) -> Result<Self, NewPowerError> {
        ensure!(value >= 0.0, NegativeSnafu);
        Ok(Self { value })
    }
}

impl Display for Power {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{} kW", self.value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Snafu)]
pub enum NewPowerError {
    #[snafu(display("power should not be negative"))]
    Negative,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_power() {
        let power = Power::new(123.45).unwrap();
        assert_eq!(power.value(), 123.45);

        let power = Power::new(0.0).unwrap();
        assert_eq!(power.value(), 0.0);
    }

    #[test]
    fn test_negative_power_returns_error() {
        assert!(matches!(Power::new(-10.0), Err(NewPowerError::Negative)));
    }

    #[test]
    fn test_display() {
        let power = Power::new(123.45).unwrap();
        assert_eq!(format!("{}", power), "123.45 kW");
    }
}
