use std::fmt::{Display, Formatter, Result as FmtResult};

use getset::CopyGetters;
use snafu::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct Period {
    value: f64,
}

impl Period {
    pub fn new(value: f64) -> Result<Self, NewPeriodError> {
        ensure!(value > 0.0, ZeroOrNegativeSnafu);
        Ok(Self { value })
    }
}

impl Display for Period {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{} s", self.value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Snafu)]
pub enum NewPeriodError {
    #[snafu(display("period should be positive"))]
    ZeroOrNegative,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_period() {
        let period = Period::new(123.45).unwrap();
        assert_eq!(period.value(), 123.45);
    }

    #[test]
    fn test_non_positive_period_returns_error() {
        assert!(matches!(
            Period::new(0.0),
            Err(NewPeriodError::ZeroOrNegative),
        ));
        assert!(matches!(
            Period::new(-10.0),
            Err(NewPeriodError::ZeroOrNegative),
        ));
    }

    #[test]
    fn test_display() {
        let period = Period::new(123.45).unwrap();
        assert_eq!(format!("{}", period), "123.45 s");
    }
}
