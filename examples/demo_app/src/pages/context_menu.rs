//! Context menu page

use icy_ui::widget::{button, center, column, container, menu, space, text};
use icy_ui::{Center as CenterAlign, Element, Fill, Theme};

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

pub fn view_context_menu(state: &ContextMenuState) -> Element<'_, Message> {
    use menu::context_menu;

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

    let menu_items = vec![
        ("Cut", Message::ContextAction("Cut".into())),
        ("Copy", Message::ContextAction("Copy".into())),
        ("Paste", Message::ContextAction("Paste".into())),
        ("Delete", Message::ContextAction("Delete".into())),
    ];

    let ctx_menu = context_menu(
        target,
        Some(
            menu_items
                .into_iter()
                .map(|(label, msg)| {
                    menu::Tree::new(
                        button(text(label))
                            .on_press(msg)
                            .width(Fill)
                            .style(button::text_style),
                    )
                })
                .collect(),
        ),
    );

    column![
        text("Context Menu").size(18),
        space().height(10),
        text("Right-click on the area below to see a context menu:").size(14),
        space().height(10),
        ctx_menu,
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
