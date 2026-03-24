use std::collections::HashMap;

use getset::Getters;

use crate::domain::item::model::ItemId;
use crate::domain::recipe::model::{Flow, Rate, RecipeId, Replica};

#[derive(Debug, Clone, PartialEq, Getters)]
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

#[derive(Debug, Clone, PartialEq)]
pub enum PlanNode {
    Normal {
        target: ItemId,
        recipe: RecipeId,
        rate: Rate,
        replica_next: Replica,
        replica_cyclic: Option<Replica>,
        flow_extra: Flow,
        dependencies: Vec<PlanNode>,
    },
    Partial {
        target: ItemId,
        recipe: RecipeId,
        flow_next: Flow,
    },
    Cyclic {
        target: ItemId,
        recipe: RecipeId,
        flow_next: Flow,
        from_steps_ahead: usize,
    },
}

impl PlanNode {
    pub fn target(&self) -> &ItemId {
        match self {
            Self::Normal { target, .. } => target,
            Self::Partial { target, .. } => target,
            Self::Cyclic { target, .. } => target,
        }
    }

    pub fn recipe(&self) -> &RecipeId {
        match self {
            Self::Normal { recipe, .. } => recipe,
            Self::Partial { recipe, .. } => recipe,
            Self::Cyclic { recipe, .. } => recipe,
        }
    }

    pub fn flow_next(&self) -> Flow {
        match self {
            Self::Normal {
                rate, replica_next, ..
            } => *rate * *replica_next,
            Self::Partial { flow_next, .. } => *flow_next,
            Self::Cyclic { flow_next, .. } => *flow_next,
        }
    }

    pub fn get_dependency(&self, dependency: &ItemId) -> Option<&PlanNode> {
        if let Self::Normal { dependencies, .. } = self {
            dependencies.iter().find(|node| node.target() == dependency)
        } else {
            None
        }
    }
}
