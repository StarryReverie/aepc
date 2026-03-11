mod recipe_repository;

pub use recipe_repository::{DynRecipeRepository, RecipeRepository};

#[cfg(test)]
pub mod test_helper {
    use super::*;

    pub use recipe_repository::RecipeRepositoryMock;
}
