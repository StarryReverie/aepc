use anyhow::Error as AnyhowError;
use getset::Getters;
use snafu::prelude::*;

use crate::application::query::plan::PlanQueryServiceImpl;
use crate::domain::item::model::{ItemId, ItemName};
use crate::domain::item::outbound::ItemRepository;
use crate::domain::machine::model::{MachineId, MachineName, Power};
use crate::domain::machine::outbound::MachineRepository;
use crate::domain::plan::model::{CyclicStep, NormalStep, Plan};
use crate::domain::plan::service::CreatePlanError;
use crate::domain::recipe::model::{Flow, Recipe, RecipeId, Replica};
use crate::domain::recipe::outbound::RecipeRepository;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueryPlanRequest {
    pub goal: ItemId,
    pub expected_flow: Flow,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueryPlanResponse {
    pub plan: PlanDetail,
}

#[derive(Debug, Snafu)]
#[non_exhaustive]
pub enum QueryPlanError {
    #[snafu(display("could not find a planning solution"))]
    Plan { source: CreatePlanError },
    #[snafu(display("entity not found: {entity}"))]
    NotFound { entity: String },
    #[snafu(display("query plan error from infrastructure: {message}"))]
    Infrastructure {
        message: String,
        source: AnyhowError,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Getters)]
#[getset(get = "pub")]
pub struct PlanDetail {
    goal_id: ItemId,
    goal_name: ItemName,
    recipe_id: RecipeId,
    machine_id: MachineId,
    machine_name: MachineName,
    machine_power: Power,
    flow_effective: Flow,
    flow_backward: Option<Flow>,
    replica_effective: Replica,
    replica_backward: Option<Replica>,
    cyclic_steps_ahead: Option<u32>,
    depth: u32,
    dependencies: Vec<PlanDetail>,
}

impl PlanQueryServiceImpl {
    pub(super) async fn query_plan_impl(
        &self,
        request: QueryPlanRequest,
    ) -> Result<QueryPlanResponse, QueryPlanError> {
        let plan = (self.plan_factory)
            .create_plan(&request.goal, request.expected_flow)
            .await
            .context(PlanSnafu)?;

        let plan_detail = self
            .convert_plan_to_detail(&plan, &request.expected_flow, 0)
            .await?;

        Ok(QueryPlanResponse { plan: plan_detail })
    }

    async fn convert_plan_to_detail(
        &self,
        plan: &Plan,
        flow_goal: &Flow,
        depth: u32,
    ) -> Result<PlanDetail, QueryPlanError> {
        match plan {
            Plan::Normal { step, dependencies } => {
                self.convert_normal_plan_to_detail(step, dependencies, depth)
                    .await
            }
            Plan::Cyclic { step } => {
                self.convert_cyclic_plan_to_detail(step, flow_goal, depth)
                    .await
            }
        }
    }

    async fn convert_normal_plan_to_detail(
        &self,
        step: &NormalStep,
        dependencies: &[Plan],
        depth: u32,
    ) -> Result<PlanDetail, QueryPlanError> {
        let goal_id = step.goal();
        let recipe_id = step.recipe();
        let replica_effective = step.replica_effective();
        let replica_backward = step.replica_backward();

        let item = (self.item_repository.get(goal_id).await)
            .context(InfrastructureSnafu {
                message: format!("could not get item: {goal_id:?}"),
            })?
            .context(NotFoundSnafu {
                entity: format!("item {goal_id:?}"),
            })?;

        let recipe = (self.recipe_repository.get(recipe_id).await)
            .context(InfrastructureSnafu {
                message: format!("could not get recipe: {recipe_id:?}"),
            })?
            .context(NotFoundSnafu {
                entity: format!("recipe {recipe_id:?}"),
            })?;

        let machine_id = recipe.machine();
        let machine = (self.machine_repository.get(machine_id).await)
            .context(InfrastructureSnafu {
                message: format!("could not get machine: {machine_id:?}"),
            })?
            .context(NotFoundSnafu {
                entity: format!("machine {machine_id:?}"),
            })?;

        let product_rate = recipe
            .get_product_rate(goal_id)
            .expect("recipe should produce the goal product");

        let flow_effective = replica_effective * product_rate;
        let flow_backward = replica_backward.map(|r| r * product_rate);

        let deps_plan_detail = self
            .convert_dependencies_to_detail(dependencies, &recipe, replica_effective, depth)
            .await?;

        Ok(PlanDetail {
            goal_id: goal_id.clone(),
            goal_name: item.name().clone(),
            recipe_id: recipe_id.clone(),
            machine_id: machine_id.clone(),
            machine_name: machine.name().clone(),
            machine_power: *machine.power(),
            flow_effective,
            flow_backward,
            replica_effective,
            replica_backward,
            cyclic_steps_ahead: None,
            depth,
            dependencies: deps_plan_detail,
        })
    }

    async fn convert_dependencies_to_detail(
        &self,
        dependencies: &[Plan],
        recipe: &Recipe,
        replica_effective: Replica,
        depth: u32,
    ) -> Result<Vec<PlanDetail>, QueryPlanError> {
        let material_flows = recipe.get_materials_flow(replica_effective);

        let mut deps = Vec::new();

        for (material_id, flow_material) in material_flows {
            let dep_plan = (dependencies.iter().find(|plan| plan.goal() == material_id))
                .expect("dependency should exist for material");

            let dep_detail =
                Box::pin(self.convert_plan_to_detail(dep_plan, &flow_material, depth + 1)).await?;
            deps.push(dep_detail);
        }

        Ok(deps)
    }

    async fn convert_cyclic_plan_to_detail(
        &self,
        step: &CyclicStep,
        flow_goal: &Flow,
        depth: u32,
    ) -> Result<PlanDetail, QueryPlanError> {
        let goal_id = step.goal();
        let recipe_id = step.recipe();
        let replica_effective = step.replica_effective();

        let item = (self.item_repository.get(goal_id).await)
            .context(InfrastructureSnafu {
                message: format!("could not get item: {goal_id:?}"),
            })?
            .context(NotFoundSnafu {
                entity: format!("item {goal_id:?}"),
            })?;

        let recipe = (self.recipe_repository.get(recipe_id).await)
            .context(InfrastructureSnafu {
                message: format!("could not get recipe: {recipe_id:?}"),
            })?
            .context(NotFoundSnafu {
                entity: format!("recipe {recipe_id:?}"),
            })?;

        let machine_id = recipe.machine();
        let machine = (self.machine_repository.get(machine_id).await)
            .context(InfrastructureSnafu {
                message: format!("could not get machine: {machine_id:?}"),
            })?
            .context(NotFoundSnafu {
                entity: format!("machine {machine_id:?}"),
            })?;

        Ok(PlanDetail {
            goal_id: goal_id.clone(),
            goal_name: item.name().clone(),
            recipe_id: recipe_id.clone(),
            machine_id: machine_id.clone(),
            machine_name: machine.name().clone(),
            machine_power: *machine.power(),
            flow_effective: *flow_goal,
            flow_backward: None,
            replica_effective,
            replica_backward: None,
            cyclic_steps_ahead: Some(step.steps_ahead()),
            depth,
            dependencies: vec![],
        })
    }
}
