use std::sync::Arc;

use anyhow::{Result as AnyhowResult, bail};

use crate::domain::item::model::ItemId;
use crate::domain::plan::model::{NormalStep, Plan};
use crate::domain::recipe::model::{Flow, Recipe};
use crate::domain::recipe::outbound::{DynRecipeRepository, RecipeRepository};

pub struct PlanFactory {
    recipe_repository: Arc<DynRecipeRepository<'static>>,
}

impl PlanFactory {
    pub fn new(recipe_repository: Arc<DynRecipeRepository<'static>>) -> Self {
        Self { recipe_repository }
    }

    pub async fn create(&self, goal: &ItemId, flow_goal: Flow) -> AnyhowResult<Plan> {
        self.make_plan(goal, flow_goal).await
    }

    async fn make_plan(&self, goal: &ItemId, flow_goal: Flow) -> AnyhowResult<Plan> {
        let recipes = self.recipe_repository.find_containing_product(goal).await?;
        for recipe in &recipes {
            if let Ok(res) = self.make_plan_with_recipe(goal, flow_goal, recipe).await {
                return Ok(res);
            }
        }
        bail!("no recipe found for product with ID = {goal:?}")
    }

    async fn make_plan_with_recipe(
        &self,
        goal: &ItemId,
        flow_goal: Flow,
        recipe: &Recipe,
    ) -> AnyhowResult<Plan> {
        let rate_goal = recipe
            .get_product_rate(goal)
            .expect("the recipe should produce the goal product");
        let replica_effective = flow_goal / rate_goal;
        let step = NormalStep::forward(goal.clone(), recipe.id().clone(), replica_effective);

        let flow_materials = recipe.get_materials_flow(replica_effective);
        let mut dependencies = Vec::with_capacity(flow_materials.len());
        for (material, flow_material) in flow_materials {
            let dependency = Box::pin(self.make_plan(material, flow_material)).await?;
            dependencies.push(dependency);
        }

        Ok(Plan::normal(step, dependencies))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::domain::machine::model::MachineId;
    use crate::domain::recipe::model::{Period, Quantity, RecipeId, Replica};

    use super::*;

    #[tokio::test]
    async fn test_simple_recipe_no_dependencies() -> AnyhowResult<()> {
        let recipe = make_recipe("r1", "m1", 30.0, vec![], vec![("i1", 60.0)])?;
        let factory = make_factory_with_recipes(vec![recipe])?;

        let goal = ItemId::new("i1")?;
        let flow_goal = Flow::new(120.0)?;
        let plan = factory.create(&goal, flow_goal).await?;

        assert_eq!(plan.goal(), &goal);
        assert_eq!(plan.replica_effective(), Replica::new(1.0)?);
        assert!(plan.get_dependency(&goal).is_none());
        Ok(())
    }

    #[tokio::test]
    async fn test_shared_dependency() -> AnyhowResult<()> {
        let r1 = make_recipe(
            "r1",
            "m1",
            10.0,
            vec![("i2", 10.0), ("i3", 5.0)],
            vec![("i1", 1.0)],
        )?;
        let r2 = make_recipe("r2", "m2", 2.0, vec![("i4", 2.0)], vec![("i2", 1.0)])?;
        let r3 = make_recipe("r3", "m3", 2.0, vec![("i4", 1.0)], vec![("i3", 1.0)])?;
        let r4 = make_recipe("r4", "m4", 1.0, vec![], vec![("i4", 1.0)])?;
        let factory = make_factory_with_recipes(vec![r1, r2, r3, r4])?;

        let goal = ItemId::new("i1")?;
        let flow_goal = Flow::new(6.0)?;
        let i1 = factory.create(&goal, flow_goal).await?;

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
        let recipe = make_recipe("r1", "m1", 10.0, vec![], vec![("i1", 1.0)])?;
        let factory = make_factory_with_recipes(vec![recipe])?;

        let goal = ItemId::new("i2")?;
        let flow_goal = Flow::new(6.0)?;
        let result = factory.create(&goal, flow_goal).await;

        assert!(result.is_err());
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

        async fn find_containing_product(&self, product_id: &ItemId) -> AnyhowResult<Vec<Recipe>> {
            let mut result = Vec::new();
            for recipe in self.recipes.values() {
                if recipe.products().iter().any(|(id, _)| id == product_id) {
                    result.push(recipe.clone());
                }
            }
            Ok(result)
        }
    }

    fn make_recipe(
        recipe_id: &str,
        machine_id: &str,
        period: f64,
        materials: Vec<(&str, f64)>,
        products: Vec<(&str, f64)>,
    ) -> AnyhowResult<Recipe> {
        let materials = materials
            .into_iter()
            .map(|(id, qty)| Ok((ItemId::new(id)?, Quantity::new(qty)?)))
            .collect::<AnyhowResult<Vec<_>>>()?;

        let products = products
            .into_iter()
            .map(|(id, qty)| Ok((ItemId::new(id)?, Quantity::new(qty)?)))
            .collect::<AnyhowResult<Vec<_>>>()?;

        Ok(Recipe::new(
            RecipeId::new(recipe_id)?,
            MachineId::new(machine_id)?,
            Period::new(period)?,
            materials,
            products,
        )?)
    }

    fn make_factory_with_recipes(recipes: Vec<Recipe>) -> AnyhowResult<PlanFactory> {
        let mut mock_repo = MockRecipeRepository::new();
        for recipe in recipes {
            mock_repo.add_recipe(recipe);
        }
        let mock_repo = DynRecipeRepository::new_arc(mock_repo);
        Ok(PlanFactory::new(mock_repo))
    }
}
