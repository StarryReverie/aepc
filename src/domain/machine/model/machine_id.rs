use std::backtrace::Backtrace;

use snafu::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MachineId(String);

impl MachineId {
    pub fn new<S: Into<String>>(value: S) -> Result<Self, NewMachineIdError> {
        let value = value.into();
        ensure!(!value.is_empty(), EmptySnafu);
        ensure!(
            value
                .chars()
                .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_'),
            InvalidCharacterSnafu,
        );
        Ok(Self(format!("machine#{}", value)))
    }

    pub fn value(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Snafu)]
#[non_exhaustive]
pub enum NewMachineIdError {
    #[snafu(display("machine ID should not be empty"))]
    Empty { backtrace: Backtrace },
    #[snafu(display(
        "machine ID should only contain alphabets, numbers, hyphens and underscores"
    ))]
    InvalidCharacter { backtrace: Backtrace },
}

#[cfg(test)]
mod tests {
    use anyhow::Result as AnyhowResult;

    use super::*;

    #[test]
    fn test_valid_machine_id() -> AnyhowResult<()> {
        let id = MachineId::new("Machine-123_abc")?;
        assert_eq!(id.value(), "machine#Machine-123_abc");
        Ok(())
    }

    #[test]
    fn test_empty_string_returns_error() {
        assert!(matches!(
            MachineId::new(""),
            Err(NewMachineIdError::Empty { .. })
        ));
    }

    #[test]
    fn test_invalid_characters_return_error() {
        for invalid in ["test id", "abc!", "test.id", "测试"] {
            assert!(matches!(
                MachineId::new(invalid),
                Err(NewMachineIdError::InvalidCharacter { .. }),
            ));
        }
    }
}
