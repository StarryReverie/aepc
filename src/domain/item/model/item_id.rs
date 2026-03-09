use getset::Getters;
use snafu::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Getters)]
#[getset(get = "pub")]
pub struct ItemId {
    value: String,
}

impl ItemId {
    pub fn new<S: Into<String>>(value: S) -> Result<Self, NewItemIdError> {
        let value = value.into();
        ensure!(!value.is_empty(), EmptySnafu);
        ensure!(
            value
                .chars()
                .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_'),
            InvalidCharacterSnafu,
        );
        Ok(Self {
            value: format!("item#{}", value),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Snafu)]
pub enum NewItemIdError {
    #[snafu(display("item ID should not be empty"))]
    Empty,
    #[snafu(display("item ID should only contain alphabets, numbers, hyphens and underscores"))]
    InvalidCharacter,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_item_id() {
        let id = ItemId::new("Item-123_abc").unwrap();
        assert_eq!(id.value(), "item#Item-123_abc");
    }

    #[test]
    fn test_empty_string_returns_error() {
        assert!(matches!(ItemId::new(""), Err(NewItemIdError::Empty)));
    }

    #[test]
    fn test_invalid_characters_return_error() {
        for invalid in ["test id", "abc!", "test.id", "测试"] {
            assert!(matches!(
                ItemId::new(invalid),
                Err(NewItemIdError::InvalidCharacter),
            ));
        }
    }
}
