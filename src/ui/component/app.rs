use tokio::sync::mpsc::Sender;

use ratatui::buffer::Buffer;
use ratatui::crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::Rect;
use ratatui::widgets::Widget;

use crate::ui::state::AppAction;

use super::Component;

pub struct AppComponent {
    app_requester: Sender<AppAction>,
}

impl AppComponent {
    pub fn new(app_requester: Sender<AppAction>) -> Self {
        Self { app_requester }
    }
}

impl Component for AppComponent {
    fn handle_input(&self, key: &KeyEvent) {
        match &key.code {
            KeyCode::Char('q') => {
                let _ = self.app_requester.try_send(AppAction::Quit);
            }
            _ => {}
        }
    }
}

impl Widget for &AppComponent {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        "Hello World".render(area, buf);
    }
}
