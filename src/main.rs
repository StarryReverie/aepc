use anyhow::Result as AnyhowResult;
use futures::StreamExt;
use ratatui::Terminal;
use ratatui::backend::Backend;
use ratatui::crossterm::event::EventStream;
use tokio::time::Duration;

use aepc::application::query::item::{DynItemQueryService, ItemQueryServiceImpl};
use aepc::application::query::plan::{DynPlanQueryService, PlanQueryServiceImpl};
use aepc::domain::item::model::{Item, ItemId, ItemName};
use aepc::domain::item::outbound::{DynItemRepository, ItemRepository};
use aepc::domain::machine::model::{Machine, MachineId, MachineName, Power};
use aepc::domain::machine::outbound::{DynMachineRepository, MachineRepository};
use aepc::domain::plan::service::{DynPlanFactory, PlanFactoryImpl};
use aepc::domain::recipe::model::{Period, Quantity, Recipe, RecipeId};
use aepc::domain::recipe::outbound::{DynRecipeRepository, RecipeRepository};
use aepc::infrastructure::util::component::Component;
use aepc::infrastructure::util::state::State;
use aepc::ui::component::*;
use aepc::ui::state::*;

#[tokio::main(flavor = "current_thread")]
async fn main() -> AnyhowResult<()> {
    let mut terminal = ratatui::init();
    let res = run(&mut terminal).await;
    ratatui::restore();
    res
}

async fn run<B>(terminal: &mut Terminal<B>) -> AnyhowResult<()>
where
    B: Backend,
    <B as Backend>::Error: Send + Sync + 'static,
{
    let (app, mut app_state) = setup();

    let mut event = EventStream::new();
    loop {
        tokio::select! {
            Some(Ok(input)) = event.next() => {
                app.handle_input(&input);
            }
            Ok(app_state) = app_state.watch() => {
                if !app_state.running() {
                    break Ok(());
                }
            }
            _ = tokio::time::sleep(Duration::from_millis(16)) => {
                terminal.draw(|frame| {
                    frame.render_widget(&app, frame.area());
                })?;
            }
        }
    }
}

fn setup() -> (AppComponent, State<AppState>) {
    let item_repo = DynItemRepository::new_arc(DemoItemRepository::new());
    let machine_repo = DynMachineRepository::new_arc(DemoMachineRepository::new());
    let recipe_repo = DynRecipeRepository::new_arc(DemoRecipeRepository::new());

    let item_query_service =
        DynItemQueryService::new_arc(ItemQueryServiceImpl::new(item_repo.clone()));
    let plan_factory = DynPlanFactory::new_arc(PlanFactoryImpl::new(recipe_repo.clone()));
    let plan_query_service = DynPlanQueryService::new_arc(PlanQueryServiceImpl::new(
        item_repo,
        machine_repo,
        recipe_repo,
        plan_factory,
    ));

    let app_context = AppStateManager::context();
    let plan_tab_context = PlanTabStateManager::context();
    let goal_item_search_context = GoalItemSearchStateManager::context();
    let goal_item_search_list_context = GoalItemSearchListStateManager::context(
        goal_item_search_context.state(),
        plan_tab_context.requester(),
        app_context.requester(),
        item_query_service,
    );
    let plan_tree_list_context = PlanTreeListStateManager::context(
        plan_tab_context.state(),
        app_context.requester(),
        plan_query_service,
    );

    let app = AppComponent::new(
        PlanTabComponent::new(
            GoalSelectionPanelComponent::new(
                GoalFlowInputComponent::new(
                    plan_tab_context.requester(),
                    app_context.requester(),
                    plan_tab_context.state(),
                ),
                GoalItemSearchComponent::new(
                    GoalItemSearchInputComponent::new(
                        goal_item_search_context.requester(),
                        plan_tab_context.requester(),
                        plan_tab_context.state(),
                    ),
                    GoalItemSearchListComponent::new(
                        goal_item_search_list_context.requester(),
                        goal_item_search_list_context.state(),
                        plan_tab_context.state(),
                    ),
                    plan_tab_context.state(),
                ),
                plan_tab_context.state(),
            ),
            PlanDisplayPanelComponent::new(
                PlanGoalLabelComponent::new(plan_tab_context.state()),
                PlanTreeListComponent::new(
                    plan_tree_list_context.requester(),
                    plan_tree_list_context.state(),
                    plan_tab_context.state(),
                ),
            ),
            plan_tab_context.requester(),
            plan_tab_context.state(),
        ),
        StatusBarComponent::new(app_context.state()),
        app_context.requester(),
    );

    let app_state = app_context.state();

    app_context.run();
    plan_tab_context.run();
    goal_item_search_context.run();
    goal_item_search_list_context.run();
    plan_tree_list_context.run();

    (app, app_state)
}

struct DemoItemRepository {
    items: Vec<Item>,
}

impl DemoItemRepository {
    fn new() -> Self {
        Self {
            items: vec![
                Item::new(
                    ItemId::new("iron_ore").unwrap(),
                    ItemName::new("Iron Ore").unwrap(),
                ),
                Item::new(
                    ItemId::new("iron_plate").unwrap(),
                    ItemName::new("Iron Plate").unwrap(),
                ),
                Item::new(
                    ItemId::new("iron_ingot").unwrap(),
                    ItemName::new("Iron Ingot").unwrap(),
                ),
                Item::new(
                    ItemId::new("copper_ore").unwrap(),
                    ItemName::new("Copper Ore").unwrap(),
                ),
                Item::new(
                    ItemId::new("copper_ingot").unwrap(),
                    ItemName::new("Copper Ingot").unwrap(),
                ),
            ],
        }
    }
}

impl ItemRepository for DemoItemRepository {
    async fn get(&self, item_id: &ItemId) -> AnyhowResult<Option<Item>> {
        Ok(self.items.iter().find(|item| item.id() == item_id).cloned())
    }

    async fn find_all_by_name_containing_pattern(&self, pattern: &str) -> AnyhowResult<Vec<Item>> {
        let pattern_lower = pattern.to_lowercase();
        Ok(self
            .items
            .iter()
            .filter(|item| item.name().value().to_lowercase().contains(&pattern_lower))
            .cloned()
            .collect())
    }
}

struct DemoMachineRepository {
    machines: Vec<Machine>,
}

impl DemoMachineRepository {
    fn new() -> Self {
        Self {
            machines: vec![
                Machine::new(
                    MachineId::new("mine").unwrap(),
                    MachineName::new("Mining Machine").unwrap(),
                    Power::new(50.0).unwrap(),
                ),
                Machine::new(
                    MachineId::new("smelter").unwrap(),
                    MachineName::new("Smelter").unwrap(),
                    Power::new(200.0).unwrap(),
                ),
                Machine::new(
                    MachineId::new("plate_press").unwrap(),
                    MachineName::new("Plate Press").unwrap(),
                    Power::new(100.0).unwrap(),
                ),
                Machine::new(
                    MachineId::new("copper_smelter").unwrap(),
                    MachineName::new("Copper Smelter").unwrap(),
                    Power::new(150.0).unwrap(),
                ),
            ],
        }
    }
}

impl MachineRepository for DemoMachineRepository {
    async fn get(&self, machine_id: &MachineId) -> AnyhowResult<Option<Machine>> {
        Ok(self.machines.iter().find(|m| m.id() == machine_id).cloned())
    }
}

struct DemoRecipeRepository {
    recipes: Vec<Recipe>,
}

impl DemoRecipeRepository {
    fn new() -> Self {
        // Materials: (item_id, quantity)
        // Products: (item_id, quantity)
        let iron_ore = ItemId::new("iron_ore").unwrap();
        let iron_plate = ItemId::new("iron_plate").unwrap();
        let iron_ingot = ItemId::new("iron_ingot").unwrap();
        let copper_ore = ItemId::new("copper_ore").unwrap();
        let copper_ingot = ItemId::new("copper_ingot").unwrap();
        let mine = MachineId::new("mine").unwrap();
        let smelter = MachineId::new("smelter").unwrap();
        let plate_press = MachineId::new("plate_press").unwrap();
        let copper_smelter = MachineId::new("copper_smelter").unwrap();

        Self {
            recipes: vec![
                // Mine: 1 iron ore per minute
                Recipe::new(
                    RecipeId::new("r_mine_iron").unwrap(),
                    mine.clone(),
                    Period::new(60.0).unwrap(),
                    vec![],
                    vec![(iron_ore.clone(), Quantity::new(60.0).unwrap())],
                )
                .unwrap(),
                // Smelter: 1 iron ingot from 2 iron ores (60s per operation)
                Recipe::new(
                    RecipeId::new("r_smelt_iron").unwrap(),
                    smelter.clone(),
                    Period::new(60.0).unwrap(),
                    vec![(iron_ore.clone(), Quantity::new(2.0).unwrap())],
                    vec![(iron_ingot.clone(), Quantity::new(1.0).unwrap())],
                )
                .unwrap(),
                // Plate Press: 1 iron plate from 1 iron ingot (30s per operation)
                Recipe::new(
                    RecipeId::new("r_press_plate").unwrap(),
                    plate_press.clone(),
                    Period::new(30.0).unwrap(),
                    vec![(iron_ingot.clone(), Quantity::new(1.0).unwrap())],
                    vec![(iron_plate.clone(), Quantity::new(1.0).unwrap())],
                )
                .unwrap(),
                // Copper Smelter: 1 copper ingot from 2 copper ores
                Recipe::new(
                    RecipeId::new("r_smelt_copper").unwrap(),
                    copper_smelter.clone(),
                    Period::new(60.0).unwrap(),
                    vec![(copper_ore.clone(), Quantity::new(2.0).unwrap())],
                    vec![(copper_ingot.clone(), Quantity::new(1.0).unwrap())],
                )
                .unwrap(),
            ],
        }
    }
}

impl RecipeRepository for DemoRecipeRepository {
    async fn get(&self, recipe_id: &RecipeId) -> AnyhowResult<Option<Recipe>> {
        Ok(self.recipes.iter().find(|r| r.id() == recipe_id).cloned())
    }

    async fn find_all_by_products_containing_target(
        &self,
        target_id: &ItemId,
    ) -> AnyhowResult<Vec<Recipe>> {
        Ok(self
            .recipes
            .iter()
            .filter(|recipe| recipe.products().iter().any(|(id, _)| id == target_id))
            .cloned()
            .collect())
    }
}
