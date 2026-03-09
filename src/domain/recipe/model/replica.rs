use std::fmt::{Display, Formatter, Result as FmtResult};
use std::ops::Add;

use getset::CopyGetters;
use snafu::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct Replica {
    value: f64,
}

impl Replica {
    pub fn new(value: f64) -> Result<Self, NewReplicaError> {
        ensure!(value > 0.0, ZeroOrNegativeSnafu);
        Ok(Self { value })
    }

    pub fn one() -> Self {
        Self::new(1.0).unwrap()
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
        Replica::new(self.value + other.value).unwrap()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Snafu)]
pub enum NewReplicaError {
    #[snafu(display("replica should be positive"))]
    ZeroOrNegative,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_replica() {
        let replica = Replica::new(3.5).unwrap();
        assert_eq!(replica.value(), 3.5);
    }

    #[test]
    fn test_non_positive_replica_returns_error() {
        assert!(matches!(
            Replica::new(0.0),
            Err(NewReplicaError::ZeroOrNegative),
        ));
        assert!(matches!(
            Replica::new(-10.0),
            Err(NewReplicaError::ZeroOrNegative),
        ));
    }

    #[test]
    fn test_display() {
        let replica = Replica::new(3.5).unwrap();
        assert_eq!(format!("{}", replica), "3.5 times");
    }

    #[test]
    fn test_add() {
        let r1 = Replica::new(1.5).unwrap();
        let r2 = Replica::new(2.5).unwrap();
        let result = r1 + r2;
        assert_eq!(result, Replica::new(4.0).unwrap());
    }
}
