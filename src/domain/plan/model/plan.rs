use getset::{CopyGetters, Getters};

use crate::domain::item::model::ItemId;
use crate::domain::recipe::model::{RecipeId, Replica};

#[derive(Debug, Clone, PartialEq)]
pub enum Plan {
    Normal {
        step: NormalStep,
        dependencies: Vec<Plan>,
    },
    Cyclic {
        step: CyclicStep,
    },
}

#[derive(Debug, Clone, PartialEq, Getters, CopyGetters)]
pub struct NormalStep {
    #[getset(get = "pub")]
    goal: ItemId,
    #[getset(get = "pub")]
    recipe: RecipeId,
    #[getset(get_copy = "pub")]
    replica_effective: Replica,
    #[getset(get_copy = "pub")]
    replica_backward: Option<Replica>,
}

impl NormalStep {
    pub fn forward(goal: ItemId, recipe: RecipeId, replica_effective: Replica) -> Self {
        Self {
            goal,
            recipe,
            replica_effective,
            replica_backward: None,
        }
    }

    pub fn diverged(
        goal: ItemId,
        recipe: RecipeId,
        replica_effective: Replica,
        replica_backward: Replica,
    ) -> Self {
        Self {
            goal,
            recipe,
            replica_effective,
            replica_backward: Some(replica_backward),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Getters, CopyGetters)]
pub struct CyclicStep {
    #[getset(get = "pub")]
    goal: ItemId,
    #[getset(get = "pub")]
    recipe: RecipeId,
    #[getset(get_copy = "pub")]
    replica_effective: Replica,
    #[getset(get_copy = "pub")]
    steps_ahead: u32,
}

impl CyclicStep {
    pub fn new(
        goal: ItemId,
        recipe: RecipeId,
        replica_effective: Replica,
        steps_ahead: u32,
    ) -> Self {
        Self {
            goal,
            recipe,
            replica_effective,
            steps_ahead,
        }
    }
}
