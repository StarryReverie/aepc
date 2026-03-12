mod app;

pub use app::AppComponent;

use ratatui::crossterm::event::KeyEvent;
use ratatui::widgets::Widget;

pub trait Component
where
    for<'a> &'a Self: Widget,
{
    fn handle_input(&self, key: &KeyEvent);
}
