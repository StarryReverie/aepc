mod app;
mod goal_selection_panel;
mod plan_tab;
mod status_bar;

pub use app::AppComponent;
pub use goal_selection_panel::GoalSelectionPanelComponent;
pub use plan_tab::PlanTabComponent;
pub use status_bar::StatusBarComponent;

use ratatui::crossterm::event::KeyEvent;
use ratatui::widgets::Widget;

pub trait Component
where
    for<'a> &'a Self: Widget,
{
    fn handle_input(&self, key: &KeyEvent);
}
