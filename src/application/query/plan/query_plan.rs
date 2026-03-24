#![expect(unused)]
use std::backtrace::Backtrace;

use anyhow::Error as AnyhowError;
use getset::Getters;
use snafu::prelude::*;

use crate::application::query::plan::PlanQueryServiceImpl;
use crate::domain::item::model::{ItemId, ItemName};
use crate::domain::item::outbound::ItemRepository;
use crate::domain::machine::model::{MachineId, MachineName, Power};
use crate::domain::machine::outbound::MachineRepository;
use crate::domain::plan::service::{CreatePlanError, PlanFactory};
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
        // let plan = (self.plan_factory)
        //     .create_plan(&request.goal, request.expected_flow)
        //     .await
        //     .context(PlanSnafu)?;

        // let plan_detail = self
        //     .convert_plan_to_detail(&plan, &request.expected_flow, 0)
        //     .await?;

        // Ok(QueryPlanResponse { plan: plan_detail })
        todo!("use new plan representation")
    }
}
