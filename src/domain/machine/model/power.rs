use std::backtrace::Backtrace;
use std::cmp::Ordering;
use std::fmt::{Display, Formatter, Result as FmtResult};

use getset::CopyGetters;
use snafu::prelude::*;

#[derive(Debug, Clone, Copy, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct Power {
    value: f64,
}

impl PartialEq for Power {
    fn eq(&self, other: &Self) -> bool {
        approx::abs_diff_eq!(self.value, other.value, epsilon = 1e-9)
    }
}

impl Eq for Power {}

impl PartialOrd for Power {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self == other {
            Some(Ordering::Equal)
        } else {
            self.value.partial_cmp(&other.value)
        }
    }
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

#[derive(Debug, Snafu)]
#[non_exhaustive]
pub enum NewPowerError {
    #[snafu(display("power should not be negative"))]
    Negative { backtrace: Backtrace },
}

#[cfg(test)]
mod tests {
    use anyhow::Result as AnyhowResult;

    use super::*;

    #[test]
    fn test_valid_power() -> AnyhowResult<()> {
        assert_eq!(Power::new(123.45)?.value(), 123.45);
        assert_eq!(Power::new(0.0)?.value(), 0.0);
        Ok(())
    }

    #[test]
    fn test_negative_power_returns_error() {
        assert!(matches!(
            Power::new(-10.0),
            Err(NewPowerError::Negative { .. })
        ));
    }

    #[test]
    fn test_display() -> AnyhowResult<()> {
        assert_eq!(format!("{}", Power::new(123.45)?), "123.45 kW");
        Ok(())
    }
}
