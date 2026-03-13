use std::time::Duration;

use anyhow::Result as AnyhowResult;
use futures::StreamExt;
use ratatui::Terminal;
use ratatui::backend::Backend;
use ratatui::crossterm::event::{Event, EventStream};

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
            Some(Ok(Event::Key(key))) = event.next() => {
                app.handle_input(&key);
            }
            Ok(app_state) = app_state.watch() => {
                if app_state.running() == false {
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

    let plan_tab = PlanTabComponent::new(plan_tab_context.requester());
    let status_bar = StatusBarComponent::new(app_context.state());
    let app = AppComponent::new(plan_tab, status_bar, app_context.requester());

    let app_state = app_context.state();

    app_context.run();
    plan_tab_context.run();

    (app, app_state)
}
