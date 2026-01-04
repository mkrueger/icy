//! Text inputs page

use icy_ui::widget::{column, space, text, text_input};
use icy_ui::Element;

use crate::Message;

#[derive(Default, Clone)]
pub struct TextInputsState {
    pub text_value: String,
    pub password_value: String,
    pub email_value: String,
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
        Message::EmailChanged(value) => {
            state.email_value = value.clone();
            true
        }
        _ => false,
    }
}

pub fn view_text_inputs(state: &TextInputsState) -> Element<'_, Message> {
    column![
        text("Text Input").size(18),
        space().height(10),
        // Basic text input with accessibility label and description
        text_input("Enter some text...", &state.text_value)
            .on_input(Message::TextChanged)
            .width(300)
            .a11y_label("Username")
            .a11y_description("Enter your username to sign in"),
        text(format!("Value: {}", state.text_value)).size(12),
        space().height(20),
        // Password input with accessibility label and required flag
        text("Password Input").size(18),
        space().height(10),
        text_input("Enter password...", &state.password_value)
            .on_input(Message::PasswordChanged)
            .secure(true)
            .width(300)
            .a11y_label("Password")
            .a11y_description("Must be at least 8 characters")
            .required(true),
        text(format!("Length: {} characters", state.password_value.len())).size(12),
        space().height(20),
        // Email input with accessibility label and required flag
        text("Email Input").size(18),
        space().height(10),
        text_input("Enter email address...", &state.email_value)
            .on_input(Message::EmailChanged)
            .width(300)
            .a11y_label("Email Address")
            .a11y_description("Enter a valid email address")
            .required(true),
        text(format!("Email: {}", state.email_value)).size(12),
        space().height(20),
        text("Text Input Sizes").size(18),
        space().height(10),
        column![
            text_input("Small", &state.text_value)
                .on_input(Message::TextChanged)
                .size(12)
                .width(200)
                .a11y_label("Small text field"),
            text_input("Normal", &state.text_value)
                .on_input(Message::TextChanged)
                .width(200)
                .a11y_label("Normal text field"),
            text_input("Large", &state.text_value)
                .on_input(Message::TextChanged)
                .size(20)
                .width(200)
                .a11y_label("Large text field"),
        ]
        .spacing(10),
    ]
    .spacing(4)
    .into()
}
