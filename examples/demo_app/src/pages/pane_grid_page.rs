//! Pane Grid page - demonstrates the pane_grid widget
//!
//! Pane grids allow users to split regions of the application and organize layout dynamically.

use icy_ui::widget::{button, column, container, pane_grid, row, text};
use icy_ui::{Color, Element, Fill};

// =============================================================================
// Pane Grid State
// =============================================================================

/// State for the pane grid demo page
#[derive(Clone)]
pub struct PaneGridPageState {
    /// The pane grid state
    pub panes: pane_grid::State<PaneContent>,
    /// Counter for generating unique pane IDs
    pane_counter: usize,
    /// Currently focused pane
    pub focus: Option<pane_grid::Pane>,
}

impl Default for PaneGridPageState {
    fn default() -> Self {
        let (panes, _) = pane_grid::State::new(PaneContent::new(1, "Welcome Pane"));
        Self {
            panes,
            pane_counter: 1,
            focus: None,
        }
    }
}

/// Content stored in each pane
#[derive(Debug, Clone)]
pub struct PaneContent {
    pub id: usize,
    pub title: String,
}

impl PaneContent {
    pub fn new(id: usize, title: impl Into<String>) -> Self {
        Self {
            id,
            title: title.into(),
        }
    }
}

// =============================================================================
// Update Function
// =============================================================================

/// Update the pane grid page state based on messages
pub fn update_pane_grid(state: &mut PaneGridPageState, message: &crate::Message) -> bool {
    match message {
        crate::Message::PaneGridSplitHorizontal(pane) => {
            state.pane_counter += 1;
            let new_content =
                PaneContent::new(state.pane_counter, format!("Pane {}", state.pane_counter));
            let _ = state
                .panes
                .split(pane_grid::Axis::Horizontal, *pane, new_content);
            true
        }
        crate::Message::PaneGridSplitVertical(pane) => {
            state.pane_counter += 1;
            let new_content =
                PaneContent::new(state.pane_counter, format!("Pane {}", state.pane_counter));
            let _ = state
                .panes
                .split(pane_grid::Axis::Vertical, *pane, new_content);
            true
        }
        crate::Message::PaneGridClose(pane) => {
            if let Some((_, sibling)) = state.panes.close(*pane) {
                state.focus = Some(sibling);
            }
            true
        }
        crate::Message::PaneGridDragged(event) => {
            match event {
                pane_grid::DragEvent::Dropped { pane, target } => {
                    state.panes.drop(*pane, *target);
                }
                pane_grid::DragEvent::Canceled { .. } => {}
                _ => {}
            }
            true
        }
        crate::Message::PaneGridResized(event) => {
            state.panes.resize(event.split, event.ratio);
            true
        }
        crate::Message::PaneGridClicked(pane) => {
            state.focus = Some(*pane);
            true
        }
        _ => false,
    }
}

// =============================================================================
// View Function
// =============================================================================

/// Create the view for the pane grid page
pub fn view_pane_grid(state: &PaneGridPageState) -> Element<'_, crate::Message> {
    let focus = state.focus;
    let total_panes = state.panes.len();

    let pane_grid_widget = pane_grid(&state.panes, |pane, content, _is_maximized| {
        let is_focused = focus == Some(pane);
        let title = text(&content.title).size(16);

        // Title bar with controls
        let title_bar = pane_grid::TitleBar::new(title)
            .controls(view_controls(pane, total_panes))
            .padding(8)
            .style(if is_focused {
                container::bordered_box
            } else {
                container::transparent
            });

        // Pane content
        let pane_content = container(
            column![
                text(format!("Pane ID: {}", content.id)).size(14),
                text("Click to focus, drag title bar to move")
                    .size(12)
                    .color(Color::from_rgb(0.5, 0.5, 0.5)),
                text("Use buttons to split or close panes")
                    .size(12)
                    .color(Color::from_rgb(0.5, 0.5, 0.5)),
            ]
            .spacing(8)
            .padding(10),
        )
        .width(Fill)
        .height(Fill)
        .center_x(Fill);

        pane_grid::Content::new(pane_content)
            .title_bar(title_bar)
            .style(if is_focused {
                container::bordered_box
            } else {
                container::transparent
            })
    })
    .width(Fill)
    .height(Fill)
    .spacing(2)
    .on_click(crate::Message::PaneGridClicked)
    .on_drag(crate::Message::PaneGridDragged)
    .on_resize(10, crate::Message::PaneGridResized);

    let instructions = column![
        text("Pane Grid Demo").size(24),
        text("A resizable, draggable pane grid layout. Split panes horizontally or vertically, drag to rearrange, and resize by dragging borders.")
            .size(14)
            .color(Color::from_rgb(0.6, 0.6, 0.6)),
    ]
    .spacing(5);

    let content = column![
        instructions,
        container(pane_grid_widget)
            .width(Fill)
            .height(Fill)
            .padding(5),
    ]
    .spacing(10)
    .padding(10)
    .width(Fill)
    .height(Fill);

    content.into()
}

/// View for the pane controls (split/close buttons)
fn view_controls(pane: pane_grid::Pane, total_panes: usize) -> Element<'static, crate::Message> {
    let split_h = button(text("⬌").size(14))
        .on_press(crate::Message::PaneGridSplitHorizontal(pane))
        .padding([2, 6])
        .style(button::secondary);

    let split_v = button(text("⬍").size(14))
        .on_press(crate::Message::PaneGridSplitVertical(pane))
        .padding([2, 6])
        .style(button::secondary);

    // Only show close button if there's more than one pane
    let close_btn: Element<'static, crate::Message> = if total_panes > 1 {
        button(text("✕").size(14))
            .on_press(crate::Message::PaneGridClose(pane))
            .padding([2, 6])
            .style(button::danger)
            .into()
    } else {
        // Placeholder for layout consistency
        button(text("✕").size(14))
            .padding([2, 6])
            .style(button::secondary)
            .into()
    };

    row![split_h, split_v, close_btn].spacing(4).into()
}
