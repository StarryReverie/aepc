use derive_builder::Builder;
use getset::{CopyGetters, Getters};

use crate::domain::item::model::ItemId;
use crate::domain::plan::model::PlanItemNode;
use crate::domain::recipe::model::{RecipeId, Replica};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlanRecipeNode {
    Normal(NormalPlanRecipeNode),
    Partial(PartialPlanRecipeNode),
    Cyclic(CyclicPlanRecipeNode),
}

impl PlanRecipeNode {
    pub fn recipe(&self) -> &RecipeId {
        match self {
            Self::Normal(variant) => variant.recipe(),
            Self::Partial(variant) => variant.recipe(),
            Self::Cyclic(variant) => variant.recipe(),
        }
    }

    pub fn replica(&self) -> Replica {
        match self {
            Self::Normal(variant) => variant.replica(),
            Self::Partial(variant) => variant.replica(),
            Self::Cyclic(variant) => variant.replica(),
        }
    }

    pub fn get_material(&self, material: &ItemId) -> Option<&PlanItemNode> {
        if let Self::Normal(variant) = self {
            variant.get_material(material)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Getters, CopyGetters, Builder)]
pub struct NormalPlanRecipeNode {
    #[getset(get = "pub")]
    recipe: RecipeId,
    #[getset(get_copy = "pub")]
    replica: Replica,
    #[getset(get = "pub")]
    materials: Vec<PlanItemNode>,
}

impl NormalPlanRecipeNode {
    pub fn builder() -> NormalPlanRecipeNodeBuilder {
        NormalPlanRecipeNodeBuilder::create_empty()
    }

    pub fn get_material(&self, material: &ItemId) -> Option<&PlanItemNode> {
        self.materials.iter().find(|node| node.target() == material)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Getters, CopyGetters, Builder)]
pub struct PartialPlanRecipeNode {
    #[getset(get = "pub")]
    recipe: RecipeId,
    #[getset(get_copy = "pub")]
    replica: Replica,
}

impl PartialPlanRecipeNode {
    pub fn builder() -> PartialPlanRecipeNodeBuilder {
        PartialPlanRecipeNodeBuilder::create_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Getters, CopyGetters, Builder)]
pub struct CyclicPlanRecipeNode {
    #[getset(get = "pub")]
    recipe: RecipeId,
    #[getset(get_copy = "pub")]
    replica: Replica,
    #[getset(get = "pub")]
    from_steps_ahead: usize,
}

impl CyclicPlanRecipeNode {
    pub fn builder() -> CyclicPlanRecipeNodeBuilder {
        CyclicPlanRecipeNodeBuilder::create_empty()
    }
}
