use anyhow::Result as AnyhowResult;
use futures::StreamExt;
use ratatui::Terminal;
use ratatui::backend::Backend;
use ratatui::crossterm::event::EventStream;
use tokio::time::Duration;

use aepc::application::query::item::{DynItemQueryService, ItemQueryServiceImpl};
use aepc::application::query::plan::{DynPlanQueryService, PlanQueryServiceImpl};
use aepc::domain::item::outbound::DynItemRepository;
use aepc::domain::machine::outbound::DynMachineRepository;
use aepc::domain::plan::service::{DynPlanFactory, PlanFactoryImpl};
use aepc::domain::recipe::outbound::DynRecipeRepository;
use aepc::infrastructure::data::*;
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
            _ = tokio::time::sleep(Duration::from_millis(100)) => {}
        }
        terminal.draw(|frame| {
            frame.render_widget(&app, frame.area());
        })?;
    }
}

fn setup() -> (AppComponent, State<AppState>) {
    let item_repo = DynItemRepository::new_arc(ItemConstantRepository::new());
    let machine_repo = DynMachineRepository::new_arc(MachineConstantRepository::new());
    let recipe_repo = DynRecipeRepository::new_arc(RecipeConstantRepository::new());

    let plan_factory = DynPlanFactory::new_arc(PlanFactoryImpl::new(recipe_repo.clone()));

    let item_query_service =
        DynItemQueryService::new_arc(ItemQueryServiceImpl::new(item_repo.clone()));
    let plan_query_service = DynPlanQueryService::new_arc(PlanQueryServiceImpl::new(
        item_repo,
        machine_repo,
        recipe_repo,
        plan_factory,
    ));

    let app_context = AppStateManager::context();
    let status_bar_context = StatusBarStateManager::context(app_context.state());
    let plan_tab_context = PlanTabStateManager::context(app_context.requester());
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
        StatusBarComponent::new(status_bar_context.state(), app_context.state()),
        KeybindingBarComponent::new(app_context.state()),
        app_context.requester(),
    );

    let app_state = app_context.state();

    app_context.run();
    status_bar_context.run();
    plan_tab_context.run();
    goal_item_search_context.run();
    goal_item_search_list_context.run();
    plan_tree_list_context.run();

    (app, app_state)
}
