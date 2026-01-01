//! Pickers page

use icy_ui::widget::{button, color_picker, column, container, date_picker, row, space, text};
use icy_ui::{Center, Element};

use crate::Message;

#[derive(Clone)]
pub struct PickersState {
    pub selected_color: icy_ui::Color,
    pub show_color_picker: bool,
    pub selected_date: date_picker::Date,
    pub visible_date: date_picker::Date,
    pub show_date_picker: bool,
}

impl Default for PickersState {
    fn default() -> Self {
        let today = date_picker::Date::today();
        Self {
            selected_color: icy_ui::Color::from_rgb(0.5, 0.7, 0.9),
            show_color_picker: false,
            selected_date: today,
            visible_date: today,
            show_date_picker: false,
        }
    }
}

pub fn update_pickers(state: &mut PickersState, message: &Message) -> bool {
    match message {
        Message::ColorChanged(color) => {
            state.selected_color = *color;
            true
        }
        Message::ToggleColorPicker => {
            state.show_color_picker = !state.show_color_picker;
            true
        }
        Message::DateChanged(date) => {
            state.selected_date = *date;
            state.visible_date = *date;
            true
        }
        Message::DatePrevMonth => {
            state.visible_date = state.visible_date.prev_month();
            true
        }
        Message::DateNextMonth => {
            state.visible_date = state.visible_date.next_month();
            true
        }
        Message::ToggleDatePicker => {
            state.show_date_picker = !state.show_date_picker;
            true
        }
        _ => false,
    }
}

pub fn view_pickers(state: &PickersState) -> Element<'static, Message> {
    let color = state.selected_color;

    column![
        text("Color Picker").size(18),
        space().height(10),
        row![
            button(
                row![
                    container(space().width(20).height(20)).style(move |_theme| container::Style {
                        background: Some(color.into()),
                        border: icy_ui::Border::default()
                            .rounded(4)
                            .width(1)
                            .color(icy_ui::Color::BLACK),
                        ..Default::default()
                    }),
                    text(if state.show_color_picker {
                        "Close"
                    } else {
                        "Pick Color"
                    }),
                ]
                .spacing(8)
                .align_y(Center)
            )
            .on_press(Message::ToggleColorPicker),
            text(format!(
                "RGB: ({:.0}, {:.0}, {:.0})",
                state.selected_color.r * 255.0,
                state.selected_color.g * 255.0,
                state.selected_color.b * 255.0
            )),
        ]
        .spacing(20)
        .align_y(Center),
        if state.show_color_picker {
            Element::from(
                container(color_picker::color_picker(
                    state.selected_color,
                    Message::ColorChanged,
                ))
                .padding(10),
            )
        } else {
            Element::from(space().height(0))
        },
        space().height(30),
        text("Date Picker").size(18),
        space().height(10),
        row![
            button(
                row![
                    text("📅"),
                    text(if state.show_date_picker {
                        "Close"
                    } else {
                        "Pick Date"
                    }),
                ]
                .spacing(8)
            )
            .on_press(Message::ToggleDatePicker),
            text(format!(
                "Selected: {:04}-{:02}-{:02}",
                state.selected_date.year, state.selected_date.month, state.selected_date.day
            )),
        ]
        .spacing(20)
        .align_y(Center),
        if state.show_date_picker {
            Element::from(
                container(date_picker::date_picker(
                    state.selected_date,
                    state.visible_date,
                    Message::DateChanged,
                    Message::DatePrevMonth,
                    Message::DateNextMonth,
                ))
                .padding(10),
            )
        } else {
            Element::from(space().height(0))
        },
    ]
    .spacing(4)
    .into()
}
