use ratatui::buffer::Buffer;
use ratatui::crossterm::event::Event;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Widget};

use crate::infrastructure::util::component::Component;
use crate::infrastructure::util::state::State;
use crate::ui::state::{AppFocus, AppState, PlanTabFocus};
use crate::ui::style;

const COLOR_KEY: Color = Color::Rgb(119, 181, 248);
const COLOR_HINT: Color = Color::Rgb(180, 180, 180);
const COLOR_SEP: Color = Color::Rgb(80, 84, 92);

pub struct KeybindingBarComponent {
    app_state: State<AppState>,
}

impl KeybindingBarComponent {
    pub fn new(app_state: State<AppState>) -> Self {
        Self { app_state }
    }
}

impl Component for KeybindingBarComponent {
    fn handle_input(&self, _input: &Event) {}
}

impl Widget for &KeybindingBarComponent {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let app_state = self.app_state.get();
        let hints = Hint::lookup_hints(app_state.focus());

        let line = hints
            .iter()
            .flat_map(|hint| {
                [
                    Span::styled(hint.key, Style::new().fg(COLOR_KEY)),
                    Span::from(" "),
                    Span::styled(hint.description, Style::new().fg(COLOR_HINT)),
                    Span::styled("  ", Style::new().fg(COLOR_SEP)),
                ]
            })
            .collect::<Line>();

        let paragraph = Paragraph::new(line).block(style::block_default().title(" Keybindings "));
        paragraph.render(area, buf);
    }
}

struct Hint {
    key: &'static str,
    description: &'static str,
}

impl Hint {
    const HINTS_PLAN_TAB_GOAL_FLOW_INPUT: &[Hint] = &[
        Hint::new("q", "Quit"),
        Hint::new("Tab/S-Tab", "Switch Panel"),
        Hint::new("Enter", "Confirm"),
        Hint::new("Esc", "Clear"),
    ];

    const HINTS_PLAN_TAB_GOAL_ITEM_SEARCH_INPUT: &[Hint] = &[
        Hint::new("q", "Quit"),
        Hint::new("Tab/S-Tab", "Switch Panel"),
        Hint::new("Enter", "Confirm"),
        Hint::new("Esc", "Clear"),
    ];

    const HINTS_PLAN_TAB_GOAL_ITEM_SEARCH_LIST: &[Hint] = &[
        Hint::new("q", "Quit"),
        Hint::new("Tab/S-Tab", "Switch Panel"),
        Hint::new("j/k/Up/Down", "Navigate"),
        Hint::new("Enter", "Select"),
    ];

    const HINTS_PLAN_TAB_PLAN_TREE_LIST: &[Hint] = &[
        Hint::new("q", "Quit"),
        Hint::new("Tab/S-Tab", "Switch Panel"),
        Hint::new("j/k/Up/Down", "Navigate"),
    ];

    const fn new(key: &'static str, description: &'static str) -> Self {
        Self { key, description }
    }

    fn lookup_hints(focus: AppFocus) -> &'static [Hint] {
        match focus {
            AppFocus::PlanTab(plan_focus) => match plan_focus {
                PlanTabFocus::GoalFlowInput => Self::HINTS_PLAN_TAB_GOAL_FLOW_INPUT,
                PlanTabFocus::GoalItemSearchInput => Self::HINTS_PLAN_TAB_GOAL_ITEM_SEARCH_INPUT,
                PlanTabFocus::GoalItemSearchList => Self::HINTS_PLAN_TAB_GOAL_ITEM_SEARCH_LIST,
                PlanTabFocus::PlanTreeList => Self::HINTS_PLAN_TAB_PLAN_TREE_LIST,
            },
        }
    }
}
