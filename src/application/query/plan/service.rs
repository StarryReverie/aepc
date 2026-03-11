use std::sync::Arc;

use crate::domain::item::outbound::DynItemRepository;
use crate::domain::machine::outbound::DynMachineRepository;
use crate::domain::plan::service::DynPlanFactory;
use crate::domain::recipe::outbound::DynRecipeRepository;

use super::{QueryPlanError, QueryPlanRequest, QueryPlanResponse};

#[unimock::unimock(api = PlanQueryServiceMock)]
#[dynosaur::dynosaur(pub DynPlanQueryService = dyn(box) PlanQueryService)]
pub trait PlanQueryService: Send + Sync {
    fn query_plan(
        &self,
        request: QueryPlanRequest,
    ) -> impl Future<Output = Result<QueryPlanResponse, QueryPlanError>> + Send;
}

pub struct PlanQueryServiceImpl {
    pub(super) item_repository: Arc<DynItemRepository<'static>>,
    pub(super) machine_repository: Arc<DynMachineRepository<'static>>,
    pub(super) recipe_repository: Arc<DynRecipeRepository<'static>>,
    pub(super) plan_factory: Arc<DynPlanFactory<'static>>,
}

impl PlanQueryServiceImpl {
    pub fn new(
        item_repository: Arc<DynItemRepository<'static>>,
        machine_repository: Arc<DynMachineRepository<'static>>,
        recipe_repository: Arc<DynRecipeRepository<'static>>,
        plan_factory: Arc<DynPlanFactory<'static>>,
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
