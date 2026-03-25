use derive_builder::Builder;
use getset::{CopyGetters, Getters};

use crate::domain::item::model::ItemId;
use crate::domain::recipe::model::{Flow, Rate, RecipeId, Replica};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlanItemNode {
    Normal(NormalPlanItemNode),
    Partial(PartialPlanItemNode),
    Cyclic(CyclicPlanItemNode),
}

impl PlanItemNode {
    pub fn target(&self) -> &ItemId {
        match self {
            Self::Normal(variant) => variant.target(),
            Self::Partial(variant) => variant.target(),
            Self::Cyclic(variant) => variant.target(),
        }
    }

    pub fn recipe(&self) -> &RecipeId {
        match self {
            Self::Normal(variant) => variant.recipe(),
            Self::Partial(variant) => variant.recipe(),
            Self::Cyclic(variant) => variant.recipe(),
        }
    }

    pub fn flow_next(&self) -> Flow {
        match self {
            Self::Normal(variant) => variant.flow_next(),
            Self::Partial(variant) => variant.flow_next(),
            Self::Cyclic(variant) => variant.flow_next(),
        }
    }

    pub fn get_dependency(&self, dependency: &ItemId) -> Option<&PlanItemNode> {
        if let Self::Normal(variant) = self {
            variant
                .dependencies()
                .iter()
                .find(|node| node.target() == dependency)
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
    rate: Rate,
    #[getset(get_copy = "pub")]
    replica_next: Replica,
    #[getset(get_copy = "pub")]
    replica_cyclic: Option<Replica>,
    #[getset(get_copy = "pub")]
    flow_extra: Flow,
    #[getset(get = "pub")]
    dependencies: Vec<PlanItemNode>,
}

impl NormalPlanItemNode {
    pub fn builder() -> NormalPlanItemNodeBuilder {
        NormalPlanItemNodeBuilder::create_empty()
    }

    pub fn flow_next(&self) -> Flow {
        self.rate() * self.replica_next()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Getters, CopyGetters, Builder)]
pub struct PartialPlanItemNode {
    #[getset(get = "pub")]
    target: ItemId,
    #[getset(get = "pub")]
    recipe: RecipeId,
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
    #[getset(get = "pub")]
    recipe: RecipeId,
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
