use derive_builder::Builder;
use getset::{CopyGetters, Getters};

use crate::domain::item::model::ItemId;
use crate::domain::plan::model::PlanRecipeNode;
use crate::domain::recipe::model::{Flow, RecipeId, Replica};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlanItemNode {
    Normal(NormalPlanItemNode),
    Aggregated(AggregatedPlanItemNode),
    Partial(PartialPlanItemNode),
    Cyclic(CyclicPlanItemNode),
}

impl PlanItemNode {
    pub fn target(&self) -> &ItemId {
        match self {
            Self::Normal(variant) => variant.target(),
            Self::Aggregated(variant) => variant.target(),
            Self::Partial(variant) => variant.target(),
            Self::Cyclic(variant) => variant.target(),
        }
    }

    pub fn flow_next(&self) -> Flow {
        match self {
            Self::Normal(variant) => variant.flow_next(),
            Self::Aggregated(variant) => variant.flow_next(),
            Self::Partial(variant) => variant.flow_next(),
            Self::Cyclic(variant) => variant.flow_next(),
        }
    }

    pub fn get_dependency(&self, dependency: &ItemId) -> Option<&PlanItemNode> {
        if let Self::Normal(variant) = self {
            variant.get_dependency(dependency)
        } else {
            None
        }
    }

    pub fn get_recipe(&self, recipe: &RecipeId) -> Option<&PlanRecipeNode> {
        if let Self::Aggregated(variant) = self {
            variant.get_recipe(recipe)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Getters, CopyGetters, Builder)]
pub struct NormalPlanItemNode {
    #[getset(get = "pub")]
    target: ItemId,
    #[getset(get = "pub")]
    recipe: RecipeId,
    #[getset(get_copy = "pub")]
    flow_next: Flow,
    #[getset(get_copy = "pub")]
    flow_cyclic: Flow,
    #[getset(get_copy = "pub")]
    replica: Replica,
    #[getset(get = "pub")]
    dependencies: Vec<PlanItemNode>,
}

impl NormalPlanItemNode {
    pub fn builder() -> NormalPlanItemNodeBuilder {
        NormalPlanItemNodeBuilder::create_empty()
    }

    pub fn get_dependency(&self, dependency: &ItemId) -> Option<&PlanItemNode> {
        self.dependencies
            .iter()
            .find(|node| node.target() == dependency)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Getters, CopyGetters, Builder)]
pub struct AggregatedPlanItemNode {
    #[getset(get = "pub")]
    target: ItemId,
    #[getset(get_copy = "pub")]
    flow_next: Flow,
    #[getset(get_copy = "pub")]
    flow_cyclic: Flow,
    #[getset(get = "pub")]
    recipes: Vec<PlanRecipeNode>,
}

impl AggregatedPlanItemNode {
    pub fn builder() -> AggregatedPlanItemNodeBuilder {
        AggregatedPlanItemNodeBuilder::create_empty()
    }

    pub fn get_recipe(&self, recipe: &RecipeId) -> Option<&PlanRecipeNode> {
        self.recipes.iter().find(|node| node.recipe() == recipe)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Getters, CopyGetters, Builder)]
pub struct PartialPlanItemNode {
    #[getset(get = "pub")]
    target: ItemId,
    #[getset(get_copy = "pub")]
    flow_next: Flow,
}

impl PartialPlanItemNode {
    pub fn builder() -> PartialPlanItemNodeBuilder {
        PartialPlanItemNodeBuilder::create_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Getters, CopyGetters, Builder)]
pub struct CyclicPlanItemNode {
    #[getset(get = "pub")]
    target: ItemId,
    #[getset(get_copy = "pub")]
    flow_next: Flow,
    #[getset(get_copy = "pub")]
    from_steps_ahead: usize,
}

impl CyclicPlanItemNode {
    pub fn builder() -> CyclicPlanItemNodeBuilder {
        CyclicPlanItemNodeBuilder::create_empty()
    }
}
