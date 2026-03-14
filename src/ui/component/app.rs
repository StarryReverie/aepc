use ratatui::buffer::Buffer;
use ratatui::crossterm::event::{Event, KeyCode};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::widgets::Widget;
use tokio::sync::mpsc::Sender;

use crate::infrastructure::util::component::Component;
use crate::ui::component::{PlanTabComponent, StatusBarComponent};
use crate::ui::state::AppAction;

pub struct AppComponent {
    plan_tab: PlanTabComponent,
    status_bar: StatusBarComponent,
    app_requester: Sender<AppAction>,
}

impl AppComponent {
    pub fn new(
        plan_tab: PlanTabComponent,
        status_bar: StatusBarComponent,
        app_requester: Sender<AppAction>,
    ) -> Self {
        Self {
            plan_tab,
            status_bar,
            app_requester,
        }
    }
}

impl Component for AppComponent {
    fn handle_input(&self, input: &Event) {
        match input {
            Event::Key(key) if key.code == KeyCode::Char('q') => {
                let _ = self.app_requester.try_send(AppAction::Quit);
            }
            _ => {
                self.plan_tab.handle_input(input);
            }
        }
    }
}

impl Widget for &AppComponent {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let layout = Layout::new(
            Direction::Vertical,
            [Constraint::Min(0), Constraint::Length(3)],
        )
        .split(area);

        self.plan_tab.render(layout[0], buf);
        self.status_bar.render(layout[1], buf);
    }
}
