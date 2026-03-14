use anyhow::Result as AnyhowResult;
use futures::StreamExt;
use ratatui::Terminal;
use ratatui::backend::Backend;
use ratatui::crossterm::event::EventStream;
use tokio::time::Duration;

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
    let app_context = AppStateManager::context();
    let plan_tab_context = PlanTabStateManager::context();

    let app = AppComponent::new(
        PlanTabComponent::new(
            GoalSelectionPanelComponent::new(
                GoalFlowInputComponent::new(
                    plan_tab_context.requester(),
                    app_context.requester(),
                    plan_tab_context.state(),
                ),
                plan_tab_context.state(),
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

    (app, app_state)
}
