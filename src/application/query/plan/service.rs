use std::sync::Arc;

use crate::domain::{
    item::outbound::DynItemRepository, machine::outbound::DynMachineRepository,
    plan::service::PlanFactory, recipe::outbound::DynRecipeRepository,
};

use super::{QueryPlanError, QueryPlanRequest, QueryPlanResponse};

#[dynosaur::dynosaur(pub DynPlanQueryService = dyn(box) PlanQueryService)]
pub trait PlanQueryService: Send + Sync {
    async fn query_plan(
        &self,
        request: QueryPlanRequest,
    ) -> Result<QueryPlanResponse, QueryPlanError>;
}

pub struct PlanQueryServiceImpl {
    pub(super) item_repository: Arc<DynItemRepository<'static>>,
    pub(super) machine_repository: Arc<DynMachineRepository<'static>>,
    pub(super) recipe_repository: Arc<DynRecipeRepository<'static>>,
    pub(super) plan_factory: Arc<PlanFactory>,
}

impl PlanQueryServiceImpl {
    pub fn new(
        item_repository: Arc<DynItemRepository<'static>>,
        machine_repository: Arc<DynMachineRepository<'static>>,
        recipe_repository: Arc<DynRecipeRepository<'static>>,
        plan_factory: Arc<PlanFactory>,
    ) -> Self {
        Self {
            item_repository,
            machine_repository,
            recipe_repository,
            plan_factory,
        }
    }
}

impl PlanQueryService for PlanQueryServiceImpl {
    async fn query_plan(
        &self,
        request: QueryPlanRequest,
    ) -> Result<QueryPlanResponse, QueryPlanError> {
        self.query_plan_impl(request).await
    }
}
