//! This example demonstrates the automatic focus system in icy.
//!
//! Tab navigation is handled automatically by the framework based on the
//! `focus_level` setting. No manual keyboard subscription is needed!
//!
//! The default `FocusLevel::AllControls` enables Tab focus for all interactive widgets.
//! Use `FocusLevel::TextOnly` for macOS-like behavior (only text inputs get Tab focus).

use icy_ui::widget::{
    button, checkbox, column, container, pick_list, radio_group, row, scrollable, slider, text,
    text_input, toggler, Space,
};
use icy_ui::{Center, Element, Fill, Task};

pub fn main() -> icy_ui::Result {
    // The focus_level can be configured via Settings:
    // icy_ui::application(...)
    //     .settings(Settings { focus_level: FocusLevel::TextOnly, ..Default::default() })
    //
    // Default is FocusLevel::AllControls for full keyboard accessibility.
    icy_ui::application(Focus::default, Focus::update, Focus::view).run()
}

#[derive(Debug, Clone)]
enum Message {
    // Widget interactions
    TextInputChanged(String),
    TextInput2Changed(String),
    ButtonPressed,
    CheckboxToggled(bool),
    TogglerToggled(bool),
    RadioSelected(RadioChoice),
    SliderChanged(f32),
    PickListSelected(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RadioChoice {
    Option1,
    Option2,
    Option3,
}

impl std::fmt::Display for RadioChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RadioChoice::Option1 => write!(f, "Option 1"),
            RadioChoice::Option2 => write!(f, "Option 2"),
            RadioChoice::Option3 => write!(f, "Option 3"),
        }
    }
}

#[derive(Default)]
struct Focus {
    text_input_value: String,
    text_input2_value: String,
    button_presses: u32,
    checkbox_checked: bool,
    toggler_enabled: bool,
    radio_choice: Option<RadioChoice>,
    slider_value: f32,
    pick_list_selection: Option<String>,
}

impl Focus {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::TextInputChanged(value) => {
                self.text_input_value = value;
            }
            Message::TextInput2Changed(value) => {
                self.text_input2_value = value;
            }
            Message::ButtonPressed => {
                self.button_presses += 1;
            }
            Message::CheckboxToggled(checked) => {
                self.checkbox_checked = checked;
            }
            Message::TogglerToggled(enabled) => {
                self.toggler_enabled = enabled;
            }
            Message::RadioSelected(choice) => {
                self.radio_choice = Some(choice);
            }
            Message::SliderChanged(value) => {
                self.slider_value = value;
            }
            Message::PickListSelected(selection) => {
                self.pick_list_selection = Some(selection);
            }
        }

        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        let header = column![
            text("Automatic Focus System Demo").size(24),
            text("Tab navigation is handled automatically by icy!").size(14),
            text("Press Tab to move forward, Shift+Tab to move backward.").size(14),
            Space::new().height(10),
            text("Configure focus behavior via Settings::focus_level:").size(12),
            text("  • AllControls (default): All widgets receive Tab focus").size(12),
            text("  • TextOnly: Only text inputs receive Tab focus").size(12),
            text("  • Manual: Application handles focus via subscriptions").size(12),
            Space::new().height(20),
        ]
        .spacing(5);

        // Text inputs (always focusable)
        let text_section = column![
            text("Text Inputs (always focusable):").size(18),
            text_input("First text input...", &self.text_input_value)
                .on_input(Message::TextInputChanged)
                .id("text-input-1"),
            text_input("Second text input...", &self.text_input2_value)
                .on_input(Message::TextInput2Changed)
                .id("text-input-2"),
        ]
        .spacing(10);

        // Buttons
        let button_section = column![
            text("Buttons (focusable with AllControls):").size(18),
            row![
                button("Press Me (Space/Enter)")
                    .on_press(Message::ButtonPressed)
                    .id("button-1"),
                text(format!("Pressed {} times", self.button_presses)),
            ]
            .spacing(10)
            .align_y(Center),
        ]
        .spacing(10);

        // Checkbox
        let checkbox_section = column![
            text("Checkbox (focusable with AllControls):").size(18),
            checkbox(self.checkbox_checked)
                .label("Check me (Space to toggle)")
                .on_toggle(Message::CheckboxToggled)
                .id("checkbox-1"),
        ]
        .spacing(10);

        // Toggler
        let toggler_section = column![
            text("Toggler (focusable with AllControls):").size(18),
            toggler(self.toggler_enabled)
                .label("Toggle me (Space to toggle)")
                .on_toggle(Message::TogglerToggled)
                .id("toggler-1"),
        ]
        .spacing(10);

        // Radio buttons
        let radio_section = column![
            text("Radio Group (Arrow Up/Down to navigate):").size(18),
            radio_group(
                [
                    RadioChoice::Option1,
                    RadioChoice::Option2,
                    RadioChoice::Option3
                ],
                self.radio_choice,
                Message::RadioSelected
            )
            .id("radio-group-1"),
        ]
        .spacing(5);

        // Slider
        let slider_section = column![
            text("Slider (focusable with AllControls):").size(18),
            row![
                slider(0.0..=100.0, self.slider_value, Message::SliderChanged)
                    .id("slider-1")
                    .width(200),
                text(format!("{:.0}", self.slider_value)),
            ]
            .spacing(10)
            .align_y(Center),
            text("Use arrow keys to adjust when focused").size(12),
        ]
        .spacing(5);

        // Pick list
        let pick_list_options = vec![
            "Choice A".to_string(),
            "Choice B".to_string(),
            "Choice C".to_string(),
        ];
        let pick_list_section = column![
            text("Pick List (focusable with AllControls):").size(18),
            pick_list(
                pick_list_options,
                self.pick_list_selection.clone(),
                Message::PickListSelected
            )
            .placeholder("Select an option...")
            .id("pick-list-1"),
        ]
        .spacing(10);

        let content = column![
            header,
            text_section,
            Space::new().height(10),
            button_section,
            Space::new().height(10),
            checkbox_section,
            Space::new().height(10),
            toggler_section,
            Space::new().height(10),
            radio_section,
            Space::new().height(10),
            slider_section,
            Space::new().height(10),
            pick_list_section,
            Space::new().height(20),
        ]
        .spacing(5)
        .padding(20);

        container(scrollable(content))
            .width(Fill)
            .height(Fill)
            .into()
    }
}
