use ratatui::buffer::Buffer;
use ratatui::crossterm::event::{Event, KeyCode};
use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::widgets::{List, ListItem, StatefulWidget, Widget};
use tokio::sync::mpsc::Sender;

use crate::application::query::plan::PlanDetail;
use crate::infrastructure::util::component::Component;
use crate::infrastructure::util::state::State;
use crate::ui::state::{PlanTabFocus, PlanTabState, PlanTreeListAction, PlanTreeListState};
use crate::ui::style;

pub struct PlanTreeListComponent {
    plan_tree_list_requester: Sender<PlanTreeListAction>,
    plan_tree_list_state: State<PlanTreeListState>,
    plan_tab_state: State<PlanTabState>,
}

impl PlanTreeListComponent {
    pub fn new(
        plan_tree_list_requester: Sender<PlanTreeListAction>,
        plan_tree_list_state: State<PlanTreeListState>,
        plan_tab_state: State<PlanTabState>,
    ) -> Self {
        Self {
            plan_tree_list_requester,
            plan_tree_list_state,
            plan_tab_state,
        }
    }
}

impl Component for PlanTreeListComponent {
    fn handle_input(&self, input: &Event) {
        if let Event::Key(key) = input {
            match key.code {
                KeyCode::Down | KeyCode::Char('j') => {
                    let _ = self
                        .plan_tree_list_requester
                        .try_send(PlanTreeListAction::SelectNext);
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    let _ = self
                        .plan_tree_list_requester
                        .try_send(PlanTreeListAction::SelectPrevious);
                }
                _ => {}
            }
        }
    }
}

impl Widget for &PlanTreeListComponent {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let plan_tree_list_state = self.plan_tree_list_state.get();
        let plan_detail = plan_tree_list_state.plan_detail();
        let mut list_state = plan_tree_list_state.list_state();

        let plan_tab_state = self.plan_tab_state.get();
        let is_focused = plan_tab_state.focus() == PlanTabFocus::PlanTreeList;

        let list_items = plan_detail.as_ref().map_or(vec![], |plan| {
            let mut items = vec![];
            flatten_plan_detail_to_list_items(plan, 0, &mut items);
            items
        });

        let list = List::new(list_items)
            .block(style::block_with_focused(is_focused).title(" Plan Tree "))
            .highlight_style(style::highlight_with_focused(is_focused));

        StatefulWidget::render(list, area, buf, &mut list_state);

        let _ = self
            .plan_tree_list_requester
            .try_send(PlanTreeListAction::SetListState(list_state));
    }
}

fn format_plan_item(_plan_detail: &PlanDetail, _indent: String) -> Vec<Line<'static>> {
    todo!("rewrite format_plan_item to use new PlanDetail structure")
}

fn flatten_plan_detail_to_list_items(
    _plan_detail: &PlanDetail,
    _depth: usize,
    _items: &mut Vec<ListItem>,
) {
    todo!("rewrite flatten_plan_detail_to_list_items to use new PlanDetail structure")
}
