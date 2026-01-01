//! Buttons page

use icy_ui::widget::{button, column, image, row, space, spin_button, text, tooltip};
use icy_ui::Length;
use icy_ui::{Element, Task};
use std::sync::LazyLock;

use crate::Message;

const FERRIS_BYTES: &[u8] = include_bytes!("../../assets/ferris.png");

/// Cached image handle for Ferris - created once, reused on every render
static FERRIS_HANDLE: LazyLock<image::Handle> =
    LazyLock::new(|| image::Handle::from_bytes(FERRIS_BYTES));

/// State for the buttons page
pub struct ButtonsState {
    pub click_count: u32,
    pub spin_value: i32,
}

impl Default for ButtonsState {
    fn default() -> Self {
        Self {
            click_count: 0,
            spin_value: 0,
        }
    }
}

/// Update the buttons page state
pub fn update_buttons(
    state: &mut ButtonsState,
    message: &Message,
) -> Option<(Task<Message>, String)> {
    match message {
        Message::ButtonClicked => {
            state.click_count += 1;
            Some((
                Task::none(),
                format!("Button clicked {} times", state.click_count),
            ))
        }
        Message::SpinValueChanged(value) => {
            state.spin_value = *value;
            Some((Task::none(), format!("Spin value: {value}")))
        }
        _ => None,
    }
}

pub fn view_buttons(state: &ButtonsState) -> Element<'static, Message> {
    let link: Element<'static, Message> =
        button::hyperlink("Visit GitHub", "https://github.com").into();

    let image_btn: Element<'static, Message> = button::image_button(FERRIS_HANDLE.clone())
        .image_width(Length::Fixed(32.0))
        .image_height(Length::Fixed(32.0))
        .padding(6)
        .on_press(Message::ButtonClicked)
        .into();

    let spin: Element<'static, Message> = spin_button(
        state.spin_value.to_string(),
        state.spin_value,
        1,
        0,
        100,
        Message::SpinValueChanged,
    )
    .into();

    column![
        // Row 1: Button Styles
        text("Button Styles").size(18),
        space().height(10),
        row![
            button("Primary")
                .style(button::primary)
                .on_press(Message::ButtonClicked),
            button("Secondary")
                .style(button::secondary)
                .on_press(Message::ButtonClicked),
            button("Success")
                .style(button::success)
                .on_press(Message::ButtonClicked),
            button("Danger")
                .style(button::danger)
                .on_press(Message::ButtonClicked),
        ]
        .spacing(10),
        space().height(20),
        // Row 2: Button States + Button with Tooltip
        row![
            column![
                text("Button States").size(18),
                space().height(10),
                row![
                    button("Enabled").on_press(Message::ButtonClicked),
                    button("Disabled"),
                ]
                .spacing(10),
            ],
            space().width(40),
            column![
                text("Button with Tooltip").size(18),
                space().height(10),
                tooltip(
                    button("Hover me!").on_press(Message::ButtonClicked),
                    text("This is a tooltip!"),
                    tooltip::Position::Top,
                ),
            ],
        ],
        space().height(20),
        // Row 3: Hyperlink + Image Button + Spin Button
        row![
            column![text("Hyperlink").size(18), space().height(10), link,],
            space().width(40),
            column![text("Image Button").size(18), space().height(10), image_btn,],
            space().width(40),
            column![text("Spin Button").size(18), space().height(10), spin,],
        ],
        space().height(20),
        text(format!("Click counter: {}", state.click_count)).size(16),
    ]
    .spacing(4)
    .into()
}
