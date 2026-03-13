use std::time::Duration;

use anyhow::Result as AnyhowResult;
use futures::StreamExt;
use ratatui::Terminal;
use ratatui::backend::Backend;
use ratatui::crossterm::event::{Event, EventStream};

use aepc::infrastructure::util::state::State;
use aepc::ui::component::*;
use aepc::ui::state::{AppState, AppStateManager};

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
    let (app_manager, app_requester) = AppStateManager::new();

    let plan_tab = PlanTabComponent::new();
    let app = AppComponent::new(plan_tab, app_requester);

    let app_state = app_manager.state();

    app_manager.run();

    (app, app_state)
}
