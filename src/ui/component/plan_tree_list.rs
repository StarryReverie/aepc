use ratatui::buffer::Buffer;
use ratatui::crossterm::event::{Event, KeyCode};
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{List, ListItem, StatefulWidget, Widget};
use tokio::sync::mpsc::Sender;

use crate::application::query::plan::{PlanDetail, PlanDetailNode, PlanNodeKind};
use crate::domain::machine::model::Power;
use crate::domain::recipe::model::{Flow, Replica};
use crate::infrastructure::util::component::Component;
use crate::infrastructure::util::state::State;
use crate::ui::state::{PlanTabFocus, PlanTabState, PlanTreeListAction, PlanTreeListState};
use crate::ui::style;

const COLOR_MACHINE: Color = Color::Rgb(100, 149, 237);
const COLOR_FLOW: Color = Color::Rgb(60, 179, 113);
const COLOR_POWER: Color = Color::Rgb(255, 191, 0);
const COLOR_PRODUCT: Color = Color::Rgb(64, 224, 208);
const COLOR_KIND: Color = Color::Rgb(255, 140, 0);

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
            flatten_plan_detail(plan, &mut items);
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

fn flatten_plan_detail(plan_detail: &PlanDetail, items: &mut Vec<ListItem<'static>>) {
    let goal_node = plan_detail.goal();
    let intermediates = plan_detail.common_intermediates();
    let recipes = plan_detail.common_recipes();

    let total_top_nodes = 1 + intermediates.len() + recipes.len();
    let mut is_last_sibling = Vec::new();

    is_last_sibling.push(0 == total_top_nodes - 1);
    flatten_node(goal_node, items, &mut is_last_sibling);
    is_last_sibling.pop();

    for (index, node) in intermediates.iter().enumerate() {
        is_last_sibling.push(1 + index == total_top_nodes - 1);
        flatten_node(node, items, &mut is_last_sibling);
        is_last_sibling.pop();
    }

    for (index, node) in recipes.iter().enumerate() {
        is_last_sibling.push(1 + intermediates.len() + index == total_top_nodes - 1);
        flatten_node(node, items, &mut is_last_sibling);
        is_last_sibling.pop();
    }
}

fn flatten_node(
    node: &PlanDetailNode,
    items: &mut Vec<ListItem<'static>>,
    is_last_sibling: &mut Vec<bool>,
) {
    let prefix = build_prefix(is_last_sibling);
    let line = format_node(node, prefix);
    items.push(ListItem::new(line));

    for (child_index, child) in node.children().iter().enumerate() {
        is_last_sibling.push(child_index == node.children().len() - 1);
        flatten_node(child, items, is_last_sibling);
        is_last_sibling.pop();
    }
}

fn build_prefix(is_last_sibling: &[bool]) -> String {
    let mut indent = String::new();
    for &is_last in is_last_sibling.iter().take(is_last_sibling.len() - 1) {
        indent.push_str(if is_last { "    " } else { "│   " });
    }
    let is_last_current = *is_last_sibling.last().unwrap();
    if is_last_current {
        indent.push_str("└── ");
    } else {
        indent.push_str("├── ");
    }
    indent
}

fn format_node(node: &PlanDetailNode, prefix: String) -> Line<'static> {
    match node {
        PlanDetailNode::Combined { target, recipe, .. } => to_line(
            prefix,
            vec![
                Span::raw(target.target_name().to_string()),
                Span::raw("@"),
                span_machine_name(recipe.machine_name().to_string()),
                span_flow(target.flow_all(), target.flow_cyclic()),
                span_replica(recipe.replica()),
                span_power(recipe.machine_power()),
                span_kind(node.kind()),
            ],
        ),
        PlanDetailNode::Target { target, .. } => to_line(
            prefix,
            vec![
                Span::raw(target.target_name().to_string()),
                Span::raw("@"),
                span_machine_name("...".to_string()),
                span_flow(target.flow_all(), target.flow_cyclic()),
                span_kind(node.kind()),
            ],
        ),
        PlanDetailNode::Recipe {
            recipe,
            product_names,
            ..
        } => to_line(
            prefix,
            vec![
                span_machine_name(recipe.machine_name().to_string()),
                Span::raw("=>"),
                Span::raw(
                    product_names
                        .iter()
                        .map(|p| p.value())
                        .collect::<Vec<_>>()
                        .join(" + "),
                ),
                span_replica(recipe.replica()),
                span_power(recipe.machine_power()),
                span_kind(node.kind()),
            ],
        ),
    }
}

fn to_line(prefix: String, spans: Vec<Span<'static>>) -> Line<'static> {
    let mut res = vec![Span::raw(prefix)];
    let mut first = true;
    for span in spans {
        if first {
            first = false;
            res.push(span);
        } else {
            res.push(Span::from(" "));
            res.push(span);
        }
    }
    Line::from(res)
}

fn span_machine_name(s: String) -> Span<'static> {
    Span::styled(s, Style::new().fg(COLOR_MACHINE))
}

fn span_flow(flow_all: Flow, flow_cyclic: Flow) -> Span<'static> {
    let str = if flow_cyclic == Flow::zero() {
        format!("[{}]", flow_all)
    } else {
        format!("[{} -> {} (back)]", flow_all.no_unit_display(), flow_cyclic)
    };
    Span::styled(str, Style::new().fg(COLOR_FLOW))
}

fn span_replica(replica: Replica) -> Span<'static> {
    let str = format!("[{}]", replica.compact_display());
    Span::styled(str, Style::new().fg(COLOR_PRODUCT))
}

fn span_power(power: Option<Power>) -> Span<'static> {
    let str = power.map_or("[-]".to_string(), |power| format!("[{power}]"));
    Span::styled(str, Style::new().fg(COLOR_POWER))
}

fn span_kind(kind: PlanNodeKind) -> Span<'static> {
    let str = match kind {
        PlanNodeKind::Source => String::new(),
        PlanNodeKind::Partial => "(partial)".to_string(),
        PlanNodeKind::Cyclic { from_steps_ahead } => format!("(cyclic: {}^)", from_steps_ahead),
    };
    match kind {
        PlanNodeKind::Source => Span::raw(str),
        _ => Span::styled(str, Style::new().fg(COLOR_KIND)),
    }
}
