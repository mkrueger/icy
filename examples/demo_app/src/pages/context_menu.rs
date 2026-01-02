//! Context menu page

use icy_ui::menu;
use icy_ui::widget::menu::context_menu;
use icy_ui::widget::{center, column, container, space, text};
use icy_ui::{Center as CenterAlign, Element, Theme};

use crate::Message;

#[derive(Default, Clone)]
pub struct ContextMenuState {
    pub context_menu_action: String,
}

pub fn update_context_menu(state: &mut ContextMenuState, message: &Message) -> Option<String> {
    match message {
        Message::ContextAction(action) => {
            state.context_menu_action = action.clone();
            Some(format!("Context action: {}", action))
        }
        _ => None,
    }
}

/// Creates the context menu nodes for this page
pub fn context_menu_nodes() -> Vec<menu::MenuNode<Message>> {
    vec![
        menu::item!("Cut", Message::ContextAction("Cut".into())),
        menu::item!("Copy", Message::ContextAction("Copy".into())),
        menu::item!("Paste", Message::ContextAction("Paste".into())),
        menu::separator!(),
        menu::submenu!(
            "More Actions",
            [
                menu::item!("Select All", Message::ContextAction("Select All".into())),
                menu::item!("Find", Message::ContextAction("Find".into())),
                menu::separator!(),
                menu::item!("Replace", Message::ContextAction("Replace".into())),
            ]
        ),
    ]
}

pub fn view_context_menu(state: &ContextMenuState) -> Element<'_, Message> {
    let target = container(center(
        column![
            text("Right-click here").size(16),
            text("to open context menu"),
        ]
        .align_x(CenterAlign),
    ))
    .width(300)
    .height(200)
    .style(|_theme: &Theme| container::Style {
        background: Some(icy_ui::Color::from_rgba(0.5, 0.5, 0.5, 0.2).into()),
        border: icy_ui::Border::default()
            .width(2)
            .color(icy_ui::Color::from_rgb(0.2, 0.5, 0.8))
            .rounded(8),
        ..Default::default()
    });

    // Use context_menu widget - automatically uses native menu on macOS
    let nodes = context_menu_nodes();
    let interactive_target = context_menu(target, &nodes);

    column![
        text("Context Menu").size(18),
        space().height(10),
        text("Right-click on the area below to open a context menu:").size(14),
        text("On macOS, this shows a native NSMenu. On other platforms, an overlay menu.").size(12),
        space().height(10),
        interactive_target,
        space().height(20),
        text(format!(
            "Last action: {}",
            if state.context_menu_action.is_empty() {
                "None"
            } else {
                &state.context_menu_action
            }
        ))
        .size(14),
    ]
    .spacing(4)
    .into()
}
