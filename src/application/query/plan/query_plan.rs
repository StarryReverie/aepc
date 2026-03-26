use std::backtrace::Backtrace;

use anyhow::Error as AnyhowError;
use getset::{CopyGetters, Getters};
use snafu::prelude::*;

use crate::application::query::plan::PlanQueryServiceImpl;
use crate::domain::item::model::{Item, ItemId, ItemName};
use crate::domain::item::outbound::ItemRepository;
use crate::domain::machine::model::{Machine, MachineId, MachineName, Power};
use crate::domain::machine::outbound::MachineRepository;
use crate::domain::plan::model::{
    AggregatedPlanItemNode, CyclicPlanItemNode, CyclicPlanRecipeNode, NormalPlanItemNode,
    NormalPlanRecipeNode, PartialPlanItemNode, PartialPlanRecipeNode, Plan, PlanItemNode,
    PlanRecipeNode,
};
use crate::domain::plan::service::{CreatePlanError, PlanFactory};
use crate::domain::recipe::model::{Flow, Recipe, RecipeId, Replica};
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
    common_recipes: Vec<PlanDetailNode>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlanDetailNode {
    Combined {
        target: PlanTargetDetail,
        recipe: PlanRecipeDetail,
        children: Vec<PlanDetailNode>,
    },
    Target {
        target: PlanTargetDetail,
        children: Vec<PlanDetailNode>,
    },
    Recipe {
        recipe: PlanRecipeDetail,
        children: Vec<PlanDetailNode>,
    },
}

impl PlanDetailNode {
    pub fn children(&self) -> &[PlanDetailNode] {
        match self {
            Self::Combined { children, .. } => &children,
            Self::Target { children, .. } => &children,
            Self::Recipe { children, .. } => &children,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Getters, CopyGetters)]
pub struct PlanTargetDetail {
    #[getset(get = "pub")]
    target_id: ItemId,
    #[getset(get = "pub")]
    target_name: ItemName,
    #[getset(get_copy = "pub")]
    flow_all: Flow,
    #[getset(get_copy = "pub")]
    flow_next: Flow,
    #[getset(get_copy = "pub")]
    flow_cyclic: Flow,
    #[getset(get_copy = "pub")]
    from_steps_ahead: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, Getters, CopyGetters)]
pub struct PlanRecipeDetail {
    #[getset(get = "pub")]
    recipe_id: RecipeId,
    #[getset(get = "pub")]
    machine_id: MachineId,
    #[getset(get = "pub")]
    machine_name: MachineName,
    #[getset(get_copy = "pub")]
    machine_power: Option<Power>,
    #[getset(get_copy = "pub")]
    replica: Replica,
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

        let plan_detail = self.convert_plan(&plan).await?;

        Ok(QueryPlanResponse { plan: plan_detail })
    }

    async fn convert_plan(&self, plan: &Plan) -> Result<PlanDetail, QueryPlanError> {
        let goal_node = plan.goal();
        let goal = self.convert_item_node(goal_node).await?;

        let mut common_intermediates = Vec::new();
        for node in plan.common_intermediates().values() {
            let detail_node = self.convert_item_node(node).await?;
            common_intermediates.push(detail_node);
        }

        let mut common_recipes = Vec::new();
        for node in plan.common_recipes().values() {
            let detail_node = self.convert_recipe_node(node).await?;
            common_recipes.push(detail_node);
        }

        Ok(PlanDetail {
            goal,
            common_intermediates,
            common_recipes,
        })
    }

    async fn convert_item_node(
        &self,
        node: &PlanItemNode,
    ) -> Result<PlanDetailNode, QueryPlanError> {
        match node {
            PlanItemNode::Normal(node) => self.convert_normal_item_node(node).await,
            PlanItemNode::Aggregated(node) => self.convert_aggregated_item_node(node).await,
            PlanItemNode::Partial(node) => self.convert_partial_item_node(node).await,
            PlanItemNode::Cyclic(node) => self.convert_cyclic_item_node(node).await,
        }
    }

    async fn convert_normal_item_node(
        &self,
        node: &NormalPlanItemNode,
    ) -> Result<PlanDetailNode, QueryPlanError> {
        let target = self.get_item_entity(node.target()).await?;
        let recipe = self.get_recipe_entity(node.recipe()).await?;
        let machine = self.get_machine_entity(recipe.machine()).await?;

        let mut dependencies = Vec::new();
        for dep in node.dependencies() {
            dependencies.push(Box::pin(self.convert_item_node(dep)).await?);
        }

        let replica = node.replica();
        let machine_power = Power::new(machine.power().value() * replica.value()).unwrap();

        Ok(PlanDetailNode::Combined {
            target: PlanTargetDetail {
                target_id: target.id().clone(),
                target_name: target.name().clone(),
                flow_all: node.flow_next() + node.flow_cyclic(),
                flow_next: node.flow_next(),
                flow_cyclic: node.flow_cyclic(),
                from_steps_ahead: None,
            },
            recipe: PlanRecipeDetail {
                recipe_id: recipe.id().clone(),
                machine_id: machine.id().clone(),
                machine_name: machine.name().clone(),
                machine_power: Some(machine_power),
                replica,
            },
            children: dependencies,
        })
    }

    async fn convert_aggregated_item_node(
        &self,
        node: &AggregatedPlanItemNode,
    ) -> Result<PlanDetailNode, QueryPlanError> {
        let target = self.get_item_entity(node.target()).await?;

        let mut children = Vec::new();
        for recipe in node.recipes() {
            let recipe_detail_node = self.convert_recipe_node(recipe).await?;
            children.push(recipe_detail_node);
        }

        Ok(PlanDetailNode::Target {
            target: PlanTargetDetail {
                target_id: target.id().clone(),
                target_name: target.name().clone(),
                flow_all: node.flow_next() + node.flow_cyclic(),
                flow_next: node.flow_next(),
                flow_cyclic: node.flow_cyclic(),
                from_steps_ahead: None,
            },
            children,
        })
    }

    async fn convert_partial_item_node(
        &self,
        node: &PartialPlanItemNode,
    ) -> Result<PlanDetailNode, QueryPlanError> {
        let target = self.get_item_entity(node.target()).await?;
        Ok(PlanDetailNode::Target {
            target: PlanTargetDetail {
                target_id: target.id().clone(),
                target_name: target.name().clone(),
                flow_all: node.flow_next(),
                flow_next: node.flow_next(),
                flow_cyclic: Flow::zero(),
                from_steps_ahead: None,
            },
            children: Vec::new(),
        })
    }

    async fn convert_cyclic_item_node(
        &self,
        node: &CyclicPlanItemNode,
    ) -> Result<PlanDetailNode, QueryPlanError> {
        let target = self.get_item_entity(node.target()).await?;
        Ok(PlanDetailNode::Target {
            target: PlanTargetDetail {
                target_id: target.id().clone(),
                target_name: target.name().clone(),
                flow_all: node.flow_next(),
                flow_next: node.flow_next(),
                flow_cyclic: Flow::zero(),
                from_steps_ahead: Some(node.from_steps_ahead()),
            },
            children: Vec::new(),
        })
    }

    async fn convert_recipe_node(
        &self,
        node: &PlanRecipeNode,
    ) -> Result<PlanDetailNode, QueryPlanError> {
        match node {
            PlanRecipeNode::Normal(node) => self.convert_normal_recipe_node(node).await,
            PlanRecipeNode::Partial(node) => self.convert_partial_recipe_node(node).await,
            PlanRecipeNode::Cyclic(node) => self.convert_cyclic_recipe_node(node).await,
        }
    }

    async fn convert_normal_recipe_node(
        &self,
        node: &NormalPlanRecipeNode,
    ) -> Result<PlanDetailNode, QueryPlanError> {
        let recipe = self.get_recipe_entity(node.recipe()).await?;
        let machine = self.get_machine_entity(recipe.machine()).await?;

        let mut children = Vec::new();
        for material in node.materials() {
            children.push(Box::pin(self.convert_item_node(material)).await?);
        }

        let replica = node.replica();
        let machine_power = Power::new(machine.power().value() * replica.value()).unwrap();

        Ok(PlanDetailNode::Recipe {
            recipe: PlanRecipeDetail {
                recipe_id: recipe.id().clone(),
                machine_id: machine.id().clone(),
                machine_name: machine.name().clone(),
                machine_power: Some(machine_power),
                replica,
            },
            children,
        })
    }

    async fn convert_partial_recipe_node(
        &self,
        node: &PartialPlanRecipeNode,
    ) -> Result<PlanDetailNode, QueryPlanError> {
        let recipe = self.get_recipe_entity(node.recipe()).await?;
        let machine = self.get_machine_entity(recipe.machine()).await?;

        let replica = node.replica();
        let machine_power = Power::new(machine.power().value() * replica.value()).unwrap();

        Ok(PlanDetailNode::Recipe {
            recipe: PlanRecipeDetail {
                recipe_id: recipe.id().clone(),
                machine_id: machine.id().clone(),
                machine_name: machine.name().clone(),
                machine_power: Some(machine_power),
                replica,
            },
            children: Vec::new(),
        })
    }

    async fn convert_cyclic_recipe_node(
        &self,
        node: &CyclicPlanRecipeNode,
    ) -> Result<PlanDetailNode, QueryPlanError> {
        let recipe = self.get_recipe_entity(node.recipe()).await?;
        let machine = self.get_machine_entity(recipe.machine()).await?;

        let replica = node.replica();
        let machine_power = Power::new(machine.power().value() * replica.value()).unwrap();

        Ok(PlanDetailNode::Recipe {
            recipe: PlanRecipeDetail {
                recipe_id: recipe.id().clone(),
                machine_id: machine.id().clone(),
                machine_name: machine.name().clone(),
                machine_power: Some(machine_power),
                replica,
            },
            children: Vec::new(),
        })
    }

    async fn get_item_entity(&self, id: &ItemId) -> Result<Item, QueryPlanError> {
        let item = (self.item_repository.get(id).await)
            .context(InfrastructureSnafu {
                message: format!("failed to fetch item {:?}", id),
            })?
            .context(NotFoundSnafu {
                entity: format!("item {:?}", id),
            })?;
        Ok(item)
    }

    async fn get_recipe_entity(&self, id: &RecipeId) -> Result<Recipe, QueryPlanError> {
        let recipe = (self.recipe_repository.get(id).await)
            .context(InfrastructureSnafu {
                message: format!("failed to fetch recipe {:?}", id),
            })?
            .context(NotFoundSnafu {
                entity: format!("recipe {:?}", id),
            })?;
        Ok(recipe)
    }

    async fn get_machine_entity(&self, id: &MachineId) -> Result<Machine, QueryPlanError> {
        let machine = (self.machine_repository.get(id).await)
            .context(InfrastructureSnafu {
                message: format!("failed to fetch machine {:?}", id),
            })?
            .context(NotFoundSnafu {
                entity: format!("machine {:?}", id),
            })?;
        Ok(machine)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use unimock::*;

    use crate::domain::item::model::{Item, ItemId, test_helper::make_item};
    use crate::domain::item::outbound::{DynItemRepository, test_helper::ItemRepositoryMock};
    use crate::domain::machine::model::{Machine, test_helper::make_machine};
    use crate::domain::machine::outbound::{
        DynMachineRepository, test_helper::MachineRepositoryMock,
    };
    use crate::domain::plan::model::{NormalPlanItemNode, Plan, PlanItemNode};
    use crate::domain::plan::service::{DynPlanFactory, test_helper::PlanFactoryMock};
    use crate::domain::recipe::model::{Flow, Recipe, Replica, test_helper::make_recipe};
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
                    .target(item1().id().clone())
                    .recipe(recipe1().id().clone())
                    .flow_next(recipe1().get_product_rate(item1().id()).unwrap() * Replica::one())
                    .flow_cyclic(Flow::zero())
                    .replica(Replica::one())
                    .dependencies(vec![])
                    .build()
                    .unwrap(),
            );
            Plan::new(goal_node, HashMap::new(), HashMap::new())
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

        let PlanDetailNode::Combined { target, recipe, .. } = plan_detail.goal() else {
            unreachable!();
        };
        assert_eq!(target.target_id(), item1().id());
        assert_eq!(target.target_name(), item1().name());
        assert_eq!(target.flow_all(), Flow::new(1.0).unwrap());
        assert_eq!(
            target.flow_next(),
            Recipe::get_product_rate(&recipe1(), item1().id()).unwrap() * Replica::one(),
        );
        assert_eq!(target.from_steps_ahead(), None);

        assert_eq!(recipe.recipe_id(), recipe1().id());
        assert_eq!(recipe.machine_id(), machine1().id());
        assert_eq!(recipe.machine_name(), machine1().name());
        assert_eq!(recipe.machine_power(), Some(machine1().power()));
    }
}
