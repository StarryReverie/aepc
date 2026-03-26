use std::backtrace::Backtrace;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use anyhow::Error as AnyhowError;
use good_lp::solvers::microlp::MicroLpSolution;
use good_lp::{
    Expression, IntoAffineExpression, ProblemVariables, ResolutionError, Solution, SolverModel,
    Variable, VariableDefinition,
};
use snafu::prelude::*;

use crate::domain::item::model::ItemId;
use crate::domain::plan::model::{
    AggregatedPlanItemNode, CyclicPlanItemNode, CyclicPlanRecipeNode, NormalPlanItemNode,
    NormalPlanRecipeNode, PartialPlanItemNode, PartialPlanRecipeNode, Plan, PlanItemNode,
    PlanRecipeNode,
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
        let plan = self.build_plan(&recipes, &solution, goal, flow_goal);
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

    fn calc_raw_plan_solution<'a>(
        &self,
        recipes: &'a HashMap<RecipeId, Recipe>,
        items: &'a HashSet<ItemId>,
        goal: &ItemId,
        flow_goal: Flow,
    ) -> Result<RawPlanSolution<'a>, CreatePlanError> {
        let mut variables = ProblemVariables::new();

        let mut recipe_replica_vars = recipes
            .iter()
            .map(|(id, recipe)| {
                let name = format!("recipe-replica@{}", recipe.id().value());
                let var = variables.add(VariableDefinition::new().min(0).name(name));
                (id, var)
            })
            .collect::<HashMap<&RecipeId, Variable>>();

        fn mapper(
            variables: &mut ProblemVariables,
            kind: &str,
        ) -> impl FnMut(&ItemId) -> (&ItemId, Variable) {
            move |id| {
                let name = format!("item-{kind}-flow@{}", id.value());
                let var = variables.add(VariableDefinition::new().min(0).name(name));
                (id, var)
            }
        }
        let mut item_production_flow_vars = items
            .iter()
            .map(mapper(&mut variables, "production"))
            .collect::<HashMap<&ItemId, Variable>>();
        let item_consumption_flow_vars = items
            .iter()
            .map(mapper(&mut variables, "consumption"))
            .collect::<HashMap<&ItemId, Variable>>();
        let item_unused_flow_vars = items
            .iter()
            .map(mapper(&mut variables, "unused"))
            .collect::<HashMap<&ItemId, Variable>>();

        let mut item_production_flow_exprs = items
            .iter()
            .map(|id| (id, Expression::default()))
            .collect::<HashMap<&ItemId, Expression>>();
        let mut item_consumption_flow_exprs = items
            .iter()
            .map(|id| (id, Expression::default()))
            .collect::<HashMap<&ItemId, Expression>>();

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

        let values = variables
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

        recipe_replica_vars
            .extract_if(|_, var| approx::relative_eq!(values.value(*var), 0.0, epsilon = 1e-9))
            .for_each(drop);
        item_production_flow_vars
            .extract_if(|_, var| approx::relative_eq!(values.value(*var), 0.0, epsilon = 1e-9))
            .for_each(drop);

        Ok(RawPlanSolution {
            values,
            recipe_replica_vars,
            item_production_flow_vars,
        })
    }

    fn build_plan(
        &self,
        recipes: &HashMap<RecipeId, Recipe>,
        solution: &RawPlanSolution,
        goal: &ItemId,
        flow_goal: Flow,
    ) -> Plan {
        let mut item_producers = HashMap::new();
        solution.recipe_replica_vars.keys().for_each(|id| {
            let recipe = recipes.get(id).unwrap();
            recipe.products().iter().for_each(|(item, _)| {
                let ps = item_producers.entry(item).or_insert(Vec::new());
                ps.push(recipe);
            });
        });

        let context = BuildPlanContext {
            item_producers: &item_producers,
            solution,
        };
        let mut trace = Vec::new();
        let mut common_intermediates = HashMap::new();
        let mut common_recipes = HashMap::new();

        let goal = self.build_plan_item_node(
            &context,
            &mut trace,
            &mut common_intermediates,
            &mut common_recipes,
            goal,
            flow_goal,
        );
        Plan::new(goal, common_intermediates, common_recipes)
    }

    fn build_plan_item_node<'a>(
        &self,
        context: &BuildPlanContext<'a>,
        trace: &mut Vec<BuildPlanTrace<'a>>,
        common_intermediates: &mut HashMap<ItemId, PlanItemNode>,
        common_recipes: &mut HashMap<RecipeId, PlanRecipeNode>,
        target: &'a ItemId,
        flow_target: Flow,
    ) -> PlanItemNode {
        let current_len = trace.len();
        if common_intermediates.contains_key(target) {
            return PlanItemNode::Partial(
                PartialPlanItemNode::builder()
                    .target(target.clone())
                    .flow_next(flow_target)
                    .build()
                    .unwrap(),
            );
        } else if let Some(frame) = trace.iter_mut().rfind(|frame| frame.is_item(target)) {
            let BuildPlanTrace::Item { flow_cyclic, .. } = frame else {
                unreachable!();
            };
            *flow_cyclic = *flow_cyclic + flow_target;

            let target_depth = frame.depth();
            return PlanItemNode::Cyclic(
                CyclicPlanItemNode::builder()
                    .target(target.clone())
                    .flow_next(flow_target)
                    .from_steps_ahead(current_len - target_depth)
                    .build()
                    .unwrap(),
            );
        }

        trace.push(BuildPlanTrace::Item {
            depth: trace.len(),
            item: target,
            flow_cyclic: Flow::zero(),
        });

        let producers = context.item_producers.get(target).unwrap();
        let node = if self.should_build_normal_plan_item_node(producers) {
            self.build_normal_plan_item_node(
                context,
                trace,
                common_intermediates,
                common_recipes,
                target,
                flow_target,
                producers.first().unwrap(),
            )
        } else {
            self.build_aggregated_plan_item_node(
                context,
                trace,
                common_intermediates,
                common_recipes,
                target,
                flow_target,
                &producers,
            )
        };

        trace.pop();
        node
    }

    fn should_build_normal_plan_item_node(&self, producers: &[&Recipe]) -> bool {
        let unique_producer = producers.len() == 1;
        let no_byproduct = producers.first().map_or(false, |p| p.products().len() == 1);
        unique_producer && no_byproduct
    }

    fn build_normal_plan_item_node<'a>(
        &self,
        context: &BuildPlanContext<'a>,
        trace: &mut Vec<BuildPlanTrace<'a>>,
        common_intermediates: &mut HashMap<ItemId, PlanItemNode>,
        common_recipes: &mut HashMap<RecipeId, PlanRecipeNode>,
        target: &'a ItemId,
        flow_target: Flow,
        producer: &'a Recipe,
    ) -> PlanItemNode {
        let solution = context.solution;

        let replica_var = *solution.recipe_replica_vars.get(producer.id()).unwrap();
        let replica = Replica::new(solution.values.value(replica_var))
            .expect("the remaining replica's value should be positive");

        let flow_all_var = *solution.item_production_flow_vars.get(target).unwrap();
        let flow_all = Flow::new(solution.values.value(flow_all_var))
            .expect("the remaining item production flow's value should be positive");

        let mut dependencies = Vec::with_capacity(producer.materials().len());
        for (material, flow_material) in producer.get_materials_flow(replica) {
            let node = self.build_plan_item_node(
                context,
                trace,
                common_intermediates,
                common_recipes,
                material,
                flow_material,
            );
            dependencies.push(node);
        }

        let flow_cyclic = match trace.last() {
            Some(BuildPlanTrace::Item { flow_cyclic, .. }) => *flow_cyclic,
            _ => unreachable!("the last trace frame should be `BuildPlanTrace::Item`"),
        };
        let flow_next = Flow::new(flow_all.value() - flow_cyclic.value())
            .expect("`flow_all` should be greater than `flow_cyclic`");

        let node = PlanItemNode::Normal(
            NormalPlanItemNode::builder()
                .target(target.clone())
                .recipe(producer.id().clone())
                .flow_next(flow_next)
                .flow_cyclic(flow_cyclic)
                .replica(replica)
                .dependencies(dependencies)
                .build()
                .unwrap(),
        );
        if node.flow_next() > flow_target {
            common_intermediates.insert(target.clone(), node);
            PlanItemNode::Partial(
                PartialPlanItemNode::builder()
                    .target(target.clone())
                    .flow_next(flow_target)
                    .build()
                    .unwrap(),
            )
        } else {
            node
        }
    }

    fn build_aggregated_plan_item_node<'a>(
        &self,
        context: &BuildPlanContext<'a>,
        trace: &mut Vec<BuildPlanTrace<'a>>,
        common_intermediates: &mut HashMap<ItemId, PlanItemNode>,
        common_recipes: &mut HashMap<RecipeId, PlanRecipeNode>,
        target: &'a ItemId,
        flow_target: Flow,
        producers: &[&'a Recipe],
    ) -> PlanItemNode {
        let solution = context.solution;

        let flow_all_var = *solution.item_production_flow_vars.get(target).unwrap();
        let flow_all = Flow::new(solution.values.value(flow_all_var))
            .expect("the remaining item production flow's value should be positive");

        let mut recipes = Vec::with_capacity(producers.len());
        for producer in producers {
            let node = self.build_plan_recipe_node(
                context,
                trace,
                common_intermediates,
                common_recipes,
                producer,
            );
            recipes.push(node);
        }

        let flow_cyclic = match trace.last() {
            Some(BuildPlanTrace::Item { flow_cyclic, .. }) => *flow_cyclic,
            _ => unreachable!("the last trace frame should be `BuildPlanTrace::Item`"),
        };
        let flow_next = Flow::new(flow_all.value() - flow_cyclic.value())
            .expect("`flow_all` should be greater than `flow_cyclic`");

        let node = PlanItemNode::Aggregated(
            AggregatedPlanItemNode::builder()
                .target(target.clone())
                .flow_next(flow_next)
                .flow_cyclic(flow_cyclic)
                .recipes(recipes)
                .build()
                .unwrap(),
        );
        if node.flow_next() > flow_target {
            common_intermediates.insert(target.clone(), node);
            PlanItemNode::Partial(
                PartialPlanItemNode::builder()
                    .target(target.clone())
                    .flow_next(flow_target)
                    .build()
                    .unwrap(),
            )
        } else {
            node
        }
    }

    fn build_plan_recipe_node<'a>(
        &self,
        context: &BuildPlanContext<'a>,
        trace: &mut Vec<BuildPlanTrace<'a>>,
        common_intermediates: &mut HashMap<ItemId, PlanItemNode>,
        common_recipes: &mut HashMap<RecipeId, PlanRecipeNode>,
        producer: &'a Recipe,
    ) -> PlanRecipeNode {
        let current_len = trace.len();

        let solution = context.solution;
        let replica_var = *solution.recipe_replica_vars.get(producer.id()).unwrap();
        let replica = Replica::new(solution.values.value(replica_var))
            .expect("the remaining replica's value should be positive");

        if common_recipes.contains_key(producer.id()) {
            return PlanRecipeNode::Partial(
                PartialPlanRecipeNode::builder()
                    .recipe(producer.id().clone())
                    .replica(replica)
                    .build()
                    .unwrap(),
            );
        } else if let Some(frame) = trace
            .iter_mut()
            .rfind(|frame| frame.is_recipe(producer.id()))
        {
            let recipe_depth = frame.depth();
            return PlanRecipeNode::Cyclic(
                CyclicPlanRecipeNode::builder()
                    .recipe(producer.id().clone())
                    .replica(replica)
                    .from_steps_ahead(current_len - recipe_depth)
                    .build()
                    .unwrap(),
            );
        }

        trace.push(BuildPlanTrace::Recipe {
            depth: trace.len(),
            recipe: producer.id(),
        });

        let node = self.build_normal_plan_recipe_node(
            context,
            trace,
            common_intermediates,
            common_recipes,
            producer,
            replica,
        );

        trace.pop();
        node
    }

    fn build_normal_plan_recipe_node<'a>(
        &self,
        context: &BuildPlanContext<'a>,
        trace: &mut Vec<BuildPlanTrace<'a>>,
        common_intermediates: &mut HashMap<ItemId, PlanItemNode>,
        common_recipes: &mut HashMap<RecipeId, PlanRecipeNode>,
        producer: &'a Recipe,
        replica: Replica,
    ) -> PlanRecipeNode {
        let mut materials = Vec::with_capacity(producer.materials().len());
        for (material, flow_material) in producer.get_materials_flow(replica) {
            let node = self.build_plan_item_node(
                context,
                trace,
                common_intermediates,
                common_recipes,
                material,
                flow_material,
            );
            materials.push(node);
        }

        let node = PlanRecipeNode::Normal(
            NormalPlanRecipeNode::builder()
                .recipe(producer.id().clone())
                .replica(replica)
                .materials(materials)
                .build()
                .unwrap(),
        );
        if producer.products().len() > 1 {
            common_recipes.insert(producer.id().clone(), node);
            PlanRecipeNode::Partial(
                PartialPlanRecipeNode::builder()
                    .recipe(producer.id().clone())
                    .replica(replica)
                    .build()
                    .unwrap(),
            )
        } else {
            node
        }
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

struct RawPlanSolution<'a> {
    values: MicroLpSolution,
    recipe_replica_vars: HashMap<&'a RecipeId, Variable>,
    item_production_flow_vars: HashMap<&'a ItemId, Variable>,
}

#[derive(Clone)]
struct BuildPlanContext<'a> {
    item_producers: &'a HashMap<&'a ItemId, Vec<&'a Recipe>>,
    solution: &'a RawPlanSolution<'a>,
}

#[derive(Debug, Clone, PartialEq)]
enum BuildPlanTrace<'a> {
    Item {
        depth: usize,
        item: &'a ItemId,
        flow_cyclic: Flow,
    },
    Recipe {
        depth: usize,
        recipe: &'a RecipeId,
    },
}

impl<'a> BuildPlanTrace<'a> {
    fn depth(&self) -> usize {
        match self {
            Self::Item { depth, .. } => *depth,
            Self::Recipe { depth, .. } => *depth,
        }
    }

    fn is_item(&self, id: &ItemId) -> bool {
        match self {
            Self::Item { item, .. } => *item == id,
            _ => false,
        }
    }

    fn is_recipe(&self, id: &RecipeId) -> bool {
        match self {
            Self::Recipe { recipe, .. } => *recipe == id,
            _ => false,
        }
    }
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
            assert_eq!(variant.replica(), Replica::new(1.0).unwrap());
            assert_eq!(variant.flow_next(), Flow::new(6.0).unwrap());
        });
        visit_node_normal(&plan, &["i1", "i2"], |variant| {
            assert_eq!(variant.replica(), Replica::new(2.0).unwrap());
            assert_eq!(variant.flow_next(), Flow::new(60.0).unwrap());
        });
        visit_node_partial(&plan, &["i1", "i2", "i4"], |variant| {
            assert_eq!(variant.flow_next(), Flow::new(120.0).unwrap());
        });
        visit_node_normal(&plan, &["i1", "i3"], |variant| {
            assert_eq!(variant.replica(), Replica::new(1.0).unwrap());
            assert_eq!(variant.flow_next(), Flow::new(30.0).unwrap());
        });
        visit_node_partial(&plan, &["i1", "i3", "i4"], |variant| {
            assert_eq!(variant.flow_next(), Flow::new(30.0).unwrap());
        });
        visit_node_normal(&plan, &["i4"], |variant| {
            assert_eq!(variant.replica(), Replica::new(2.5).unwrap());
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
            assert_eq!(variant.replica(), Replica::new(1.0).unwrap());
            assert_eq!(variant.flow_next(), Flow::new(60.0).unwrap());
        });
        visit_node_normal(&plan, &["i1", "i2"], |variant| {
            assert_eq!(variant.replica(), Replica::new(1.0).unwrap());
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
