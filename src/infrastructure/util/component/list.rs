use std::sync::Mutex;

use ratatui::buffer::Buffer;
use ratatui::crossterm::event::{Event, KeyCode};
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::widgets::{Block, List, ListItem, ListState, StatefulWidget, Widget};

use super::Component;

pub struct ListUtilComponent {
    list_state: Mutex<ListState>,
    on_select: Mutex<Box<dyn FnMut(usize) + Send + Sync>>,
    items: Box<dyn Fn() -> Vec<ListItem<'static>> + Send + Sync>,
    is_focused: Box<dyn Fn() -> bool + Send + Sync>,
    title: Box<dyn Fn() -> String + Send + Sync>,
    block_style: Box<dyn Fn(bool) -> Block<'static> + Send + Sync + 'static>,
    highlight_style: Box<dyn Fn(bool) -> Style + Send + Sync + 'static>,
}

impl ListUtilComponent {
    pub fn new<FSelect, FItems, FFocused, FTitle, FBlock, FHighlight>(
        on_select: FSelect,
        items: FItems,
        is_focused: FFocused,
        title: FTitle,
        block_style: FBlock,
        highlight_style: FHighlight,
    ) -> Self
    where
        FSelect: FnMut(usize) + Send + Sync + 'static,
        FItems: Fn() -> Vec<ListItem<'static>> + Send + Sync + 'static,
        FFocused: Fn() -> bool + Send + Sync + 'static,
        FTitle: Fn() -> String + Send + Sync + 'static,
        FBlock: Fn(bool) -> Block<'static> + Send + Sync + 'static,
        FHighlight: Fn(bool) -> Style + Send + Sync + 'static,
    {
        Self {
            list_state: Mutex::new(ListState::default().with_selected(Some(0))),
            on_select: Mutex::new(Box::new(on_select)),
            items: Box::new(items),
            is_focused: Box::new(is_focused),
            title: Box::new(title),
            block_style: Box::new(block_style),
            highlight_style: Box::new(highlight_style),
        }
    }
}

impl Component for ListUtilComponent {
    fn handle_input(&self, input: &Event) {
        if let Event::Key(key) = input {
            match key.code {
                KeyCode::Down | KeyCode::Char('j') => {
                    self.list_state.lock().unwrap().select_next();
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    self.list_state.lock().unwrap().select_previous();
                }
                KeyCode::Enter => {
                    let list_state = self.list_state.lock().unwrap();
                    let selected = list_state.selected().unwrap_or(0);
                    (self.on_select.lock().unwrap())(selected);
                }
                _ => {}
            }
        }
    }
}

impl Widget for &ListUtilComponent {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let is_focused = (self.is_focused)();
        let list_items = (self.items)();
        let mut list_state = self.list_state.lock().unwrap();

        let list = List::new(list_items)
            .block((self.block_style)(is_focused).title((self.title)()))
            .highlight_style((self.highlight_style)(is_focused));

        StatefulWidget::render(list, area, buf, &mut list_state);
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    use super::*;

    #[test]
    fn test_navigate() {
        let component = ListUtilComponent::new(
            |_| {},
            || vec![ListItem::new("a"), ListItem::new("b"), ListItem::new("c")],
            || true,
            || "Test".to_string(),
            |_| Block::new(),
            |_| Style::default(),
        );

        component.handle_input(&Event::Key(KeyCode::Down.into()));
        assert_eq!(component.list_state.lock().unwrap().selected(), Some(1));

        component.handle_input(&Event::Key(KeyCode::Char('j').into()));
        assert_eq!(component.list_state.lock().unwrap().selected(), Some(2));

        component.handle_input(&Event::Key(KeyCode::Up.into()));
        assert_eq!(component.list_state.lock().unwrap().selected(), Some(1));

        component.handle_input(&Event::Key(KeyCode::Char('k').into()));
        assert_eq!(component.list_state.lock().unwrap().selected(), Some(0));
    }

    #[test]
    fn test_select_callback_called() {
        let selected = Arc::new(Mutex::new(None));
        let selected_clone = selected.clone();

        let component = ListUtilComponent::new(
            move |index| {
                *selected_clone.lock().unwrap() = Some(index);
            },
            || vec![ListItem::new("a"), ListItem::new("b")],
            || true,
            || "Test".to_string(),
            |_| Block::new(),
            |_| Style::default(),
        );

        component.handle_input(&Event::Key(KeyCode::Enter.into()));
        assert_eq!(*selected.lock().unwrap(), Some(0));

        component.handle_input(&Event::Key(KeyCode::Down.into()));
        component.handle_input(&Event::Key(KeyCode::Enter.into()));
        assert_eq!(*selected.lock().unwrap(), Some(1));
    }

    #[test]
    fn test_render_empty_list_does_not_panic() {
        let component = ListUtilComponent::new(
            |_| {},
            || vec![],
            || true,
            || "Test".to_string(),
            |_| Block::new(),
            |_| Style::default(),
        );

        let backend = TestBackend::new(20, 3);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|frame| {
                frame.render_widget(&component, frame.area());
            })
            .unwrap();
    }
}
