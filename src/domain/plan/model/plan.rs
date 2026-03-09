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

impl Plan {
    pub fn normal(step: NormalStep, dependencies: Vec<Plan>) -> Self {
        Self::Normal { step, dependencies }
    }

    pub fn cyclic(step: CyclicStep) -> Self {
        Self::Cyclic { step }
    }

    pub fn goal(&self) -> &ItemId {
        match self {
            Plan::Normal { step, .. } => step.goal(),
            Plan::Cyclic { step } => step.goal(),
        }
    }

    pub fn replica_effective(&self) -> Replica {
        match self {
            Plan::Normal { step, .. } => step.replica_effective(),
            Plan::Cyclic { step } => step.replica_effective(),
        }
    }

    pub fn get_dependency(&self, dependency: &ItemId) -> Option<&Plan> {
        match self {
            Plan::Normal { dependencies, .. } => {
                dependencies.iter().find(|plan| plan.goal() == dependency)
            }
            Plan::Cyclic { .. } => None,
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_goal_normal() {
        let goal = ItemId::new("i1").unwrap();
        let recipe = RecipeId::new("r1").unwrap();
        let replica = Replica::new(2.0).unwrap();
        let step = NormalStep::forward(goal.clone(), recipe, replica);
        let plan = Plan::normal(step, vec![]);
        assert_eq!(plan.goal(), &goal);
    }

    #[test]
    fn test_goal_cyclic() {
        let goal = ItemId::new("i1").unwrap();
        let recipe = RecipeId::new("r1").unwrap();
        let replica = Replica::new(2.0).unwrap();
        let step = CyclicStep::new(goal.clone(), recipe, replica, 1);
        let plan = Plan::cyclic(step);
        assert_eq!(plan.goal(), &goal);
    }

    #[test]
    fn test_replica_effective_normal() {
        let goal = ItemId::new("i1").unwrap();
        let recipe = RecipeId::new("r1").unwrap();
        let replica = Replica::new(2.5).unwrap();
        let step = NormalStep::forward(goal, recipe, replica);
        let plan = Plan::normal(step, vec![]);
        assert_eq!(plan.replica_effective().value(), 2.5);
    }

    #[test]
    fn test_replica_effective_cyclic() {
        let goal = ItemId::new("i1").unwrap();
        let recipe = RecipeId::new("r1").unwrap();
        let replica = Replica::new(3.7).unwrap();
        let step = CyclicStep::new(goal, recipe, replica, 1);
        let plan = Plan::cyclic(step);
        assert_eq!(plan.replica_effective().value(), 3.7);
    }

    #[test]
    fn test_get_dependency_normal() {
        let goal1 = ItemId::new("i1").unwrap();
        let goal2 = ItemId::new("i2").unwrap();
        let goal3 = ItemId::new("i3").unwrap();
        let recipe = RecipeId::new("r1").unwrap();
        let replica = Replica::new(2.0).unwrap();

        let dep1 = Plan::normal(
            NormalStep::forward(goal2.clone(), recipe.clone(), replica),
            vec![],
        );
        let dep2 = Plan::normal(
            NormalStep::forward(goal3.clone(), recipe.clone(), replica),
            vec![],
        );
        let plan = Plan::normal(
            NormalStep::forward(goal1, recipe, replica),
            vec![dep1, dep2],
        );

        assert_eq!(plan.get_dependency(&goal2).unwrap().goal(), &goal2);
        assert_eq!(plan.get_dependency(&goal3).unwrap().goal(), &goal3);
        assert!(plan.get_dependency(&ItemId::new("i4").unwrap()).is_none());
    }

    #[test]
    fn test_get_dependency_cyclic() {
        let goal1 = ItemId::new("i1").unwrap();
        let goal2 = ItemId::new("i2").unwrap();
        let recipe = RecipeId::new("r1").unwrap();
        let replica = Replica::new(2.0).unwrap();

        let step = CyclicStep::new(goal1, recipe, replica, 1);
        let plan = Plan::cyclic(step);

        assert!(plan.get_dependency(&goal2).is_none());
    }
}
