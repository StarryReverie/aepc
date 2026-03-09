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
    use anyhow::Result as AnyhowResult;

    use super::*;

    #[test]
    fn test_goal_normal() -> AnyhowResult<()> {
        let goal = ItemId::new("i1")?;
        let plan = Plan::normal(
            NormalStep::forward(goal.clone(), RecipeId::new("r1")?, Replica::new(2.0)?),
            vec![],
        );
        assert_eq!(plan.goal(), &goal);
        Ok(())
    }

    #[test]
    fn test_goal_cyclic() -> AnyhowResult<()> {
        let goal = ItemId::new("i1")?;
        let plan = Plan::cyclic(CyclicStep::new(
            goal.clone(),
            RecipeId::new("r1")?,
            Replica::new(2.0)?,
            1,
        ));
        assert_eq!(plan.goal(), &goal);
        Ok(())
    }

    #[test]
    fn test_replica_effective_normal() -> AnyhowResult<()> {
        let plan = Plan::normal(
            NormalStep::forward(ItemId::new("i1")?, RecipeId::new("r1")?, Replica::new(2.5)?),
            vec![],
        );
        assert_eq!(plan.replica_effective(), Replica::new(2.5)?);
        Ok(())
    }

    #[test]
    fn test_replica_effective_cyclic() -> AnyhowResult<()> {
        let plan = Plan::cyclic(CyclicStep::new(
            ItemId::new("i1")?,
            RecipeId::new("r1")?,
            Replica::new(3.7)?,
            1,
        ));
        assert_eq!(plan.replica_effective(), Replica::new(3.7)?);
        Ok(())
    }

    #[test]
    fn test_get_dependency_normal() -> AnyhowResult<()> {
        let goal1 = ItemId::new("i1")?;
        let goal2 = ItemId::new("i2")?;
        let goal3 = ItemId::new("i3")?;
        let recipe = RecipeId::new("r1")?;
        let replica = Replica::new(2.0)?;

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
        assert!(plan.get_dependency(&ItemId::new("i4")?).is_none());
        Ok(())
    }

    #[test]
    fn test_get_dependency_cyclic() -> AnyhowResult<()> {
        let goal1 = ItemId::new("i1")?;
        let goal2 = ItemId::new("i2")?;
        let recipe = RecipeId::new("r1")?;
        let replica = Replica::new(2.0)?;

        let step = CyclicStep::new(goal1, recipe, replica, 1);
        let plan = Plan::cyclic(step);

        assert!(plan.get_dependency(&goal2).is_none());
        Ok(())
    }
}
