#![expect(unused)]
use std::backtrace::Backtrace;

use anyhow::Error as AnyhowError;
use getset::{CopyGetters, Getters};
use snafu::prelude::*;

use crate::application::query::plan::PlanQueryServiceImpl;
use crate::domain::item::model::{ItemId, ItemName};
use crate::domain::item::outbound::ItemRepository;
use crate::domain::machine::model::{MachineId, MachineName, Power};
use crate::domain::machine::outbound::MachineRepository;
use crate::domain::plan::model::{Plan, PlanItemNode};
use crate::domain::plan::service::{CreatePlanError, PlanFactory};
use crate::domain::recipe::model::{Flow, Rate, Recipe, RecipeId, Replica};
use crate::domain::recipe::outbound::RecipeRepository;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueryPlanRequest {
    pub goal: ItemId,
    pub expected_flow: Flow,
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryPlanResponse {
    pub plan: PlanDetail,
}

#[derive(Debug, Snafu)]
#[non_exhaustive]
pub enum QueryPlanError {
    #[snafu(display("could not find a planning solution"))]
    Plan { source: CreatePlanError },
    #[snafu(display("entity not found: {entity}"))]
    NotFound {
        entity: String,
        backtrace: Backtrace,
    },
    #[snafu(display("infrastructure error when querying plan: {message}"))]
    Infrastructure {
        message: String,
        source: AnyhowError,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Getters)]
#[getset(get = "pub")]
pub struct PlanDetail {
    goal: PlanDetailNode,
    common_intermediates: Vec<PlanDetailNode>,
}

#[derive(Debug, Clone, PartialEq, Eq, Getters, CopyGetters)]
pub struct PlanDetailNode {
    #[getset(get = "pub")]
    target_id: ItemId,
    #[getset(get = "pub")]
    target_name: ItemName,
    #[getset(get = "pub")]
    recipe_id: RecipeId,
    #[getset(get = "pub")]
    machine_id: MachineId,
    #[getset(get = "pub")]
    machine_name: MachineName,
    #[getset(get_copy = "pub")]
    machine_power: Option<Power>,
    #[getset(get_copy = "pub")]
    flow_all: Flow,
    #[getset(get_copy = "pub")]
    flow_next: Flow,
    #[getset(get_copy = "pub")]
    flow_extra: Option<Flow>,
    #[getset(get_copy = "pub")]
    rate: Option<Rate>,
    #[getset(get_copy = "pub")]
    replica_next: Option<Replica>,
    #[getset(get_copy = "pub")]
    replica_cyclic: Option<Replica>,
    #[getset(get_copy = "pub")]
    from_steps_ahead: Option<usize>,
    #[getset(get = "pub")]
    dependencies: Vec<PlanDetailNode>,
}

impl PlanQueryServiceImpl {
    pub(super) async fn query_plan_impl(
        &self,
        request: QueryPlanRequest,
    ) -> Result<QueryPlanResponse, QueryPlanError> {
        let plan = self
            .plan_factory
            .create_plan(&request.goal, request.expected_flow)
            .await
            .context(PlanSnafu)?;

        let plan_detail = self.convert_plan_to_detail(&plan).await?;

        Ok(QueryPlanResponse { plan: plan_detail })
    }

    async fn convert_plan_to_detail(&self, plan: &Plan) -> Result<PlanDetail, QueryPlanError> {
        let goal_node = plan.goal();
        let goal = self.convert_node_to_detail(goal_node, plan).await?;

        let mut common_intermediates = Vec::new();
        for node in plan.common_intermediates().values() {
            let detail_node = self.convert_node_to_detail(node, plan).await?;
            common_intermediates.push(detail_node);
        }

        Ok(PlanDetail {
            goal,
            common_intermediates,
        })
    }

    async fn convert_node_to_detail(
        &self,
        node: &PlanItemNode,
        plan: &Plan,
    ) -> Result<PlanDetailNode, QueryPlanError> {
        let target_id = node.target().clone();
        let target = self
            .item_repository
            .get(&target_id)
            .await
            .context(InfrastructureSnafu {
                message: format!("failed to fetch item {:?}", target_id),
            })?
            .context(NotFoundSnafu {
                entity: format!("item {:?}", target_id),
            })?;
        let target_name = target.name().clone();

        let recipe_id = node.recipe().clone();
        let recipe = self
            .recipe_repository
            .get(&recipe_id)
            .await
            .context(InfrastructureSnafu {
                message: format!("failed to fetch recipe {:?}", recipe_id),
            })?
            .context(NotFoundSnafu {
                entity: format!("recipe {:?}", recipe_id),
            })?;

        let machine_id = recipe.machine().clone();
        let machine = self
            .machine_repository
            .get(&machine_id)
            .await
            .context(InfrastructureSnafu {
                message: format!("failed to fetch machine {:?}", machine_id),
            })?
            .context(NotFoundSnafu {
                entity: format!("machine {:?}", machine_id),
            })?;
        let machine_name = machine.name().clone();
        let machine_base_power = machine.power();

        let flow_next = node.flow_next();

        match node {
            PlanItemNode::Normal(variant) => {
                let mut dependencies = Vec::new();
                for dep in variant.dependencies() {
                    dependencies.push(Box::pin(self.convert_node_to_detail(dep, plan)).await?);
                }
                dependencies.sort_by(|a, b| a.target_name().cmp(b.target_name()));

                let replica_cyclic = variant.replica_cyclic();
                let flow_cyclic = replica_cyclic.map_or(Flow::zero(), |r| variant.rate() * r);
                let flow_all = flow_next + flow_cyclic + variant.flow_extra();

                let replica_all = match replica_cyclic {
                    Some(cyclic) => variant.replica_next() + cyclic,
                    None => variant.replica_next(),
                };
                let machine_power =
                    Some(Power::new(machine_base_power.value() * replica_all.value()).unwrap());

                Ok(PlanDetailNode {
                    target_id,
                    target_name,
                    recipe_id,
                    machine_id,
                    machine_name,
                    machine_power,
                    flow_all,
                    flow_next,
                    flow_extra: Some(variant.flow_extra()),
                    rate: Some(variant.rate()),
                    replica_next: Some(variant.replica_next()),
                    replica_cyclic,
                    from_steps_ahead: None,
                    dependencies,
                })
            }
            PlanItemNode::Partial(_) => Ok(PlanDetailNode {
                target_id,
                target_name,
                recipe_id,
                machine_id,
                machine_name,
                machine_power: None,
                flow_all: flow_next,
                flow_next,
                flow_extra: None,
                rate: None,
                replica_next: None,
                replica_cyclic: None,
                from_steps_ahead: None,
                dependencies: vec![],
            }),
            PlanItemNode::Cyclic(variant) => Ok(PlanDetailNode {
                target_id,
                target_name,
                recipe_id,
                machine_id,
                machine_name,
                machine_power: None,
                flow_all: flow_next,
                flow_next,
                flow_extra: None,
                rate: None,
                replica_next: None,
                replica_cyclic: None,
                from_steps_ahead: Some(variant.from_steps_ahead()),
                dependencies: vec![],
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use unimock::*;

    use crate::domain::item::model::{Item, ItemId, test_helper::make_item};
    use crate::domain::item::outbound::{DynItemRepository, test_helper::ItemRepositoryMock};
    use crate::domain::machine::model::{Machine, MachineId, test_helper::make_machine};
    use crate::domain::machine::outbound::{
        DynMachineRepository, test_helper::MachineRepositoryMock,
    };
    use crate::domain::plan::model::{Plan, PlanItemNode, NormalPlanItemNode};
    use crate::domain::plan::service::{DynPlanFactory, test_helper::PlanFactoryMock};
    use crate::domain::recipe::model::{Flow, Recipe, RecipeId, Replica, test_helper::make_recipe};
    use crate::domain::recipe::outbound::{DynRecipeRepository, test_helper::RecipeRepositoryMock};

    use super::*;

    #[tokio::test]
    async fn test_query_plan_with_single_normal_node() {
        fn item1() -> Item {
            make_item("i1", "Item 1")
        }
        fn machine1() -> Machine {
            make_machine("m1", "Machine 1", 100.0)
        }
        fn recipe1() -> Recipe {
            make_recipe("r1", "m1", 60.0, vec![], vec![("i1", 1.0)])
        }
        fn plan() -> Plan {
            let goal_node = PlanItemNode::Normal(
                NormalPlanItemNode::builder()
                    .target(ItemId::new("i1").unwrap())
                    .recipe(RecipeId::new("r1").unwrap())
                    .rate(Recipe::get_product_rate(&recipe1(), item1().id()).unwrap())
                    .replica_next(Replica::new(1.0).unwrap())
                    .replica_cyclic(None)
                    .flow_extra(Flow::zero())
                    .dependencies(vec![])
                    .build()
                    .unwrap(),
            );
            Plan::new(goal_node, HashMap::new())
        }

        let item_repo =
            DynItemRepository::new_arc(Unimock::new(ItemRepositoryMock::get.stub(|each| {
                each.call(matching!((id) if *id == item1().id()))
                    .answers(&|_, _| Ok(Some(item1())));
            })));
        let machine_repo =
            DynMachineRepository::new_arc(Unimock::new(MachineRepositoryMock::get.stub(|each| {
                each.call(matching!((id) if *id == machine1().id()))
                    .answers(&|_, _| Ok(Some(machine1())));
            })));
        let recipe_repo =
            DynRecipeRepository::new_arc(Unimock::new(RecipeRepositoryMock::get.stub(|each| {
                each.call(matching!((id) if *id == recipe1().id()))
                    .answers(&|_, _| Ok(Some(recipe1())));
            })));
        let plan_factory =
            DynPlanFactory::new_arc(Unimock::new(PlanFactoryMock::create_plan.stub(|each| {
                each.call(matching!()).answers(&|_, _, _| Ok(plan()));
            })));
        let service = PlanQueryServiceImpl::new(item_repo, machine_repo, recipe_repo, plan_factory);

        let request = QueryPlanRequest {
            goal: ItemId::new("i1").unwrap(),
            expected_flow: Flow::new(1.0).unwrap(),
        };

        let response = service.query_plan_impl(request).await.unwrap();
        let plan_detail = response.plan;

        let goal = plan_detail.goal();
        assert_eq!(goal.target_id(), item1().id());
        assert_eq!(goal.target_name(), item1().name());
        assert_eq!(goal.recipe_id(), recipe1().id());
        assert_eq!(goal.machine_id(), machine1().id());
        assert_eq!(goal.machine_name(), machine1().name());
        assert_eq!(
            goal.flow_next(),
            Recipe::get_product_rate(&recipe1(), item1().id()).unwrap()
                * Replica::new(1.0).unwrap()
        );
        assert_eq!(goal.flow_all(), Flow::new(1.0).unwrap());
        assert_eq!(goal.flow_extra(), Some(Flow::zero()));
        assert_eq!(goal.machine_power(), Some(machine1().power()));
        assert_eq!(
            goal.rate(),
            Some(Recipe::get_product_rate(&recipe1(), item1().id()).unwrap())
        );
        assert_eq!(goal.replica_next(), Some(Replica::new(1.0).unwrap()));
        assert_eq!(goal.replica_cyclic(), None);
        assert_eq!(goal.from_steps_ahead(), None);
    }
}
