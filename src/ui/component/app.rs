use ratatui::buffer::Buffer;
use ratatui::crossterm::event::{Event, KeyCode};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::widgets::Widget;
use tokio::sync::mpsc::Sender;

use crate::infrastructure::util::component::Component;
use crate::ui::component::{KeybindingBarComponent, PlanTabComponent, StatusBarComponent};
use crate::ui::state::AppAction;

pub struct AppComponent {
    plan_tab: PlanTabComponent,
    status_bar: StatusBarComponent,
    keybinding_bar: KeybindingBarComponent,
    app_requester: Sender<AppAction>,
}

impl AppComponent {
    pub fn new(
        plan_tab: PlanTabComponent,
        status_bar: StatusBarComponent,
        keybinding_bar: KeybindingBarComponent,
        app_requester: Sender<AppAction>,
    ) -> Self {
        Self {
            plan_tab,
            status_bar,
            keybinding_bar,
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
        let vertical = Layout::new(
            Direction::Vertical,
            [Constraint::Min(0), Constraint::Length(3)],
        )
        .split(area);

        let horizontal = Layout::new(
            Direction::Horizontal,
            [Constraint::Fill(2), Constraint::Fill(5)],
        )
        .split(vertical[1]);

        self.plan_tab.render(vertical[0], buf);
        self.status_bar.render(horizontal[0], buf);
        self.keybinding_bar.render(horizontal[1], buf);
    }
}
