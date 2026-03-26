use ratatui::buffer::Buffer;
use ratatui::crossterm::event::{Event, KeyCode};
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{List, ListItem, StatefulWidget, Widget};
use tokio::sync::mpsc::Sender;

use crate::application::query::plan::{PlanDetail, PlanDetailNode};
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

fn flatten_plan_detail_to_list_items(
    plan_detail: &PlanDetail,
    _depth: usize,
    items: &mut Vec<ListItem<'static>>,
) {
    flatten_node_to_list_items(plan_detail.goal(), 0, items);

    for intermediate in plan_detail.common_intermediates() {
        flatten_node_to_list_items(intermediate, 0, items);
    }

    for recipe in plan_detail.common_recipes() {
        flatten_node_to_list_items(recipe, 0, items);
    }
}

fn flatten_node_to_list_items(
    node: &PlanDetailNode,
    depth: usize,
    items: &mut Vec<ListItem<'static>>,
) {
    let indent = "    ".repeat(depth) + " ";
    let line = format_plan_detail_node(node, &indent);
    items.push(ListItem::new(line.clone()));

    for dep in node.children() {
        flatten_node_to_list_items(dep, depth + 1, items);
    }
}

fn format_plan_detail_node(node: &PlanDetailNode, indent: &str) -> Line<'static> {
    match node {
        PlanDetailNode::Combined { target, recipe, .. } => to_line(
            indent,
            vec![
                Span::raw(target.target_name().to_string()),
                Span::raw("@"),
                span_machine_name(recipe.machine_name().to_string()),
                span_flow(target.flow_all(), target.flow_cyclic()),
                span_replica(recipe.replica()),
                span_power(recipe.machine_power()),
            ],
        ),
        PlanDetailNode::Target { target, .. } => to_line(
            indent,
            vec![
                Span::raw(target.target_name().to_string()),
                Span::raw("@"),
                span_machine_name("...".to_string()),
                span_flow(target.flow_all(), target.flow_cyclic()),
            ],
        ),
        PlanDetailNode::Recipe {
            recipe,
            product_names,
            ..
        } => {
            // let product_names = ;
            to_line(
                indent,
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
                ],
            )
        }
    }
}

fn to_line(indent: &str, spans: Vec<Span<'static>>) -> Line<'static> {
    let mut res = vec![Span::raw(indent.to_string())];
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
        format!("[{} -> {} back]", flow_all, flow_cyclic)
    };
    Span::styled(str, Style::new().fg(COLOR_FLOW))
}

fn span_replica(replica: Replica) -> Span<'static> {
    let str = format!("[{replica}]");
    Span::styled(str, Style::new().fg(COLOR_PRODUCT))
}

fn span_power(power: Option<Power>) -> Span<'static> {
    let str = power.map_or("[-]".to_string(), |power| format!("[{power}]"));
    Span::styled(str, Style::new().fg(COLOR_POWER))
}
