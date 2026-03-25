use std::collections::HashMap;

use getset::Getters;

use crate::domain::item::model::ItemId;
use crate::domain::plan::model::PlanItemNode;

#[derive(Debug, Clone, PartialEq, Eq, Getters)]
#[getset(get = "pub")]
pub struct Plan {
    goal: PlanItemNode,
    common_intermediates: HashMap<ItemId, PlanItemNode>,
}

impl Plan {
    pub fn new(goal: PlanItemNode, common_intermediates: HashMap<ItemId, PlanItemNode>) -> Self {
        Self {
            goal,
            common_intermediates,
        }
    }
}
