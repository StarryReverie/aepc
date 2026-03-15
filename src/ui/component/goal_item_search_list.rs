use ratatui::buffer::Buffer;
use ratatui::crossterm::event::{Event, KeyCode};
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, BorderType, Borders, List, ListItem, Widget};
use tokio::sync::mpsc::Sender;

use crate::infrastructure::util::component::Component;
use crate::infrastructure::util::state::State;
use crate::ui::state::{
    GoalItemSearchListAction, GoalItemSearchListState, PlanTabFocus, PlanTabState,
};

pub struct GoalItemSearchListComponent {
    goal_item_search_list_requester: Sender<GoalItemSearchListAction>,
    goal_item_search_list_state: State<GoalItemSearchListState>,
    plan_tab_state: State<PlanTabState>,
}

impl GoalItemSearchListComponent {
    pub fn new(
        goal_item_search_list_requester: Sender<GoalItemSearchListAction>,
        goal_item_search_list_state: State<GoalItemSearchListState>,
        plan_tab_state: State<PlanTabState>,
    ) -> Self {
        Self {
            goal_item_search_list_requester,
            goal_item_search_list_state,
            plan_tab_state,
        }
    }
}

impl Component for GoalItemSearchListComponent {
    fn handle_input(&self, input: &Event) {
        if let Event::Key(key) = input {
            match key.code {
                KeyCode::Down | KeyCode::Char('j') => {
                    let _ = self
                        .goal_item_search_list_requester
                        .try_send(GoalItemSearchListAction::SelectNext);
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    let _ = self
                        .goal_item_search_list_requester
                        .try_send(GoalItemSearchListAction::SelectPrevious);
                }
                KeyCode::Enter => {
                    let _ = self
                        .goal_item_search_list_requester
                        .try_send(GoalItemSearchListAction::ConfirmSelection);
                }
                _ => {}
            }
        }
    }
}

impl Widget for &GoalItemSearchListComponent {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let focus = self.plan_tab_state.get().focus();
        let is_focused = focus == PlanTabFocus::GoalItemSearchList;

        let list_state = self.goal_item_search_list_state.get();
        let items = list_state.filtered_items();
        let selected_index = list_state.selected_index();

        let list_items: Vec<ListItem> = items
            .iter()
            .enumerate()
            .map(|(idx, item)| {
                let is_selected = Some(idx) == selected_index;
                let item_text = item.name().value().as_str();
                if is_focused && is_selected {
                    ListItem::new(item_text).style(
                        Style::default()
                            .bg(Color::Cyan)
                            .fg(Color::Black)
                            .add_modifier(Modifier::BOLD),
                    )
                } else {
                    ListItem::new(item_text)
                }
            })
            .collect();

        let border_style = if is_focused {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray)
        };

        let list = List::new(list_items).block(
            Block::new()
                .border_type(BorderType::Plain)
                .borders(Borders::all())
                .border_style(border_style)
                .title(" Expected Item Selection "),
        );

        list.render(area, buf);
    }
}
