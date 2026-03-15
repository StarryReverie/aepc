use std::sync::Arc;

use anyhow::Error as AnyhowError;
use snafu::prelude::*;

use crate::domain::item::model::ItemId;
use crate::domain::plan::model::{CyclicStep, NormalStep, Plan};
use crate::domain::recipe::model::{Flow, Rate, Recipe, RecipeId, Replica};
use crate::domain::recipe::outbound::{DynRecipeRepository, RecipeRepository};

#[unimock::unimock(api = PlanFactoryMock)]
#[dynosaur::dynosaur(pub DynPlanFactory = dyn(box) PlanFactory)]
pub trait PlanFactory: Send + Sync {
    fn create_plan(
        &self,
        goal: &ItemId,
        flow_goal: Flow,
    ) -> impl Future<Output = Result<Plan, CreatePlanError>> + Send;
}

pub struct PlanFactoryImpl {
    recipe_repository: Arc<DynRecipeRepository<'static>>,
}

impl PlanFactoryImpl {
    pub fn new(recipe_repository: Arc<DynRecipeRepository<'static>>) -> Self {
        Self { recipe_repository }
    }

    async fn make_plan(
        &self,
        goal: &ItemId,
        flow_goal: Flow,
        resolution_trace: &mut Vec<ResolutionTraceElement>,
    ) -> Result<(Plan, Vec<BackwardDemandElement>), CreatePlanError> {
        if let Some(target) = resolution_trace.iter().rfind(|e| &e.goal == goal) {
            let plan = Plan::cyclic(CyclicStep::new(
                goal.clone(),
                target.recipe.clone(),
                flow_goal / target.rate_goal,
                resolution_trace.len() as u32 - target.depth,
            ));
            let demand = BackwardDemandElement {
                goal: goal.clone(),
                flow_goal,
            };
            return Ok((plan, vec![demand]));
        }

        let recipes = self
            .recipe_repository
            .find_all_by_products_containing_target(goal)
            .await
            .context(InfrastructureSnafu {
                message: "could not query recipes",
            })?;

        for recipe in &recipes {
            if let Ok(res) = self
                .make_plan_with_recipe(goal, flow_goal, recipe, resolution_trace)
                .await
            {
                return Ok(res);
            }
        }

        (NoRecipeSnafu { goal: goal.clone() }).fail()
    }

    async fn make_plan_with_recipe(
        &self,
        goal: &ItemId,
        flow_goal: Flow,
        recipe: &Recipe,
        resolution_trace: &mut Vec<ResolutionTraceElement>,
    ) -> Result<(Plan, Vec<BackwardDemandElement>), CreatePlanError> {
        let rate_goal = recipe
            .get_product_rate(goal)
            .expect("the recipe should produce the goal product");
        let replica_full = flow_goal / rate_goal;
        let step = NormalStep::forward(goal.clone(), recipe.id().clone(), replica_full);

        resolution_trace.push(ResolutionTraceElement {
            depth: resolution_trace.len() as u32,
            goal: goal.clone(),
            recipe: recipe.id().clone(),
            rate_goal,
        });

        let flow_materials = recipe.get_materials_flow(replica_full);
        let mut dependencies = Vec::with_capacity(flow_materials.len());
        let mut demands = Vec::new();
        for (material, flow_material) in flow_materials {
            let (dependency, extra_demands) =
                Box::pin(self.make_plan(material, flow_material, resolution_trace))
                    .await
                    .inspect_err(|_| drop(resolution_trace.pop()))?;
            dependencies.push(dependency);
            demands.extend(extra_demands);
        }

        let flow_backward = demands
            .extract_if(.., |demand| &demand.goal == goal)
            .map(|demand| demand.flow_goal)
            .fold(Flow::zero(), |sum, x| sum + x);

        let plan = if flow_backward == Flow::zero() {
            Plan::normal(step, dependencies)
        } else if flow_goal > flow_backward {
            let flow_effective = Flow::new(flow_goal.value() - flow_backward.value())
                .expect("`flow_goal` should be greater than `flow_backward`");
            let adjust_multipler = Replica::new(flow_goal.value() / flow_effective.value())
                .expect("the replica should be positive");

            let adjusted_flow_backward =
                Flow::new(adjust_multipler.value() * flow_backward.value())
                    .expect("the flow should be positive");
            let adjusted_step = NormalStep::diverged(
                goal.clone(),
                recipe.id().clone(),
                flow_goal / rate_goal,
                adjusted_flow_backward / rate_goal,
            );
            let adjusted_dependencies = dependencies
                .into_iter()
                .map(|plan| plan.amplify(adjust_multipler))
                .collect();

            Plan::normal(adjusted_step, adjusted_dependencies)
        } else {
            let _ = resolution_trace.pop();
            return ExcessiveCyclicFlowSnafu.fail();
        };

        let _ = resolution_trace.pop();
        Ok((plan, demands))
    }
}

impl PlanFactory for PlanFactoryImpl {
    async fn create_plan(&self, goal: &ItemId, flow_goal: Flow) -> Result<Plan, CreatePlanError> {
        let mut resolution_trace = Vec::new();
        let (plan, _) = self
            .make_plan(goal, flow_goal, &mut resolution_trace)
            .await?;
        Ok(plan)
    }
}

#[derive(Debug, Snafu)]
#[non_exhaustive]
pub enum CreatePlanError {
    #[snafu(display("could not find recipe that can produce {goal:?}"))]
    NoRecipe { goal: ItemId },
    #[snafu(display("cyclic flow demanded by backward exceeds current production"))]
    ExcessiveCyclicFlow,
    #[snafu(display("infrastructure error: {message}"))]
    Infrastructure {
        message: String,
        source: AnyhowError,
    },
}

struct ResolutionTraceElement {
    depth: u32,
    goal: ItemId,
    recipe: RecipeId,
    rate_goal: Rate,
}

struct BackwardDemandElement {
    goal: ItemId,
    flow_goal: Flow,
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use anyhow::Result as AnyhowResult;

    use crate::domain::recipe::model::{RecipeId, Replica, test_helper::make_recipe};

    use super::*;

    #[tokio::test]
    async fn test_pipeline_without_dependencies() -> AnyhowResult<()> {
        #[rustfmt::skip]
        let factory = {
            let recipe = make_recipe("r1", "m1", 30.0, vec![], vec![("i1", 60.0)]);
            make_factory_with_recipes(vec![recipe])?
        };

        let goal = ItemId::new("i1")?;
        let flow_goal = Flow::new(120.0)?;
        let plan = factory.create_plan(&goal, flow_goal).await?;

        assert_eq!(plan.goal(), &goal);
        assert_eq!(plan.replica_effective(), Replica::new(1.0)?);
        assert!(plan.get_dependency(&goal).is_none());
        Ok(())
    }

    #[tokio::test]
    async fn test_pipeline_with_shared_dependency() -> AnyhowResult<()> {
        #[rustfmt::skip]
        let factory = {
            let r1 = make_recipe("r1", "m1", 10.0, vec![("i2", 10.0), ("i3", 5.0)], vec![("i1", 1.0)]);
            let r2 = make_recipe("r2", "m2", 2.0, vec![("i4", 2.0)], vec![("i2", 1.0)]);
            let r3 = make_recipe("r3", "m3", 2.0, vec![("i4", 1.0)], vec![("i3", 1.0)]);
            let r4 = make_recipe("r4", "m4", 1.0, vec![], vec![("i4", 1.0)]);
            make_factory_with_recipes(vec![r1, r2, r3, r4])?
        };

        let goal = ItemId::new("i1")?;
        let flow_goal = Flow::new(6.0)?;
        let i1 = factory.create_plan(&goal, flow_goal).await?;

        if i1.goal() == &goal {
            assert_eq!(i1.replica_effective(), Replica::new(1.0)?);

            if let Some(i2) = i1.get_dependency(&ItemId::new("i2")?) {
                assert_eq!(i2.replica_effective(), Replica::new(2.0)?);

                if let Some(i4) = i2.get_dependency(&ItemId::new("i4")?) {
                    assert_eq!(i4.replica_effective(), Replica::new(2.0)?);
                } else {
                    panic!("i4 dependency not found in i2 plan");
                }
            } else {
                panic!("i2 dependency not found");
            }

            if let Some(i3) = i1.get_dependency(&ItemId::new("i3")?) {
                assert_eq!(i3.replica_effective(), Replica::new(1.0)?);

                if let Some(i4) = i3.get_dependency(&ItemId::new("i4")?) {
                    assert_eq!(i4.replica_effective(), Replica::new(0.5)?);
                } else {
                    panic!("i4 dependency not found in i3 plan");
                }
            } else {
                panic!("i3 dependency not found");
            }
        } else {
            panic!("plan not found");
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_no_recipe_found() -> AnyhowResult<()> {
        #[rustfmt::skip]
        let factory = {
            let recipe = make_recipe("r1", "m1", 10.0, vec![], vec![("i1", 1.0)]);
            make_factory_with_recipes(vec![recipe])?
        };

        let goal = ItemId::new("i2")?;
        let flow_goal = Flow::new(6.0)?;
        let result = factory.create_plan(&goal, flow_goal).await;

        assert!(result.is_err());
        Ok(())
    }

    #[tokio::test]
    async fn test_cyclic_pipeline() -> AnyhowResult<()> {
        #[rustfmt::skip]
        let factory = {
            let r1 = make_recipe("r1", "m1", 1.0, vec![("i2", 1.0)], vec![("i1", 2.0)]);
            let r2 = make_recipe("r2", "m2", 1.0, vec![("i1", 1.0)], vec![("i2", 1.0)]);
            make_factory_with_recipes(vec![r1, r2])?
        };

        let flow_goal = Flow::new(60.0)?;
        let plan = factory.create_plan(&ItemId::new("i1")?, flow_goal).await?;

        if plan.goal() == &ItemId::new("i1")? {
            assert_eq!(plan.replica_effective(), Replica::new(0.5)?);

            if let Some(i2) = plan.get_dependency(&ItemId::new("i2")?) {
                assert_eq!(i2.replica_effective(), Replica::new(1.0)?);

                if let Some(i1) = i2.get_dependency(&ItemId::new("i1")?) {
                    assert_eq!(i1.replica_effective(), Replica::new(0.5)?);
                } else {
                    panic!("i1 dependency not found in i2 plan");
                }
            } else {
                panic!("i2 dependency not found");
            }
        } else {
            panic!("i1 not found");
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_no_recipe_found_when_cyclic_flow_exceeds_production() -> AnyhowResult<()> {
        #[rustfmt::skip]
        let factory = {
            let r1 = make_recipe("r1", "m1", 1.0, vec![("i2", 1.0)], vec![("i1", 1.0)]);
            let r2 = make_recipe("r2", "m2", 1.0, vec![("i1", 1.0)], vec![("i2", 1.0)]);
            make_factory_with_recipes(vec![r1, r2])?
        };

        let goal = ItemId::new("i1")?;
        let flow_goal = Flow::new(60.0)?;
        let result = factory.create_plan(&goal, flow_goal).await;

        assert!(result.is_err());
        Ok(())
    }

    #[tokio::test]
    async fn test_complex_cyclic_pipeline() -> AnyhowResult<()> {
        #[rustfmt::skip]
        let factory = {
            let r1 = make_recipe("r1", "m1", 1.0, vec![("i1", 1.0), ("i2", 1.0)], vec![("i3", 3.0)]);
            let r2 = make_recipe("r2", "m2", 1.0, vec![("i3", 1.0)], vec![("i1", 1.0)]);
            let r3 = make_recipe("r3", "m3", 1.0, vec![("i3", 1.0), ("i4", 1.0)], vec![("i5", 1.0)]);
            let r4 = make_recipe("r4", "m4", 1.0, vec![("i5", 1.0)], vec![("i2", 1.0)]);
            let r5 = make_recipe("r5", "m5", 1.0, vec![("i3", 1.0), ("i6", 1.0), ("i7", 1.0)], vec![("i8", 1.0)]);
            let r6 = make_recipe("r6", "m6", 1.0, vec![], vec![("i4", 1.0)]);
            let r7 = make_recipe("r7", "m7", 1.0, vec![], vec![("i6", 1.0)]);
            let r8 = make_recipe("r8", "m8", 1.0, vec![], vec![("i7", 1.0)]);
            make_factory_with_recipes(vec![r1, r2, r3, r4, r5, r6, r7, r8])?
        };

        let goal = ItemId::new("i8")?;
        let flow_goal = Flow::new(180.0)?;
        let plan = factory.create_plan(&goal, flow_goal).await?;

        if plan.goal() == &goal {
            assert_eq!(plan.replica_effective(), Replica::new(3.0)?);

            if let Some(i3) = plan.get_dependency(&ItemId::new("i3")?) {
                assert_eq!(i3.replica_effective(), Replica::new(1.0)?);

                if let Some(i1) = i3.get_dependency(&ItemId::new("i1")?) {
                    assert_eq!(i1.replica_effective(), Replica::new(3.0)?);

                    if let Some(i3_cyclic) = i1.get_dependency(&ItemId::new("i3")?) {
                        assert_eq!(i3_cyclic.replica_effective(), Replica::new(1.0)?);
                    } else {
                        panic!("i3 dependency not found in i1 plan");
                    }
                } else {
                    panic!("i1 dependency not found in i3 plan");
                }

                if let Some(i2) = i3.get_dependency(&ItemId::new("i2")?) {
                    assert_eq!(i2.replica_effective(), Replica::new(3.0)?);

                    if let Some(i5) = i2.get_dependency(&ItemId::new("i5")?) {
                        assert_eq!(i5.replica_effective(), Replica::new(3.0)?);

                        if let Some(i3_cyclic) = i5.get_dependency(&ItemId::new("i3")?) {
                            assert_eq!(i3_cyclic.replica_effective(), Replica::new(1.0)?);
                        } else {
                            panic!("i3 dependency not found in i5 plan");
                        }

                        if let Some(i4) = i5.get_dependency(&ItemId::new("i4")?) {
                            assert_eq!(i4.replica_effective(), Replica::new(3.0)?);
                        } else {
                            panic!("i4 dependency not found in i5 plan");
                        }
                    } else {
                        panic!("i5 dependency not found in i2 plan");
                    }
                } else {
                    panic!("i2 dependency not found in i3 plan");
                }
            } else {
                panic!("i3 dependency not found");
            }

            if let Some(i6) = plan.get_dependency(&ItemId::new("i6")?) {
                assert_eq!(i6.replica_effective(), Replica::new(3.0)?);
            } else {
                panic!("i6 dependency not found");
            }

            if let Some(i7) = plan.get_dependency(&ItemId::new("i7")?) {
                assert_eq!(i7.replica_effective(), Replica::new(3.0)?);
            } else {
                panic!("i7 dependency not found");
            }
        } else {
            panic!("plan not found");
        }

        Ok(())
    }

    struct MockRecipeRepository {
        recipes: HashMap<RecipeId, Recipe>,
    }

    impl MockRecipeRepository {
        fn new() -> Self {
            Self {
                recipes: HashMap::new(),
            }
        }

        fn add_recipe(&mut self, recipe: Recipe) {
            let id = recipe.id().clone();
            self.recipes.insert(id, recipe);
        }
    }

    impl RecipeRepository for MockRecipeRepository {
        async fn get(&self, _recipe_id: &RecipeId) -> AnyhowResult<Option<Recipe>> {
            Ok(None)
        }

        async fn find_all_by_products_containing_target(
            &self,
            product_id: &ItemId,
        ) -> AnyhowResult<Vec<Recipe>> {
            let mut result = Vec::new();
            for recipe in self.recipes.values() {
                if recipe.products().iter().any(|(id, _)| id == product_id) {
                    result.push(recipe.clone());
                }
            }
            Ok(result)
        }
    }

    fn make_factory_with_recipes(recipes: Vec<Recipe>) -> AnyhowResult<PlanFactoryImpl> {
        let mut mock_repo = MockRecipeRepository::new();
        for recipe in recipes {
            mock_repo.add_recipe(recipe);
        }
        let mock_repo = DynRecipeRepository::new_arc(mock_repo);
        Ok(PlanFactoryImpl::new(mock_repo))
    }
}
