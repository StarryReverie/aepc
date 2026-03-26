use ratatui::buffer::Buffer;
use ratatui::crossterm::event::Event;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::Span;
use ratatui::widgets::{Paragraph, Widget};

use crate::infrastructure::util::component::Component;
use crate::infrastructure::util::state::State;
use crate::ui::state::{AppState, StatusBarState, StatusLevel};
use crate::ui::style;

pub struct StatusBarComponent {
    status_bar_state: State<StatusBarState>,
    app_state: State<AppState>,
}

impl StatusBarComponent {
    pub fn new(status_bar_state: State<StatusBarState>, app_state: State<AppState>) -> Self {
        Self {
            status_bar_state,
            app_state,
        }
    }
}

impl Component for StatusBarComponent {
    fn handle_input(&self, _input: &Event) {}
}

impl Widget for &StatusBarComponent {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let status_bar_state = self.status_bar_state.get();
        let app_state = self.app_state.get();

        let color = match app_state.status_level() {
            StatusLevel::Info => Color::Rgb(100, 180, 100),
            StatusLevel::Error => Color::Rgb(200, 80, 80),
        };

        let text = Span::raw(status_bar_state.text()).style(Style::new().fg(color));

        let paragraph = Paragraph::new(text)
            .block(style::block_default().title(" Status "))
            .scroll((0, status_bar_state.scroll_offset() as u16));

        paragraph.render(area, buf);
    }
}
