use std::collections::HashMap;

use derive_builder::Builder;
use getset::{CopyGetters, Getters};

use crate::domain::item::model::ItemId;
use crate::domain::recipe::model::{Flow, Rate, RecipeId, Replica};

#[derive(Debug, Clone, PartialEq, Eq, Getters)]
#[getset(get = "pub")]
pub struct Plan {
    goal: PlanNode,
    common_intermediates: HashMap<ItemId, PlanNode>,
}

impl Plan {
    pub fn new(goal: PlanNode, common_intermediates: HashMap<ItemId, PlanNode>) -> Self {
        Self {
            goal,
            common_intermediates,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlanNode {
    Normal(PlanNodeNormalVariant),
    Partial(PlanNodePartialVariant),
    Cyclic(PlanNodeCyclicVariant),
}

impl PlanNode {
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

    pub fn get_dependency(&self, dependency: &ItemId) -> Option<&PlanNode> {
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
pub struct PlanNodeNormalVariant {
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
    dependencies: Vec<PlanNode>,
}

impl PlanNodeNormalVariant {
    pub fn builder() -> PlanNodeNormalVariantBuilder {
        PlanNodeNormalVariantBuilder::create_empty()
    }

    pub fn flow_next(&self) -> Flow {
        self.rate() * self.replica_next()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Getters, CopyGetters, Builder)]
pub struct PlanNodePartialVariant {
    #[getset(get = "pub")]
    target: ItemId,
    #[getset(get = "pub")]
    recipe: RecipeId,
    #[getset(get_copy = "pub")]
    flow_next: Flow,
}

impl PlanNodePartialVariant {
    pub fn builder() -> PlanNodePartialVariantBuilder {
        PlanNodePartialVariantBuilder::create_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Getters, CopyGetters, Builder)]
pub struct PlanNodeCyclicVariant {
    #[getset(get = "pub")]
    target: ItemId,
    #[getset(get = "pub")]
    recipe: RecipeId,
    #[getset(get_copy = "pub")]
    flow_next: Flow,
    #[getset(get_copy = "pub")]
    from_steps_ahead: usize,
}

impl PlanNodeCyclicVariant {
    pub fn builder() -> PlanNodeCyclicVariantBuilder {
        PlanNodeCyclicVariantBuilder::create_empty()
    }
}
