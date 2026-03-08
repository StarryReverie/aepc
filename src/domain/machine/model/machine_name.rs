use std::fmt::{Display, Formatter, Result as FmtResult};

use getset::Getters;
use snafu::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Getters)]
#[getset(get = "pub")]
pub struct MachineName {
    value: String,
}

impl MachineName {
    pub fn new<S: Into<String>>(value: S) -> Result<Self, NewMachineNameError> {
        let value = value.into();
        ensure!(!value.is_empty(), EmptySnafu);
        Ok(Self { value })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Snafu)]
pub enum NewMachineNameError {
    #[snafu(display("machine name should not be empty"))]
    Empty,
}

impl Display for MachineName {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_machine_name() {
        let name = MachineName::new("Test Machine 123 测试").unwrap();
        assert_eq!(name.value(), "Test Machine 123 测试");
    }

    #[test]
    fn test_empty_string_returns_error() {
        assert!(matches!(
            MachineName::new(""),
            Err(NewMachineNameError::Empty)
        ));
    }

    #[test]
    fn test_display() {
        let name = MachineName::new("Test Machine").unwrap();
        assert_eq!(format!("{}", name), "Test Machine");
    }
}
