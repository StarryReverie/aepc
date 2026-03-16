use std::backtrace::Backtrace;
use std::cmp::Ordering;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::ops::Add;

use getset::CopyGetters;
use snafu::prelude::*;

#[derive(Debug, Clone, Copy, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct Replica {
    value: f64,
}

impl PartialEq for Replica {
    fn eq(&self, other: &Self) -> bool {
        approx::abs_diff_eq!(self.value, other.value, epsilon = 1e-9)
    }
}

impl Eq for Replica {}

impl PartialOrd for Replica {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self == other {
            Some(Ordering::Equal)
        } else {
            self.value.partial_cmp(&other.value)
        }
    }
}

impl Replica {
    pub fn new(value: f64) -> Result<Self, NewReplicaError> {
        ensure!(value > 0.0, ZeroOrNegativeSnafu);
        Ok(Self { value })
    }

    pub fn one() -> Self {
        Self::new(1.0).expect("1.0 should be positive")
    }
}

impl Display for Replica {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{} times", self.value)
    }
}

impl Add for Replica {
    type Output = Replica;

    fn add(self, other: Self) -> Self::Output {
        Replica::new(self.value + other.value)
            .expect("the result should be positive because both operands are positive")
    }
}

#[derive(Debug, Snafu)]
#[non_exhaustive]
pub enum NewReplicaError {
    #[snafu(display("replica should be positive"))]
    ZeroOrNegative { backtrace: Backtrace },
}

#[cfg(test)]
mod tests {
    use anyhow::Result as AnyhowResult;

    use super::*;

    #[test]
    fn test_valid_replica() -> AnyhowResult<()> {
        let replica = Replica::new(3.5)?;
        assert_eq!(replica.value(), 3.5);
        Ok(())
    }

    #[test]
    fn test_non_positive_replica_returns_error() {
        assert!(matches!(
            Replica::new(0.0),
            Err(NewReplicaError::ZeroOrNegative { .. }),
        ));
        assert!(matches!(
            Replica::new(-10.0),
            Err(NewReplicaError::ZeroOrNegative { .. }),
        ));
    }

    #[test]
    fn test_display() -> AnyhowResult<()> {
        assert_eq!(format!("{}", Replica::new(3.5)?), "3.5 times");
        Ok(())
    }

    #[test]
    fn test_add() -> AnyhowResult<()> {
        let r1 = Replica::new(1.5)?;
        let r2 = Replica::new(2.5)?;
        let result = r1 + r2;
        assert_eq!(result, Replica::new(4.0)?);
        Ok(())
    }
}
