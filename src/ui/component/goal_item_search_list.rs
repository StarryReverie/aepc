use ratatui::widgets::{ListItem, Widget};
use tokio::sync::mpsc::Sender;

use crate::infrastructure::util::component::{Component, ListUtilComponent};
use crate::infrastructure::util::state::State;
use crate::ui::state::{self, GoalItemSearchListAction, GoalItemSearchListState, PlanTabState};
use crate::ui::style;

pub struct GoalItemSearchListComponent {
    list: ListUtilComponent,
}

impl GoalItemSearchListComponent {
    pub fn new(
        goal_item_search_list_requester: Sender<GoalItemSearchListAction>,
        goal_item_search_list_state: State<GoalItemSearchListState>,
        plan_tab_state: State<PlanTabState>,
    ) -> Self {
        let list = ListUtilComponent::new(
            state::create_goal_item_search_list_on_select(goal_item_search_list_requester),
            create_goal_item_search_list_items(goal_item_search_list_state),
            state::create_goal_item_search_list_is_focused(plan_tab_state),
            || " Expected Item Selection ".to_string(),
            style::block_with_focused,
            style::highlight_with_focused,
        );

        Self { list }
    }
}

impl Component for GoalItemSearchListComponent {
    fn handle_input(&self, input: &ratatui::crossterm::event::Event) {
        self.list.handle_input(input);
    }
}

impl Widget for &GoalItemSearchListComponent {
    fn render(self, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer)
    where
        Self: Sized,
    {
        self.list.render(area, buf);
    }
}

fn create_goal_item_search_list_items(
    goal_item_search_list_state: State<GoalItemSearchListState>,
) -> impl Fn() -> Vec<ListItem<'static>> + Send + Sync + 'static {
    move || {
        let state = goal_item_search_list_state.get();
        state
            .filtered_items()
            .iter()
            .map(|item| ListItem::new(item.name().to_string()))
            .collect::<Vec<ListItem>>()
    }
}
