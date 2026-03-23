use std::backtrace::Backtrace;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use anyhow::Error as AnyhowError;
use good_lp::{
    Expression, IntoAffineExpression, ProblemVariables, ResolutionError, Solution, SolverModel,
    Variable, VariableDefinition,
};
use snafu::prelude::*;

use crate::domain::item::model::ItemId;
use crate::domain::plan::model::Plan;
use crate::domain::recipe::model::{Flow, Recipe, RecipeId, Replica};
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

    async fn create_plan_impl(
        &self,
        goal: &ItemId,
        flow_goal: Flow,
    ) -> Result<Plan, CreatePlanError> {
        ensure!(flow_goal != Flow::zero(), ZeroFlowSnafu);

        let recipes = self.collect_related_recipes(goal).await?;
        let items = self.collect_related_items(&recipes);
        let solution = self.calc_raw_plan_solution(&recipes, &items, goal, flow_goal)?;

        todo!()
    }

    async fn collect_related_recipes(
        &self,
        goal: &ItemId,
    ) -> Result<HashMap<RecipeId, Recipe>, CreatePlanError> {
        let mut recipes = HashMap::new();
        self.collect_related_recipes_impl(&mut recipes, goal)
            .await?;
        Ok(recipes)
    }

    async fn collect_related_recipes_impl(
        &self,
        recipes_res: &mut HashMap<RecipeId, Recipe>,
        current_item: &ItemId,
    ) -> Result<(), CreatePlanError> {
        let recipes = self
            .recipe_repository
            .find_all_by_products_containing_target(current_item)
            .await
            .context(InfrastructureSnafu {
                message: "querying recipes by target",
            })?;

        ensure!(
            !recipes.is_empty(),
            NoRecipeSnafu {
                item: current_item.clone()
            }
        );

        for recipe in recipes {
            if !recipes_res.contains_key(recipe.id()) {
                let materials = recipe
                    .materials()
                    .iter()
                    .map(|item| item.0.clone())
                    .collect::<Vec<_>>();
                recipes_res.insert(recipe.id().clone(), recipe);

                for material in materials {
                    Box::pin(self.collect_related_recipes_impl(recipes_res, &material)).await?;
                }
            }
        }

        Ok(())
    }

    fn collect_related_items(&self, recipes: &HashMap<RecipeId, Recipe>) -> HashSet<ItemId> {
        let mut items_res = HashSet::with_capacity(recipes.len() * 2);
        for recipe in recipes.values() {
            recipe
                .materials()
                .iter()
                .chain(recipe.products())
                .for_each(|(item, _)| {
                    if !items_res.contains(item) {
                        items_res.insert(item.clone());
                    }
                });
        }
        items_res
    }

    fn calc_raw_plan_solution(
        &self,
        recipes: &HashMap<RecipeId, Recipe>,
        items: &HashSet<ItemId>,
        goal: &ItemId,
        flow_goal: Flow,
    ) -> Result<RawPlanSolution, CreatePlanError> {
        let mut variables = ProblemVariables::new();

        let recipe_replica_vars = recipes
            .iter()
            .map(|(id, recipe)| {
                let name = format!("recipe-replica@{}", recipe.id().value());
                let var = variables.add(VariableDefinition::new().min(0).name(name));
                (id.clone(), var)
            })
            .collect::<HashMap<_, _>>();

        fn mapper(
            variables: &mut ProblemVariables,
            kind: &str,
        ) -> impl FnMut(&ItemId) -> (ItemId, Variable) {
            move |id| {
                let name = format!("item-{kind}-flow@{}", id.value());
                let var = variables.add(VariableDefinition::new().min(0).name(name));
                (id.clone(), var)
            }
        }
        let item_production_flow_vars = items
            .iter()
            .map(mapper(&mut variables, "production"))
            .collect::<HashMap<_, _>>();
        let item_consumption_flow_vars = items
            .iter()
            .map(mapper(&mut variables, "consumption"))
            .collect::<HashMap<_, _>>();
        let item_unused_flow_vars = items
            .iter()
            .map(mapper(&mut variables, "unused"))
            .collect::<HashMap<_, _>>();

        let mut item_production_flow_exprs = items
            .iter()
            .map(|id| (id.clone(), Expression::default()))
            .collect::<HashMap<_, _>>();
        let mut item_consumption_flow_exprs = items
            .iter()
            .map(|id| (id.clone(), Expression::default()))
            .collect::<HashMap<_, _>>();

        for recipe in recipes.values() {
            let replica = recipe_replica_vars.get(recipe.id()).unwrap();
            recipe
                .get_products_rate()
                .into_iter()
                .for_each(|(item, rate)| {
                    let expr = item_production_flow_exprs.get_mut(item).unwrap();
                    *expr += rate.value() * *replica;
                });
            recipe
                .get_materials_rate()
                .into_iter()
                .for_each(|(item, rate)| {
                    let expr = item_consumption_flow_exprs.get_mut(item).unwrap();
                    *expr += rate.value() * *replica;
                });
        }
        *item_consumption_flow_exprs.get_mut(goal).unwrap() += flow_goal.value();

        let objective = item_unused_flow_vars.values().sum::<Expression>()
            + recipe_replica_vars.values().sum::<Expression>();

        let solution = variables
            .minimise(objective)
            .using(good_lp::default_solver)
            .with_all(items.iter().flat_map(|item| {
                let production = item_production_flow_vars.get(item).unwrap();
                let consumption = item_consumption_flow_vars.get(item).unwrap();
                let unused = item_unused_flow_vars.get(item).unwrap();
                [
                    Expression::eq(
                        production.into_expression(),
                        item_production_flow_exprs.remove(item).unwrap(),
                    ),
                    Expression::eq(
                        consumption.into_expression(),
                        item_consumption_flow_exprs.remove(item).unwrap(),
                    ),
                    Expression::eq(
                        production.into_expression(),
                        consumption.into_expression() + unused.into_expression(),
                    ),
                ]
            }))
            .solve()
            .context(NoSolutionSnafu)?;

        Ok(RawPlanSolution {
            recipe_replica: recipes
                .keys()
                .filter_map(|id| {
                    let replica = solution.value(*recipe_replica_vars.get(id).unwrap());
                    Replica::new(replica).ok().map(|r| (id.clone(), r))
                })
                .collect(),
            item_production_flow: items
                .iter()
                .filter_map(|id| {
                    let flow = solution.value(*item_production_flow_vars.get(id).unwrap());
                    Flow::new(flow)
                        .ok()
                        .filter(|f| *f > Flow::zero())
                        .map(|f| (id.clone(), f))
                })
                .collect(),
            item_consumption_flow: items
                .iter()
                .filter_map(|id| {
                    let flow = solution.value(*item_consumption_flow_vars.get(id).unwrap());
                    Flow::new(flow)
                        .ok()
                        .filter(|f| *f > Flow::zero())
                        .map(|f| (id.clone(), f))
                })
                .collect(),
            item_unused_flow: items
                .iter()
                .filter_map(|id| {
                    let flow = solution.value(*item_unused_flow_vars.get(id).unwrap());
                    Flow::new(flow).ok().map(|f| (id.clone(), f))
                })
                .collect(),
        })
    }
}

impl PlanFactory for PlanFactoryImpl {
    async fn create_plan(&self, goal: &ItemId, flow_goal: Flow) -> Result<Plan, CreatePlanError> {
        self.create_plan_impl(goal, flow_goal).await
    }
}

#[derive(Debug, Snafu)]
#[non_exhaustive]
pub enum CreatePlanError {
    #[snafu(display("flow of the goal item should not be zero"))]
    ZeroFlow { backtrace: Backtrace },
    #[snafu(display("could not find recipe that can produce {item:?}"))]
    NoRecipe { item: ItemId, backtrace: Backtrace },
    #[snafu(display("could not find a feasible solution"))]
    NoSolution {
        source: ResolutionError,
        backtrace: Backtrace,
    },
    #[snafu(display("infrastructure error when creating plan: {message}"))]
    Infrastructure {
        message: String,
        source: AnyhowError,
    },
}

#[derive(Debug, Clone, PartialEq)]
struct RawPlanSolution {
    recipe_replica: HashMap<RecipeId, Replica>,
    item_production_flow: HashMap<ItemId, Flow>,
    item_consumption_flow: HashMap<ItemId, Flow>,
    item_unused_flow: HashMap<ItemId, Flow>,
}
