use getset::Getters;
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::{Event, KeyCode};
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph, Widget};
use tokio::sync::mpsc::{self, Receiver, Sender};

use crate::infrastructure::util::state::{State, StateSource};

use super::Component;

pub struct TextInputUtilComponent {
    requester: Sender<TextInputAction>,
    state: State<TextInputState>,
    is_focused: Box<dyn Fn() -> bool + Send + Sync>,
    title: Box<dyn Fn() -> String + Send + Sync>,
}

impl TextInputUtilComponent {
    pub fn new<FChar, FConfirm, FFocused, FTitle>(
        allowed_char: FChar,
        on_confirm: FConfirm,
        is_focused: FFocused,
        title: FTitle,
    ) -> Self
    where
        FChar: Fn(char) -> bool + Send + Sync + 'static,
        FConfirm: FnMut(&str) + Send + Sync + 'static,
        FFocused: Fn() -> bool + Send + Sync + 'static,
        FTitle: Fn() -> String + Send + Sync + 'static,
    {
        let (requester, state) = Self::create_and_run_state_manager(allowed_char, on_confirm);

        Self {
            requester,
            state,
            is_focused: Box::new(is_focused),
            title: Box::new(title),
        }
    }

    fn create_and_run_state_manager(
        allowed_char: impl Fn(char) -> bool + Send + Sync + 'static,
        on_confirm: impl FnMut(&str) + Send + Sync + 'static,
    ) -> (Sender<TextInputAction>, State<TextInputState>) {
        let (requester, actions) = mpsc::channel(32);
        let (source, state) = StateSource::new(TextInputState::default());
        let manager = TextInputStateManager {
            source,
            actions,
            allowed_char: Box::new(allowed_char),
            on_confirm: Box::new(on_confirm),
        };

        tokio::spawn(async move {
            let mut manager = manager;
            loop {
                tokio::select! {
                    Some(action) = manager.actions.recv() => {
                        manager.handle_action(action);
                    }
                }
            }
        });

        (requester, state)
    }
}

impl Component for TextInputUtilComponent {
    fn handle_input(&self, input: &Event) {
        if let Event::Key(key) = input {
            match key.code {
                KeyCode::Char(c) => {
                    let _ = self.requester.try_send(TextInputAction::Input(c));
                }
                KeyCode::Backspace => {
                    let _ = self.requester.try_send(TextInputAction::Backspace);
                }
                KeyCode::Enter => {
                    let _ = self.requester.try_send(TextInputAction::Confirm);
                }
                KeyCode::Esc => {
                    let _ = self.requester.try_send(TextInputAction::Clear);
                }
                _ => {}
            }
        }
    }
}

impl Widget for &TextInputUtilComponent {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let text_input_state = self.state.get();
        let is_focused = (self.is_focused)();

        let line = Line::from(vec![
            Span::from(text_input_state.input_text()),
            Span::raw(if is_focused { "█" } else { "" }),
        ]);

        let border_style = if is_focused {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };

        let paragraph = Paragraph::new(line).block(
            Block::new()
                .border_type(BorderType::Plain)
                .borders(Borders::all())
                .border_style(border_style)
                .title((self.title)()),
        );

        paragraph.render(area, buf);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Getters)]
struct TextInputState {
    #[getset(get = "pub")]
    input_text: String,
}

impl Default for TextInputState {
    fn default() -> Self {
        Self {
            input_text: String::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum TextInputAction {
    Input(char),
    Backspace,
    Confirm,
    Clear,
}

struct TextInputStateManager {
    source: StateSource<TextInputState>,
    actions: Receiver<TextInputAction>,
    allowed_char: Box<dyn Fn(char) -> bool + Send + Sync>,
    on_confirm: Box<dyn FnMut(&str) + Send + Sync>,
}

impl TextInputStateManager {
    fn handle_action(&mut self, action: TextInputAction) {
        match action {
            TextInputAction::Input(c) => self.handle_action_input(c),
            TextInputAction::Backspace => self.handle_action_backspace(),
            TextInputAction::Confirm => self.handle_action_confirm(),
            TextInputAction::Clear => self.handle_action_clear(),
        }
    }

    fn handle_action_input(&mut self, c: char) {
        if (self.allowed_char)(c) {
            self.source.modify(|state| {
                let mut text = state.input_text.clone();
                text.push(c);
                TextInputState {
                    input_text: text,
                    ..state.clone()
                }
            });
        }
    }

    fn handle_action_backspace(&mut self) {
        self.source.modify(|state| {
            let mut text = state.input_text.clone();
            text.pop();
            TextInputState {
                input_text: text,
                ..state.clone()
            }
        });
    }

    fn handle_action_confirm(&mut self) {
        let input_text = self.source.get().input_text().clone();
        (self.on_confirm)(&input_text);
    }

    fn handle_action_clear(&mut self) {
        self.source.modify(|state| TextInputState {
            input_text: String::new(),
            ..state.clone()
        });
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use tokio::time::Duration;

    use super::*;

    #[tokio::test]
    async fn test_basic_input() {
        let confirmed = Arc::new(Mutex::new(Vec::new()));
        let confirmed_clone = confirmed.clone();

        let component = TextInputUtilComponent::new(
            |_| true,
            move |text| {
                confirmed_clone.lock().unwrap().push(text.to_string());
            },
            || true,
            || "Test".to_string(),
        );

        let _ = component.requester.try_send(TextInputAction::Input('a'));
        let _ = component.requester.try_send(TextInputAction::Input('b'));

        tokio::time::sleep(Duration::from_millis(50)).await;

        assert_eq!(component.state.get().input_text(), "ab");
    }

    #[tokio::test]
    async fn test_backspace() {
        let component =
            TextInputUtilComponent::new(|_| true, |_| {}, || true, || "Test".to_string());

        let _ = component.requester.try_send(TextInputAction::Input('a'));
        let _ = component.requester.try_send(TextInputAction::Input('b'));
        let _ = component.requester.try_send(TextInputAction::Backspace);

        tokio::time::sleep(Duration::from_millis(50)).await;

        assert_eq!(component.state.get().input_text(), "a");
    }

    #[tokio::test]
    async fn test_clear() {
        let component = TextInputUtilComponent::new(
            |_| true,
            |_| {},
            Box::new(|| true),
            Box::new(|| "Test".to_string()),
        );

        let _ = component.requester.try_send(TextInputAction::Input('a'));
        let _ = component.requester.try_send(TextInputAction::Clear);

        tokio::time::sleep(Duration::from_millis(50)).await;

        assert_eq!(component.state.get().input_text(), "");
    }

    #[tokio::test]
    async fn test_char_filter() {
        let component = TextInputUtilComponent::new(
            |c| c.is_ascii_digit(),
            |_| {},
            || true,
            || "Test".to_string(),
        );

        let _ = component.requester.try_send(TextInputAction::Input('1'));
        let _ = component.requester.try_send(TextInputAction::Input('a'));
        let _ = component.requester.try_send(TextInputAction::Input('2'));

        tokio::time::sleep(Duration::from_millis(50)).await;

        assert_eq!(component.state.get().input_text(), "12");
    }

    #[tokio::test]
    async fn test_on_confirm_callback_invoked() {
        let confirmed = Arc::new(Mutex::new(Vec::new()));
        let confirmed_clone = confirmed.clone();

        let component = TextInputUtilComponent::new(
            |_| true,
            move |text| {
                confirmed_clone.lock().unwrap().push(text.to_string());
            },
            || true,
            || "Test".to_string(),
        );

        let _ = component.requester.try_send(TextInputAction::Input('x'));
        let _ = component.requester.try_send(TextInputAction::Confirm);

        tokio::time::sleep(Duration::from_millis(50)).await;

        assert_eq!(*confirmed.lock().unwrap(), vec!["x"]);
    }
}
