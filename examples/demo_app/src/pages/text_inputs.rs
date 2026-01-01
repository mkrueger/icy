//! Text inputs page

use icy_ui::widget::{column, space, text, text_input};
use icy_ui::Element;

use crate::Message;

#[derive(Default, Clone)]
pub struct TextInputsState {
    pub text_value: String,
    pub password_value: String,
}

pub fn update_text_inputs(state: &mut TextInputsState, message: &Message) -> bool {
    match message {
        Message::TextChanged(value) => {
            state.text_value = value.clone();
            true
        }
        Message::PasswordChanged(value) => {
            state.password_value = value.clone();
            true
        }
        _ => false,
    }
}

pub fn view_text_inputs(state: &TextInputsState) -> Element<'_, Message> {
    column![
        text("Text Input").size(18),
        space().height(10),
        text_input("Enter some text...", &state.text_value)
            .on_input(Message::TextChanged)
            .width(300),
        text(format!("Value: {}", state.text_value)).size(12),
        space().height(20),
        text("Password Input").size(18),
        space().height(10),
        text_input("Enter password...", &state.password_value)
            .on_input(Message::PasswordChanged)
            .secure(true)
            .width(300),
        text(format!("Length: {} characters", state.password_value.len())).size(12),
        space().height(20),
        text("Text Input Sizes").size(18),
        space().height(10),
        column![
            text_input("Small", &state.text_value)
                .on_input(Message::TextChanged)
                .size(12)
                .width(200),
            text_input("Normal", &state.text_value)
                .on_input(Message::TextChanged)
                .width(200),
            text_input("Large", &state.text_value)
                .on_input(Message::TextChanged)
                .size(20)
                .width(200),
        ]
        .spacing(10),
    ]
    .spacing(4)
    .into()
}
