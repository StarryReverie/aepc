use ratatui::buffer::Buffer;
use ratatui::crossterm::event::{Event, KeyCode};
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
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

fn format_plan_item(plan_detail: &PlanDetail, indent: String) -> Vec<Line<'static>> {
    let mut line1_spans = vec![Span::raw(indent.clone())];
    line1_spans.push(Span::raw(plan_detail.goal_name().to_string()));
    line1_spans.push(Span::raw(" x "));

    let flow_text = if let Some(backward) = plan_detail.flow_backward() {
        format!(
            "({} effective + {} backward)",
            plan_detail.flow_effective(),
            backward
        )
    } else {
        format!("{}", plan_detail.flow_effective())
    };
    line1_spans.push(Span::raw(flow_text));

    if let Some(steps) = plan_detail.cyclic_steps_ahead() {
        line1_spans.push(Span::raw(format!(" (cyclic: {} step(s) ahead)", steps)));
    }

    let mut line2_spans = vec![Span::raw(indent)];
    line2_spans.push(Span::raw("["));
    line2_spans.push(Span::raw(plan_detail.machine_name().to_string()));
    line2_spans.push(Span::raw("] x "));

    let replica_text = if let Some(backward) = plan_detail.replica_backward() {
        format!(
            "({} effective + {} backward)",
            plan_detail.replica_effective(),
            backward
        )
    } else {
        format!("{}", plan_detail.replica_effective())
    };
    line2_spans.push(Span::raw(replica_text));
    line2_spans.push(Span::raw(format!(" ({})", plan_detail.machine_power())));

    vec![Line::from(line1_spans), Line::from(line2_spans)]
}

fn flatten_plan_detail_to_list_items(
    plan_detail: &PlanDetail,
    depth: usize,
    items: &mut Vec<ListItem>,
) {
    let indent = "    ".repeat(depth) + " ";
    let lines = format_plan_item(plan_detail, indent);
    items.push(ListItem::new(lines));

    for dep in plan_detail.dependencies() {
        flatten_plan_detail_to_list_items(dep, depth + 1, items);
    }
}
