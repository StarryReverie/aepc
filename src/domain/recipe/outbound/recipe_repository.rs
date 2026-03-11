use anyhow::Result as AnyhowResult;

use crate::domain::item::model::ItemId;
use crate::domain::recipe::model::{Recipe, RecipeId};

#[unimock::unimock(api = RecipeRepositoryMock)]
#[dynosaur::dynosaur(pub DynRecipeRepository = dyn(box) RecipeRepository)]
pub trait RecipeRepository: Send + Sync {
    fn get(
        &self,
        recipe_id: &RecipeId,
    ) -> impl Future<Output = AnyhowResult<Option<Recipe>>> + Send;

    fn find_containing_product(
        &self,
        product_id: &ItemId,
    ) -> impl Future<Output = AnyhowResult<Vec<Recipe>>> + Send;
}
