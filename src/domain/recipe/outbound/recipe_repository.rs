use anyhow::Result as AnyhowResult;

use crate::domain::item::model::ItemId;
use crate::domain::recipe::model::{Recipe, RecipeId};

#[unimock::unimock(api = RecipeRepositoryMock)]
#[dynosaur::dynosaur(pub DynRecipeRepository = dyn(box) RecipeRepository)]
pub trait RecipeRepository: Send + Sync {
    async fn get(&self, recipe_id: &RecipeId) -> AnyhowResult<Option<Recipe>>;
    async fn find_containing_product(&self, product_id: &ItemId) -> AnyhowResult<Vec<Recipe>>;
}
