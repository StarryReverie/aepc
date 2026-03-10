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

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use unimock::*;

    use crate::domain::item::model::Item;
    use crate::domain::item::outbound::{DynItemRepository, ItemRepositoryMock};
    use crate::domain::machine::model::Machine;
    use crate::domain::machine::outbound::{DynMachineRepository, MachineRepositoryMock};
    use crate::domain::plan::service::PlanFactory;
    use crate::domain::recipe::model::{Period, Quantity};
    use crate::domain::recipe::outbound::{DynRecipeRepository, RecipeRepositoryMock};

    use super::*;

    fn setup_cyclic_pipeline_mocks() -> PlanQueryServiceImpl {
        fn i1() -> Item {
            Item::new(ItemId::new("i1").unwrap(), ItemName::new("Item 1").unwrap())
        }
        fn i2() -> Item {
            Item::new(ItemId::new("i2").unwrap(), ItemName::new("Item 2").unwrap())
        }
        fn m1() -> Machine {
            Machine::new(
                MachineId::new("m1").unwrap(),
                MachineName::new("Machine 1").unwrap(),
                Power::new(100.0).unwrap(),
            )
        }
        fn m2() -> Machine {
            Machine::new(
                MachineId::new("m2").unwrap(),
                MachineName::new("Machine 2").unwrap(),
                Power::new(200.0).unwrap(),
            )
        }
        fn r1() -> Recipe {
            Recipe::new(
                RecipeId::new("r1").unwrap(),
                m1().id().clone(),
                Period::new(1.0).unwrap(),
                vec![(i2().id().clone(), Quantity::new(1.0).unwrap())],
                vec![(i1().id().clone(), Quantity::new(2.0).unwrap())],
            )
            .unwrap()
        }
        fn r2() -> Recipe {
            Recipe::new(
                RecipeId::new("r2").unwrap(),
                m2().id().clone(),
                Period::new(1.0).unwrap(),
                vec![(i1().id().clone(), Quantity::new(1.0).unwrap())],
                vec![(i2().id().clone(), Quantity::new(1.0).unwrap())],
            )
            .unwrap()
        }

        let item_repo =
            DynItemRepository::new_arc(Unimock::new(ItemRepositoryMock::get.stub(|each| {
                each.call(matching!((id) if *id == i1().id()))
                    .answers(&|_, _| Ok(Some(i1())));
                each.call(matching!((id) if *id == i2().id()))
                    .answers(&|_, _| Ok(Some(i2())));
            })));

        let machine_repo =
            DynMachineRepository::new_arc(Unimock::new(MachineRepositoryMock::get.stub(|each| {
                each.call(matching!((id) if *id == m1().id()))
                    .answers(&|_, _| Ok(Some(m1())));
                each.call(matching!((id) if *id == m2().id()))
                    .answers(&|_, _| Ok(Some(m2())));
            })));

        let recipe_repo = DynRecipeRepository::new_arc(Unimock::new((
            RecipeRepositoryMock::get.stub(|each| {
                each.call(matching!((id) if *id == r1().id()))
                    .answers(&|_, _| Ok(Some(r1())));
                each.call(matching!((id) if *id == r2().id()))
                    .answers(&|_, _| Ok(Some(r2())));
            }),
            RecipeRepositoryMock::find_containing_product.stub(|each| {
                each.call(matching!((id) if *id == i1().id()))
                    .answers(&|_, _| Ok(vec![r1()]));
                each.call(matching!((id) if *id == i2().id()))
                    .answers(&|_, _| Ok(vec![r2()]));
            }),
        )));

        let factory = Arc::new(PlanFactory::new(recipe_repo.clone()));
        PlanQueryServiceImpl::new(item_repo, machine_repo, recipe_repo, factory)
    }

    #[tokio::test]
    async fn test_query_plan_with_cyclic_pipeline() {
        let service = setup_cyclic_pipeline_mocks();

        let request = QueryPlanRequest {
            goal: ItemId::new("i1").unwrap(),
            expected_flow: Flow::new(60.0).unwrap(),
        };

        let response = service.query_plan_impl(request).await.unwrap();
        let detail = response.plan;

        assert_eq!(*detail.flow_effective(), Flow::new(60.0).unwrap());
        assert_eq!(*detail.flow_backward(), Some(Flow::new(60.0).unwrap()));
        assert_eq!(*detail.replica_effective(), Replica::new(0.5).unwrap());
        assert_eq!(*detail.replica_backward(), Some(Replica::new(0.5).unwrap()));
        assert_eq!(*detail.cyclic_steps_ahead(), None);
        assert_eq!(*detail.depth(), 0);
        assert_eq!(detail.dependencies().len(), 1);

        let i2_detail = &detail.dependencies()[0];
        assert_eq!(*i2_detail.flow_effective(), Flow::new(60.0).unwrap());
        assert_eq!(*i2_detail.flow_backward(), None);
        assert_eq!(*i2_detail.replica_effective(), Replica::new(1.0).unwrap());
        assert_eq!(*i2_detail.replica_backward(), None);
        assert_eq!(*i2_detail.cyclic_steps_ahead(), None);
        assert_eq!(*i2_detail.depth(), 1);
        assert_eq!(i2_detail.dependencies().len(), 1);

        let i1_detail = &i2_detail.dependencies()[0];
        assert_eq!(*i1_detail.flow_effective(), Flow::new(60.0).unwrap());
        assert_eq!(*i1_detail.flow_backward(), None);
        assert_eq!(*i1_detail.replica_effective(), Replica::new(0.5).unwrap());
        assert_eq!(*i1_detail.replica_backward(), None);
        assert_eq!(*i1_detail.cyclic_steps_ahead(), Some(2));
        assert_eq!(*i1_detail.depth(), 2);
        assert_eq!(i1_detail.dependencies().len(), 0);
    }
}
