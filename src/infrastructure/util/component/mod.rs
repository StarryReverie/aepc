mod text_input;

pub use text_input::TextInputUtilComponent;

use ratatui::crossterm::event::Event;
use ratatui::widgets::Widget;

pub trait Component
where
    for<'a> &'a Self: Widget,
{
    fn handle_input(&self, input: &Event);
}
