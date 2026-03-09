use getset::Getters;
use snafu::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Getters)]
#[getset(get = "pub")]
pub struct RecipeId {
    value: String,
}

impl RecipeId {
    pub fn new<S: Into<String>>(value: S) -> Result<Self, NewRecipeIdError> {
        let value = value.into();
        ensure!(!value.is_empty(), EmptySnafu);
        ensure!(
            value
                .chars()
                .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_'),
            InvalidCharacterSnafu,
        );
        Ok(Self {
            value: format!("recipe#{}", value),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Snafu)]
pub enum NewRecipeIdError {
    #[snafu(display("recipe ID should not be empty"))]
    Empty,
    #[snafu(display("recipe ID should only contain alphabets, numbers, hyphens and underscores"))]
    InvalidCharacter,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_recipe_id() {
        let id = RecipeId::new("Recipe-123_abc").unwrap();
        assert_eq!(id.value(), "recipe#Recipe-123_abc");
    }

    #[test]
    fn test_empty_string_returns_error() {
        assert!(matches!(RecipeId::new(""), Err(NewRecipeIdError::Empty)));
    }

    #[test]
    fn test_invalid_characters_return_error() {
        for invalid in ["test id", "abc!", "test.id", "测试"] {
            assert!(matches!(
                RecipeId::new(invalid),
                Err(NewRecipeIdError::InvalidCharacter),
            ));
        }
    }
}
