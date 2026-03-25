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
use crate::domain::plan::model::{
    CyclicPlanItemNode, NormalPlanItemNode, PartialPlanItemNode, Plan, PlanItemNode,
};
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
        let plan = self.build_plan(&recipes, &solution, goal);
        Ok(plan)
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

    fn build_plan(
        &self,
        recipes: &HashMap<RecipeId, Recipe>,
        solution: &RawPlanSolution,
        goal: &ItemId,
    ) -> Plan {
        let mut item_producers = HashMap::new();
        recipes
            .values()
            .filter(|recipe| solution.recipe_replica.contains_key(recipe.id()))
            .for_each(|recipe| {
                recipe.products().iter().for_each(|(item, _)| {
                    let ps = item_producers.entry(item).or_insert(Vec::new());
                    ps.push(recipe);
                });
            });

        let item_main_producers = item_producers
            .into_iter()
            .map(|(id, producers)| {
                let has_unique_product = producers.iter().find(|p| p.products().len() == 1);
                let otherwise_first = producers.first();
                let main_producer = *has_unique_product.or(otherwise_first).unwrap();
                (id, main_producer)
            })
            .collect::<HashMap<&ItemId, &Recipe>>();

        let context = BuildPlanContext {
            recipes,
            item_main_producers: &item_main_producers,
            solution,
        };
        let mut trace = Vec::new();
        let mut common_intermediates = HashMap::new();

        let goal = self.build_plan_recursive(&context, &mut trace, &mut common_intermediates, goal);
        Plan::new(goal, common_intermediates)
    }

    fn build_plan_recursive<'a>(
        &self,
        context: &BuildPlanContext<'a>,
        trace: &mut Vec<BuildPlanTrace<'a>>,
        common_intermediates: &mut HashMap<ItemId, PlanItemNode>,
        target: &'a ItemId,
    ) -> PlanItemNode {
        let BuildPlanContext {
            recipes,
            item_main_producers,
            solution,
        } = context;

        let recipe = recipes
            .get(item_main_producers.get(target).unwrap().id())
            .unwrap();
        let rate = recipe.get_product_rate(target).unwrap();
        let replica_all = *solution.recipe_replica.get(recipe.id()).unwrap();

        trace.push(BuildPlanTrace {
            depth: trace.len(),
            target,
            main_producer: recipe,
            flow_cyclic: Flow::zero(),
        });

        let mut dependencies = Vec::new();
        for (material, flow_material) in recipe.get_materials_flow(replica_all) {
            let dependency =
                if let Some(frame) = trace.iter_mut().rfind(|frame| frame.target == material) {
                    frame.flow_cyclic = frame.flow_cyclic + flow_material;
                    let prev_depth = frame.depth;
                    PlanItemNode::Cyclic(
                        CyclicPlanItemNode::builder()
                            .target(material.clone())
                            .recipe(frame.main_producer.id().clone())
                            .flow_next(flow_material)
                            .from_steps_ahead(trace.last().unwrap().depth - prev_depth + 1)
                            .build()
                            .unwrap(),
                    )
                } else if let Some(node) = common_intermediates.get(material) {
                    PlanItemNode::Partial(
                        PartialPlanItemNode::builder()
                            .target(material.clone())
                            .recipe(node.recipe().clone())
                            .flow_next(flow_material)
                            .build()
                            .unwrap(),
                    )
                } else {
                    let dependency =
                        self.build_plan_recursive(context, trace, common_intermediates, material);
                    if dependency.flow_next() > flow_material {
                        let partial = PlanItemNode::Partial(
                            PartialPlanItemNode::builder()
                                .target(material.clone())
                                .recipe(dependency.recipe().clone())
                                .flow_next(flow_material)
                                .build()
                                .unwrap(),
                        );
                        common_intermediates.insert(material.clone(), dependency);
                        partial
                    } else {
                        dependency
                    }
                };
            dependencies.push(dependency);
        }

        let flow_extra = Flow::new(
            solution.item_production_flow.get(target).unwrap().value()
                - (rate * replica_all).value(),
        )
        .expect("total production flow shouldn't be less than main producer recipe's flow");

        let flow_cyclic = trace.last().unwrap().flow_cyclic;
        let (replica_next, replica_cyclic) = if flow_cyclic > flow_extra {
            let flow_cyclic_from_main = Flow::new(flow_cyclic.value() - flow_extra.value())
                .expect("`flow_cyclic` should be greater than `flow_extra`");
            let replica_cyclic = flow_cyclic_from_main / rate;
            let replica_next = Replica::new(replica_all.value() - replica_cyclic.value())
                .expect("`replica_all` should be greater than `replica_cyclic`");
            (replica_next, Some(replica_cyclic))
        } else {
            (replica_all, None)
        };

        let node = PlanItemNode::Normal(
            NormalPlanItemNode::builder()
                .target(target.clone())
                .recipe(recipe.id().clone())
                .rate(rate)
                .replica_next(replica_next)
                .replica_cyclic(replica_cyclic)
                .flow_extra(flow_extra)
                .dependencies(dependencies)
                .build()
                .unwrap(),
        );

        trace.pop();
        node
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

#[derive(Debug, Clone, PartialEq)]
struct BuildPlanContext<'a> {
    recipes: &'a HashMap<RecipeId, Recipe>,
    item_main_producers: &'a HashMap<&'a ItemId, &'a Recipe>,
    solution: &'a RawPlanSolution,
}

#[derive(Debug, Clone, PartialEq)]
struct BuildPlanTrace<'a> {
    depth: usize,
    target: &'a ItemId,
    main_producer: &'a Recipe,
    flow_cyclic: Flow,
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use anyhow::Result as AnyhowResult;

    use crate::domain::recipe::model::{RecipeId, Replica, test_helper::make_recipe};

    use super::*;

    #[tokio::test]
    async fn test_no_recipe_found() -> AnyhowResult<()> {
        #[rustfmt::skip]
        let factory = {
            let recipe = make_recipe("r1", "m1", 10.0, vec![], vec![("i1", 1.0)]);
            make_factory_with_recipes(vec![recipe])?
        };

        let goal = ItemId::new("i2")?;
        let flow_goal = Flow::new(6.0)?;
        let res = factory.create_plan(&goal, flow_goal).await;

        assert!(matches!(res, Err(CreatePlanError::NoRecipe { .. })));
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
        let plan = factory.create_plan(&goal, flow_goal).await?;

        visit_node_normal(&plan, &["i1"], |variant| {
            assert_eq!(variant.replica_next(), Replica::new(1.0).unwrap());
            assert_eq!(variant.flow_next(), Flow::new(6.0).unwrap());
        });
        visit_node_normal(&plan, &["i1", "i2"], |variant| {
            assert_eq!(variant.replica_next(), Replica::new(2.0).unwrap());
            assert_eq!(variant.flow_next(), Flow::new(60.0).unwrap());
        });
        visit_node_partial(&plan, &["i1", "i2", "i4"], |variant| {
            assert_eq!(variant.flow_next(), Flow::new(120.0).unwrap());
        });
        visit_node_normal(&plan, &["i1", "i3"], |variant| {
            assert_eq!(variant.replica_next(), Replica::new(1.0).unwrap());
            assert_eq!(variant.flow_next(), Flow::new(30.0).unwrap());
        });
        visit_node_partial(&plan, &["i1", "i3", "i4"], |variant| {
            assert_eq!(variant.flow_next(), Flow::new(30.0).unwrap());
        });
        visit_node_normal(&plan, &["i4"], |variant| {
            assert_eq!(variant.replica_next(), Replica::new(2.5).unwrap());
            assert_eq!(variant.flow_next(), Flow::new(150.0).unwrap());
        });

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

        let goal = ItemId::new("i1")?;
        let flow_goal = Flow::new(60.0)?;
        let plan = factory.create_plan(&goal, flow_goal).await?;

        visit_node_normal(&plan, &["i1"], |variant| {
            assert_eq!(variant.replica_next(), Replica::new(0.5).unwrap());
            assert_eq!(variant.replica_cyclic(), Some(Replica::new(0.5).unwrap()));
            assert_eq!(variant.flow_next(), Flow::new(60.0).unwrap());
        });
        visit_node_normal(&plan, &["i1", "i2"], |variant| {
            assert_eq!(variant.replica_next(), Replica::new(1.0).unwrap());
            assert_eq!(variant.flow_next(), Flow::new(60.0).unwrap());
        });
        visit_node_cyclic(&plan, &["i1", "i2", "i1"], |variant| {
            assert_eq!(variant.flow_next(), Flow::new(60.0).unwrap());
            assert_eq!(variant.from_steps_ahead(), 2);
        });

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

    fn visit_node<F>(plan: &Plan, path: &[&str], f: F)
    where
        F: FnOnce(&PlanItemNode),
    {
        let first = ItemId::new(path[0]).unwrap();
        let mut current = if plan.goal().target() == &first {
            plan.goal()
        } else if let Some(node) = plan.common_intermediates().get(&first) {
            node
        } else {
            panic!("starting node '{}' not found", path[0]);
        };
        for item in &path[1..] {
            current = current
                .get_dependency(&ItemId::new(*item).unwrap())
                .expect("invalid path");
        }
        f(current)
    }

    fn visit_node_normal(plan: &Plan, path: &[&str], f: impl FnOnce(&NormalPlanItemNode)) {
        visit_node(plan, path, |node| match node {
            PlanItemNode::Normal(variant) => f(variant),
            _ => panic!("expect `PlanNode::Normal`"),
        })
    }

    fn visit_node_partial(plan: &Plan, path: &[&str], f: impl FnOnce(&PartialPlanItemNode)) {
        visit_node(plan, path, |node| match node {
            PlanItemNode::Partial(variant) => f(variant),
            _ => panic!("expect `PlanNode::Partial`"),
        })
    }

    fn visit_node_cyclic(plan: &Plan, path: &[&str], f: impl FnOnce(&CyclicPlanItemNode)) {
        visit_node(plan, path, |node| match node {
            PlanItemNode::Cyclic(variant) => f(variant),
            _ => panic!("expect `PlanNode::Cyclic`"),
        })
    }
}
