use ratatui::buffer::Buffer;
use ratatui::crossterm::event::{Event, KeyCode};
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, List, ListItem, StatefulWidget, Widget};
use tokio::sync::mpsc::Sender;

use crate::application::query::plan::PlanDetail;
use crate::infrastructure::util::component::Component;
use crate::infrastructure::util::state::State;
use crate::ui::state::{PlanTabFocus, PlanTabState, PlanTreeListAction, PlanTreeListState};

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
        let state = self.plan_tree_list_state.get();
        let plan_detail = state.plan_detail();
        let mut list_state = state.list_state();
        let is_focused = self.plan_tab_state.get().focus() == PlanTabFocus::PlanTreeList;

        let list_items = plan_detail.as_ref().map_or(vec![], |plan| {
            let mut items = vec![];
            flatten_plan_detail_to_list_items(plan, 0, &mut items);
            items
        });

        let border_style = if is_focused {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray)
        };

        let highlight_style = if is_focused {
            Style::default()
                .bg(Color::Cyan)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };

        let list = List::new(list_items)
            .block(
                Block::new()
                    .border_type(BorderType::Plain)
                    .borders(Borders::all())
                    .border_style(border_style)
                    .title(" Plan Tree "),
            )
            .highlight_style(highlight_style);

        StatefulWidget::render(list, area, buf, &mut list_state);

        let _ = self
            .plan_tree_list_requester
            .try_send(PlanTreeListAction::SetListState(list_state));
    }
}

fn format_plan_item(plan_detail: &PlanDetail, indent: String) -> Vec<Line<'static>> {
    let mut line1_spans = vec![Span::raw(indent.clone())];
    line1_spans.push(Span::raw(format!("{}", plan_detail.goal_name())));
    line1_spans.push(Span::raw(" x "));

    let flow_text = if let Some(backward) = plan_detail.flow_backward() {
        format!(
            "(Effective {} + Backward {})",
            plan_detail.flow_effective(),
            backward
        )
    } else {
        format!("{}", plan_detail.flow_effective())
    };
    line1_spans.push(Span::raw(flow_text));

    if let Some(steps) = plan_detail.cyclic_steps_ahead() {
        line1_spans.push(Span::raw(format!(" [Cyclic: {} steps ahead]", steps)));
    }

    let mut line2_spans = vec![Span::raw(indent)];
    line2_spans.push(Span::raw("["));
    line2_spans.push(Span::raw(format!("{}", plan_detail.machine_name())));
    line2_spans.push(Span::raw("] x "));

    let replica_text = if let Some(backward) = plan_detail.replica_backward() {
        format!(
            "(Effective {} + Backward {})",
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
    let indent = "    ".repeat(depth);
    let lines = format_plan_item(plan_detail, indent);
    items.push(ListItem::new(lines));

    for dep in plan_detail.dependencies() {
        flatten_plan_detail_to_list_items(dep, depth + 1, items);
    }
}
