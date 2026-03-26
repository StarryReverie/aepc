use std::collections::HashMap;

use getset::Getters;

use crate::domain::item::model::ItemId;
use crate::domain::plan::model::{PlanItemNode, PlanRecipeNode};
use crate::domain::recipe::model::RecipeId;

#[derive(Debug, Clone, PartialEq, Eq, Getters)]
#[getset(get = "pub")]
pub struct Plan {
    goal: PlanItemNode,
    common_intermediates: HashMap<ItemId, PlanItemNode>,
    common_recipes: HashMap<RecipeId, PlanRecipeNode>,
}

impl Plan {
    pub fn new(
        goal: PlanItemNode,
        common_intermediates: HashMap<ItemId, PlanItemNode>,
        common_recipes: HashMap<RecipeId, PlanRecipeNode>,
    ) -> Self {
        Self {
            goal,
            common_intermediates,
            common_recipes,
        }
    }
}
