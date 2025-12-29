//! This example demonstrates the focus system with various controls.
//!
//! Use Tab to move focus forward, Shift+Tab to move focus backward.
//! The focus mode can be toggled between TextOnly and AllControls.

use iced::keyboard::{self, key::Named, Key};
use iced::widget::operation::{focus_next_filtered, focus_previous_filtered, FocusLevel};
use iced::widget::{
    button, checkbox, column, container, pick_list, radio_group, row, scrollable, slider, text,
    text_input, toggler, Space,
};
use iced::{Center, Element, Fill, Subscription, Task};

pub fn main() -> iced::Result {
    iced::application(Focus::default, Focus::update, Focus::view)
        .subscription(Focus::subscription)
        .run()
}

#[derive(Debug, Clone)]
enum Message {
    // Focus navigation
    FocusNext,
    FocusPrevious,

    // Focus level toggle
    FocusLevelChanged(bool),

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

struct Focus {
    focus_level: FocusLevel,
    text_input_value: String,
    text_input2_value: String,
    button_presses: u32,
    checkbox_checked: bool,
    toggler_enabled: bool,
    radio_choice: Option<RadioChoice>,
    slider_value: f32,
    pick_list_selection: Option<String>,
}

impl Default for Focus {
    fn default() -> Self {
        Self {
            focus_level: FocusLevel::TextOnly,
            text_input_value: String::new(),
            text_input2_value: String::new(),
            button_presses: 0,
            checkbox_checked: false,
            toggler_enabled: false,
            radio_choice: None,
            slider_value: 50.0,
            pick_list_selection: None,
        }
    }
}

impl Focus {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::FocusNext => {
                return focus_next_filtered(self.focus_level);
            }
            Message::FocusPrevious => {
                return focus_previous_filtered(self.focus_level);
            }
            Message::FocusLevelChanged(all_controls) => {
                self.focus_level = if all_controls {
                    FocusLevel::AllControls
                } else {
                    FocusLevel::TextOnly
                };
            }
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
        let focus_mode_info = text(format!(
            "Focus Mode: {} (Tab/Shift+Tab to navigate)",
            match self.focus_level {
                FocusLevel::TextOnly => "Text Only - Only text inputs receive focus",
                FocusLevel::AllControls => "All Controls - All interactive widgets receive focus",
                FocusLevel::Manual => "Manual",
            }
        ))
        .size(14);

        let focus_toggle = toggler(self.focus_level == FocusLevel::AllControls)
            .label("Enable Full Keyboard Access (All Controls)")
            .on_toggle(Message::FocusLevelChanged);

        let header = column![
            text("Focus System Demo").size(24),
            focus_mode_info,
            focus_toggle,
            Space::new().height(20),
        ]
        .spacing(10);

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
            text("Buttons (focusable with All Controls):").size(18),
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
            text("Checkbox (focusable with All Controls):").size(18),
            checkbox(self.checkbox_checked)
                .label("Check me (Space to toggle)")
                .on_toggle(Message::CheckboxToggled)
                .id("checkbox-1"),
        ]
        .spacing(10);

        // Toggler
        let toggler_section = column![
            text("Toggler (focusable with All Controls):").size(18),
            toggler(self.toggler_enabled)
                .label("Toggle me (Space to toggle)")
                .on_toggle(Message::TogglerToggled)
                .id("toggler-1"),
        ]
        .spacing(10);

        // Radio buttons (using RadioGroup for proper keyboard navigation)
        let radio_section = column![
            text("Radio Group (Arrow Up/Down to navigate, single Tab stop):").size(18),
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
            text("Slider (focusable with All Controls):").size(18),
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
            text("Pick List (focusable with All Controls):").size(18),
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

    fn subscription(&self) -> Subscription<Message> {
        fn handle_tab(event: keyboard::Event) -> Option<Message> {
            let keyboard::Event::KeyPressed { key, modifiers, .. } = event else {
                return None;
            };

            println!(
                "[focus] key pressed: {:?} (shift={}, ctrl={}, alt={}, logo={})",
                key,
                modifiers.shift(),
                modifiers.control(),
                modifiers.alt(),
                modifiers.logo()
            );

            match key.as_ref() {
                Key::Named(Named::Tab) => {
                    if modifiers.shift() {
                        println!("[focus] -> FocusPrevious (Shift+Tab)");
                        Some(Message::FocusPrevious)
                    } else {
                        println!("[focus] -> FocusNext (Tab)");
                        Some(Message::FocusNext)
                    }
                }
                _ => None,
            }
        }

        keyboard::listen().filter_map(handle_tab)
    }
}
