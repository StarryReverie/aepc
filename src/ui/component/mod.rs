mod app;
mod plan_tab;

pub use app::AppComponent;
pub use plan_tab::PlanTabComponent;

use ratatui::crossterm::event::KeyEvent;
use ratatui::widgets::Widget;

pub trait Component
where
    for<'a> &'a Self: Widget,
{
    fn handle_input(&self, key: &KeyEvent);
}
