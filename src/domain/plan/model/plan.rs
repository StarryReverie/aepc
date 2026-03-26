use std::collections::BTreeMap;

use getset::Getters;

use crate::domain::item::model::ItemId;
use crate::domain::plan::model::{PlanItemNode, PlanRecipeNode};
use crate::domain::recipe::model::RecipeId;

#[derive(Debug, Clone, PartialEq, Eq, Getters)]
#[getset(get = "pub")]
pub struct Plan {
    goal: PlanItemNode,
    common_intermediates: BTreeMap<ItemId, PlanItemNode>,
    common_recipes: BTreeMap<RecipeId, PlanRecipeNode>,
}

impl Plan {
    pub fn new(
        goal: PlanItemNode,
        common_intermediates: BTreeMap<ItemId, PlanItemNode>,
        common_recipes: BTreeMap<RecipeId, PlanRecipeNode>,
    ) -> Self {
        Self {
            goal,
            common_intermediates,
            common_recipes,
        }
    }
}
