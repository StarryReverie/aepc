use std::fmt::{Display, Formatter, Result as FmtResult};

use getset::Getters;
use snafu::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Getters)]
#[getset(get = "pub")]
pub struct ItemName {
    value: String,
}

impl ItemName {
    pub fn new<S: Into<String>>(value: S) -> Result<Self, NewItemNameError> {
        let value = value.into();
        ensure!(!value.is_empty(), EmptySnafu);
        Ok(Self { value })
    }
}

impl Display for ItemName {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Snafu)]
pub enum NewItemNameError {
    #[snafu(display("item name should not be empty"))]
    Empty,
}

#[cfg(test)]
mod tests {
    use anyhow::Result as AnyhowResult;

    use super::*;

    #[test]
    fn test_valid_item_name() -> AnyhowResult<()> {
        let name = ItemName::new("Test Item 123 测试")?;
        assert_eq!(name.value(), "Test Item 123 测试");
        Ok(())
    }

    #[test]
    fn test_empty_string_returns_error() {
        assert!(matches!(ItemName::new(""), Err(NewItemNameError::Empty)));
    }

    #[test]
    fn test_display() -> AnyhowResult<()> {
        assert_eq!(format!("{}", ItemName::new("Test Item")?), "Test Item");
        Ok(())
    }
}
