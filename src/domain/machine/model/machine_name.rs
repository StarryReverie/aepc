use std::backtrace::Backtrace;
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

#[derive(Debug, Snafu)]
#[non_exhaustive]
pub enum NewMachineNameError {
    #[snafu(display("machine name should not be empty"))]
    Empty { backtrace: Backtrace },
}

impl Display for MachineName {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.value)
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result as AnyhowResult;

    use super::*;

    #[test]
    fn test_valid_machine_name() -> AnyhowResult<()> {
        let name = MachineName::new("Test Machine 123 测试")?;
        assert_eq!(name.value(), "Test Machine 123 测试");
        Ok(())
    }

    #[test]
    fn test_empty_string_returns_error() {
        assert!(matches!(
            MachineName::new(""),
            Err(NewMachineNameError::Empty { .. })
        ));
    }

    #[test]
    fn test_display() -> AnyhowResult<()> {
        assert_eq!(
            format!("{}", MachineName::new("Test Machine")?),
            "Test Machine"
        );
        Ok(())
    }
}
