//! Toggles page

use icy_ui::widget::{checkbox, column, radio, space, text, toggler};
use icy_ui::Element;

use crate::{Message, RadioChoice};

#[derive(Clone)]
pub struct TogglesState {
    pub checkbox_value: bool,
    pub toggler_value: bool,
    pub radio_value: Option<RadioChoice>,
}

impl Default for TogglesState {
    fn default() -> Self {
        Self {
            checkbox_value: false,
            toggler_value: false,
            radio_value: None,
        }
    }
}

pub fn update_toggles(state: &mut TogglesState, message: &Message) -> bool {
    match message {
        Message::CheckboxToggled(value) => {
            state.checkbox_value = *value;
            true
        }
        Message::TogglerToggled(value) => {
            state.toggler_value = *value;
            true
        }
        Message::RadioSelected(choice) => {
            state.radio_value = Some(*choice);
            true
        }
        _ => false,
    }
}

pub fn view_toggles(state: &TogglesState) -> Element<'static, Message> {
    column![
        text("Checkbox").size(18),
        space().height(10),
        checkbox(state.checkbox_value)
            .label("Enable feature")
            .on_toggle(Message::CheckboxToggled),
        text(format!("Checked: {}", state.checkbox_value)).size(12),
        space().height(20),
        text("Toggler").size(18),
        space().height(10),
        toggler(state.toggler_value)
            .label("Dark mode simulation")
            .on_toggle(Message::TogglerToggled),
        text(format!("Enabled: {}", state.toggler_value)).size(12),
        space().height(20),
        text("Radio Buttons").size(18),
        space().height(10),
        column![
            radio(
                "Option 1 - First choice",
                RadioChoice::Option1,
                state.radio_value,
                Message::RadioSelected,
            ),
            radio(
                "Option 2 - Second choice",
                RadioChoice::Option2,
                state.radio_value,
                Message::RadioSelected,
            ),
            radio(
                "Option 3 - Third choice",
                RadioChoice::Option3,
                state.radio_value,
                Message::RadioSelected,
            ),
        ]
        .spacing(8),
        text(format!(
            "Selected: {}",
            state
                .radio_value
                .map(|r| r.to_string())
                .unwrap_or_else(|| "None".into())
        ))
        .size(12),
    ]
    .spacing(4)
    .into()
}
