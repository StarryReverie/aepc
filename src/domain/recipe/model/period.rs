use std::cmp::Ordering;
use std::fmt::{Display, Formatter, Result as FmtResult};

use getset::CopyGetters;
use snafu::prelude::*;

#[derive(Debug, Clone, Copy, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct Period {
    value: f64,
}

impl PartialEq for Period {
    fn eq(&self, other: &Self) -> bool {
        approx::abs_diff_eq!(self.value, other.value, epsilon = 1e-9)
    }
}

impl Eq for Period {}

impl PartialOrd for Period {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self == other {
            Some(Ordering::Equal)
        } else {
            self.value.partial_cmp(&other.value)
        }
    }
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
#[non_exhaustive]
pub enum NewPeriodError {
    #[snafu(display("period should be positive"))]
    ZeroOrNegative,
}

#[cfg(test)]
mod tests {
    use anyhow::Result as AnyhowResult;

    use super::*;

    #[test]
    fn test_valid_period() -> AnyhowResult<()> {
        let period = Period::new(123.45)?;
        assert_eq!(period.value(), 123.45);
        Ok(())
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
    fn test_display() -> AnyhowResult<()> {
        assert_eq!(format!("{}", Period::new(123.45)?), "123.45 s");
        Ok(())
    }
}
