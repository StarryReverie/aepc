use ratatui::buffer::Buffer;
use ratatui::crossterm::event::{Event, KeyCode};
use ratatui::layout::Rect;
use ratatui::widgets::{List, ListItem, StatefulWidget, Widget};
use tokio::sync::mpsc::Sender;

use crate::infrastructure::util::component::Component;
use crate::infrastructure::util::state::State;
use crate::ui::state::{
    GoalItemSearchListAction, GoalItemSearchListState, PlanTabFocus, PlanTabState,
};
use crate::ui::style;

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
        let plan_tab_state = self.plan_tab_state.get();
        let is_focused = plan_tab_state.focus() == PlanTabFocus::GoalItemSearchList;

        let goal_item_search_list_state = self.goal_item_search_list_state.get();
        let items = goal_item_search_list_state.filtered_items();
        let mut list_state = goal_item_search_list_state.list_state();

        let list_items: Vec<ListItem> = items
            .iter()
            .map(|item| ListItem::new(item.name().to_string()))
            .collect();

        let list = List::new(list_items)
            .block(style::block_with_focused(is_focused).title(" Expected Item Selection "))
            .highlight_style(style::highlight_with_focused(is_focused));

        StatefulWidget::render(list, area, buf, &mut list_state);

        let _ = self
            .goal_item_search_list_requester
            .try_send(GoalItemSearchListAction::SetListState(list_state));
    }
}
